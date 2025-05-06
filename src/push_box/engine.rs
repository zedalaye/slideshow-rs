use std::path::PathBuf;
use raylib::prelude::*;
use crate::load_texture_with_exif_rotation;
use crate::push_box::slide::Slide;
use crate::push_box::state::PushBoxState;

pub struct PushBoxEngine {
    slides: Vec<Slide>,
    current_slide_index: usize,
}

impl crate::engine::Engine for PushBoxEngine {
    fn new() -> Self {
        Self {
            slides: Vec::new(),
            current_slide_index: 0,
        }
    }

    fn initialize(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, paths: Vec<PathBuf>) -> bool {
        for path in paths {
            match load_texture_with_exif_rotation(rl, thread, &path) {
                Ok(image) => {
                    self.slides.push(Slide::new(image));
                }
                Err(e) => {
                    println!("Failed to load image: {}", e);
                }
            }
        }

        // If there are slides, start the first one
        if !self.slides.is_empty() {
            self.slides[0].visible = true;
            self.slides[0].is_animating = true;
            return true;
        }

        false
    }

    fn render_frame(&mut self, dt: f32, rl: &mut RaylibHandle, thread: &RaylibThread, framebuffer: &mut RenderTexture2D) -> bool {      

        // Iterate only current, previous and next slides
        let mut slides_to_update = vec![self.current_slide_index];
        if self.current_slide_index > 0 {
            slides_to_update.push(self.current_slide_index - 1);
        }
        if self.current_slide_index < self.slides.len() - 1 {
            slides_to_update.push(self.current_slide_index + 1);
        }

        let mut is_animating = false;
        let mut animate_next_slide = false;

        for slide_index in slides_to_update.iter() {
            let slide = &mut self.slides[*slide_index];

            let slide_state_before = slide.state.clone();
            slide.update(dt);
            let slide_state_after = slide.state.clone();

            // If current slide is exiting, move to next slide
            if *slide_index == self.current_slide_index && slide_state_before == PushBoxState::ZoomingOut && slide_state_after == PushBoxState::Exiting {
                animate_next_slide = true;
                is_animating = true;
            }    

            is_animating = slide.is_animating || is_animating;
        }

        if !is_animating {
            return false;
        }

        // Activate next slide
        if animate_next_slide && self.current_slide_index < self.slides.len() - 1 {
            self.current_slide_index += 1;
            self.slides[self.current_slide_index].visible = true;
            self.slides[self.current_slide_index].is_animating = true;
        }

        rl.draw_texture_mode(thread, framebuffer,  |mut tmd| {
            let mut d = tmd.begin_drawing(thread);
            d.clear_background(Color::BLACK);
            
            for slide in self.slides.iter() {
                if slide.visible {
                    slide.draw(&mut d);
                }
            }
        });   

        true
    }
}
