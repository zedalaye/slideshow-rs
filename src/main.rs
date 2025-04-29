use std::env;
use std::process;
use std::process::{Command, Stdio};
use std::io::Write;
use std::time::Duration;
use raylib::prelude::*;

mod constants;
mod state;
mod slide;
mod texture_loader;

use crate::constants::*;
use crate::state::SlideshowState;
use crate::slide::Slide;
use crate::texture_loader::{load_sorted_image_paths, load_texture_with_exif_rotation};

fn main() {
    // --- Get Directory from Command Line ---
    let args: Vec<String> = env::args().collect(); // Collect args into a vector

    // Check if the directory argument was provided
    if args.len() < 2 {
        // Print usage message to standard error
        eprintln!(
            "Usage: {} <image_directory>",
            args.get(0).map_or("slideshow", |s| s.as_str()) // Try to get program name, default to "slideshow"
        );
        process::exit(1); // Exit with a non-zero code to indicate error
    }

    // The directory path is the second argument (index 1)
    let image_directory_path = &args[1]; // Borrow the string from the vector

    let (mut rl, thread) = raylib::init()
        .size(RENDER_WIDTH / 2, RENDER_HEIGHT / 2)
        .title("Photo Wall Slideshow")
        .vsync()
        .resizable()
        .build();
    rl.set_target_fps(FPS);

    // --- Load Slides ---
    let image_paths = match load_sorted_image_paths(image_directory_path) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error loading images from '{}': {}", image_directory_path, e);
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            d.draw_text(&format!("Error: {}", e), 20, 20, 20, Color::RED);
            drop(d);
            std::thread::sleep(Duration::from_secs(5));
            return;
        }
    };

    // keep first 5 images for testing
    // let image_paths = image_paths.into_iter().take(5).collect::<Vec<_>>();

    let mut slides: Vec<Slide> = Vec::new();
    for path in image_paths {
        match load_texture_with_exif_rotation(&mut rl, &thread, &path) {
            Ok(texture) => {
                // Now create the Slide with the loaded texture
                match Slide::new(texture) {
                    Ok(slide) => slides.push(slide),
                    Err(e) => eprintln!("Error creating slide object for {:?}: {}", path.file_name().unwrap(), e),
                }
            }
            Err(e) => {
                eprintln!("Error processing image {:?}: {}", path.file_name().unwrap(), e);
                // Optionally skip this image or handle error
            }
        }
    }

    if slides.is_empty() {
        eprintln!("No slides were created successfully.");
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_text("Error: No slides loaded.", 20, 20, 20, Color::RED);
        drop(d);
        std::thread::sleep(Duration::from_secs(5));
        return;
    }

    // Infer output video name from the directory name
    let video_name = image_directory_path
        .split('/')
        .last()
        .unwrap_or("slideshow")
        .to_string() + ".mp4";
    println!("Output video name: {}", video_name);

    // Start ffmpeg process and connect pipes so we can send rendered frames
    let mut ffmpeg_process = Command::new("ffmpeg")
        .stdin(Stdio::piped())
        .arg("-loglevel")
        .arg("verbose")
        .arg("-y")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pixel_format")
        .arg("rgba")
        .arg("-video_size")
        .arg(format!("{}x{}", RENDER_WIDTH, RENDER_HEIGHT))
        .arg("-framerate")
        .arg(format!("{}", FPS))
        .arg("-i")
        .arg("-")
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("ultrafast")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(video_name)
        .spawn()
        .expect("Failed to start ffmpeg process");
    let mut ffmpeg_stdin = ffmpeg_process.stdin.take().expect("Failed to open ffmpeg stdin");
    
    // --- Slideshow State Variables ---
    let mut current_slide_index = 0;
    let mut display_timer = 0.0;
    let mut cleanup_timer = 0.0;
    let mut cleanup_index = if slides.len() > 0 { slides.len() - 1 } else { 0 };
    let mut slideshow_state = SlideshowState::Displaying;

    let mut framebuffer = rl.load_render_texture(&thread, 1920, 1080)
        .expect("Failed to create render texture");

    // --- Main Loop ---
    while !rl.window_should_close() {
        // let dt = rl.get_frame_time(); // realtime rendering
        let dt = FRAME_TIME;
    
        // --- Update Logic ---

        // 1. Update all individual slide animations
        for slide in slides.iter_mut() {
            slide.update(dt);
        }

        // 2. Update slideshow state machine
        match slideshow_state {
            SlideshowState::Displaying => {
                display_timer += dt;
                if display_timer >= DISPLAY_DURATION {
                    // Time to start transition
                    if current_slide_index < slides.len() {
                        // Start the current slide's background animation
                        slides[current_slide_index].start_background_animation();
                        // Change state to wait for animation to finish
                        slideshow_state = SlideshowState::Transitioning;
                        // DO NOT increment index or reset timer here yet
                    } else {
                        // Should not happen if logic is correct, but safety first
                        slideshow_state = SlideshowState::Cleanup;
                        cleanup_timer = 0.0;
                    }
                }
            }
            SlideshowState::Transitioning => {
                // Check if the *current* slide (which is animating) has finished
                if current_slide_index < slides.len() && !slides[current_slide_index].is_animating {
                    // Animation finished, move to the next slide
                    current_slide_index += 1;

                    // Check if that was the last slide
                    if current_slide_index >= slides.len() {
                        slideshow_state = SlideshowState::Cleanup;
                        // Setup cleanup index (points to the last slide that animated)
                        cleanup_index = current_slide_index -1; // Correct index
                        cleanup_timer = 0.0;
                    } else {
                        // More slides remain, go back to displaying the new current slide
                        slideshow_state = SlideshowState::Displaying;
                        display_timer = 0.0; // Reset display timer for the new slide
                    }
                }
                // Else: Still animating, do nothing, stay in Transitioning state
            }
            SlideshowState::Cleanup => {
                cleanup_timer += dt;
                if cleanup_timer >= CLEANUP_INTERVAL {
                    // Hide the slide at cleanup_index
                    // Check bounds just in case cleanup_index became invalid somehow
                    if let Some(slide) = slides.get_mut(cleanup_index) {
                        slide.visible = false;
                    } else if cleanup_index == 0 && slides.len() == 1 {
                        // Handle edge case: only one slide, hide it
                        slides[0].visible = false;
                    }

                    if cleanup_index > 0 {
                        cleanup_index -= 1;
                        cleanup_timer = 0.0;
                    } else {
                        // Finished hiding slide 0
                        slideshow_state = SlideshowState::Finished;
                    }
                }
            }
            SlideshowState::Finished => {
                // Slideshow rendering is done. Close stdin pipe and wait for ffmpeg to finish
                drop(ffmpeg_stdin);
                ffmpeg_process.wait().expect("Failed to wait for ffmpeg process");
                
                break;

                // if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_R) {
                //     println!("Restarting slideshow");
                //     // Reset state variables
                //     current_slide_index = 0;
                //     display_timer = 0.0;
                //     cleanup_timer = 0.0;
                //     cleanup_index = if slides.len() > 0 { slides.len() - 1 } else { 0 };
                //     // Reset all slides
                //     for slide in slides.iter_mut() {
                //         slide.visible = true;
                //         slide.is_animating = false;
                //         slide.position = slide.initial_prominent_position;
                //         slide.scale = slide.initial_prominent_scale;
                //         slide.rotation = slide.initial_prominent_rotation;
                //         slide.animation_timer = 0.0;
                //     }
                //     slideshow_state = SlideshowState::Displaying; // Start over
                // }
            }
        }

        // --- Render each frame into fixed size "framebuffer" ---
    
        rl.draw_texture_mode(&thread, &mut framebuffer,  |mut tmd| {
            let mut d = tmd.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            // CORRECTED Drawing Logic: Draw background slides and the current active slide
            for (i, slide) in slides.iter().enumerate() {
                // Draw if it's a background slide (index less than current)
                // OR if it's the current slide being displayed or transitioning.
                // The Cleanup/Finished states are implicitly handled because
                // background slides (i < current_slide_index) will be drawn,
                // and slide.draw() checks the slide.visible flag.
                if i < current_slide_index ||
                    (i == current_slide_index && (slideshow_state == SlideshowState::Displaying || slideshow_state == SlideshowState::Transitioning))
                {
                    // The slide.draw() method internally checks for slide.visible,
                    // which handles the cleanup phase correctly.
                    slide.draw(&mut d);
                }
                // Slides with index > current_slide_index are not drawn yet.
            }
        });

        // Draw inverted copy of framebuffer to the screen for feedback

        let mut d2 = rl.begin_drawing(&thread);
        
        let sw = d2.get_screen_width() as f32;
        let sh = d2.get_screen_height() as f32;

        d2.draw_texture_pro(
            &framebuffer,
            Rectangle::new(0.0, 0.0, framebuffer.width() as f32, -(framebuffer.height() as f32)),
            Rectangle::new(0.0, 0.0, sw, sh),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE
        );

        // Grab rendered texture pixels as an Image
        let image = &framebuffer.load_image().expect("Failed to load image from framebuffer");

        unsafe {
            let image_ptr = image.data() as *const u8;
            let image_len = (image.width() * image.height() * 4) as usize; // 4 bytes per pixel (RGBA)
            let image_slice = std::slice::from_raw_parts(image_ptr, image_len);

            // Write the image data flipped vertically to ffmpeg stdin
            // This is necessary because ffmpeg expects the image data in a specific order
            // (top to bottom), while raylib provides it in bottom to top order.

            for y in 0..image.height() {
                let row_start = (image.height() - 1 - y) * image.width() * 4;
                let row_end = row_start + image.width() * 4;
                let row_slice = &image_slice[row_start as usize..row_end as usize];
                ffmpeg_stdin.write_all(row_slice).expect("Failed to write to ffmpeg stdin");
            }
        }


        // --- Optional Debug Info ---
        // let state_text = format!("State: {:?}", slideshow_state);
        // let current_text = format!("Current Idx: {}", current_slide_index);
        // let anim_text = if current_slide_index < slides.len() { format!("Animating: {}", slides[current_slide_index].is_animating) } else { "".to_string() };
        // d.draw_text(&state_text, 10, 10, 20, Color::WHITE);
        // d.draw_text(&current_text, 10, 30, 20, Color::WHITE);
        // d.draw_text(&anim_text, 10, 50, 20, Color::WHITE);
        // if slideshow_state == SlideshowState::Finished {
        //     d.draw_text("Slideshow Finished. Press 'R' to restart.", 10, 10, 20, Color::LIME);
        // }

    } // End main loop
} // End main