use crate::spiral::layout::Layout;
use crate::spiral::state::SpiralState;
use raylib::prelude::*;
use std::path::PathBuf;
use crate::texture_loader::load_texture_with_exif_rotation;
use crate::constants::*;

pub struct SpiralEngine {
    layout: Layout,

    state: SpiralState,

    current_slide_index: usize,
    display_timer: f32,
    cleanup_timer: f32,
    cleanup_index: usize,
}

impl crate::engine::Engine for SpiralEngine {
    fn new() -> Self {
        Self {
            layout: Layout::new(),
            state: SpiralState::Displaying,
            current_slide_index: 0,
            display_timer: 0.0,
            cleanup_timer: 0.0,
            cleanup_index: 0,
        }
    }

    fn initialize(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, paths: Vec<PathBuf>) -> bool {
        for path in paths {
            match load_texture_with_exif_rotation(rl, thread, &path) {
                Ok(image) => {
                    self.layout.add_image(image);
                }
                Err(e) => {
                    println!("Failed to load image: {}", e);
                }
            }
        }
        self.layout.compute_layout();

        // Set cleanup index to the last slide
        self.cleanup_index = if self.layout.slides.len() > 0 { self.layout.slides.len() - 1 } else { 0 };

        return !self.layout.slides.is_empty();
    }

    fn render_frame(&mut self, dt: f32, rl: &mut RaylibHandle, thread: &RaylibThread, framebuffer: &mut RenderTexture2D) -> bool {
        for slide in self.layout.slides.iter_mut() {
            slide.update(dt);
        }

        // 2. Update slideshow state machine
        match self.state {
            SpiralState::Displaying => {
                self.display_timer += dt;
                if self.display_timer >= DISPLAY_DURATION {
                    // Time to start transition
                    if self.current_slide_index < self.layout.slides.len() {
                        // Start the current slide's background animation
                        self.layout.slides[self.current_slide_index].start_background_animation();
                        // Change state to wait for animation to finish
                        self.state = SpiralState::Transitioning;
                        // DO NOT increment index or reset timer here yet
                    } else {
                        // Should not happen if logic is correct, but safety first
                        self.state = SpiralState::Cleanup;
                        self.cleanup_timer = 0.0;
                    }
                }
            }
            SpiralState::Transitioning => {
                // Check if the *current* slide (which is animating) has finished
                if self.current_slide_index < self.layout.slides.len() && !self.layout.slides[self.current_slide_index].is_animating {
                    // Animation finished, move to the next slide
                    self.current_slide_index += 1;

                    // Check if that was the last slide
                    if self.current_slide_index >= self.layout.slides.len() {
                        self.state = SpiralState::Cleanup;
                        // Setup cleanup index (points to the last slide that animated)
                        self.cleanup_index = self.current_slide_index -1; // Correct index
                        self.cleanup_timer = 0.0;
                    } else {
                        // More slides remain, go back to displaying the new current slide
                        self.state = SpiralState::Displaying;
                        self.display_timer = 0.0; // Reset display timer for the new slide
                    }
                }
                // Else: Still animating, do nothing, stay in Transitioning state
            }
            SpiralState::Cleanup => {
                self.cleanup_timer += dt;
                if self.cleanup_timer >= CLEANUP_INTERVAL {
                    // Hide the slide at cleanup_index
                    // Check bounds just in case cleanup_index became invalid somehow
                    if let Some(slide) = self.layout.slides.get_mut(self.cleanup_index) {
                        slide.visible = false;
                    } else if self.cleanup_index == 0 && self.layout.slides.len() == 1 {
                        // Handle edge case: only one slide, hide it
                        self.layout.slides[0].visible = false;
                    }

                    if self.cleanup_index > 0 {
                        self.cleanup_index -= 1;
                        self.cleanup_timer = 0.0;
                    } else {
                        // Finished hiding slide 0
                        self.state = SpiralState::Finished;
                    }
                }
            }
            SpiralState::Finished => {
                return false;
            }
        }     

        rl.draw_texture_mode(thread, framebuffer,  |mut tmd| {
            let mut d = tmd.begin_drawing(thread);
            d.clear_background(Color::BLACK);

            // CORRECTED Drawing Logic: Draw background slides and the current active slide
            for (i, slide) in self.layout.slides.iter().enumerate() {
                // Draw if it's a background slide (index less than current)
                // OR if it's the current slide being displayed or transitioning.
                // The Cleanup/Finished states are implicitly handled because
                // background slides (i < current_slide_index) will be drawn,
                // and slide.draw() checks the slide.visible flag.
                if i < self.current_slide_index ||
                    (i == self.current_slide_index && (self.state == SpiralState::Displaying || self.state == SpiralState::Transitioning))
                {
                    // The slide.draw() method internally checks for slide.visible,
                    // which handles the cleanup phase correctly.
                    slide.draw(&mut d);
                }
                // Slides with index > current_slide_index are not drawn yet.
            }
        });   

        return true;
    }
}