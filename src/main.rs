use std::path::Path;
use std::time::Duration;
use raylib::prelude::*;
use clap::Parser;

mod constants;
mod texture_loader;
mod ffmpeg;
mod engine;
mod spiral;

use crate::constants::*;
use crate::texture_loader::*;
use crate::ffmpeg::*;
use crate::engine::Engine;
use crate::spiral::engine::SpiralEngine;

fn display_error(rl: &mut RaylibHandle, thread: &RaylibThread, error: &str) {
    eprintln!("{}", error);
    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::BLACK);
    d.draw_text(error, 20, 20, 20, Color::RED);
    drop(d);
    std::thread::sleep(Duration::from_secs(5));
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct App {
    #[arg(short, long, help = "Engine to use (spiral or push-box)")]
    engine: String,
    
    #[arg(short, long, help = "Directory containing images")]
    directory: String,
}

fn main() {
    // --- Get Directory from Command Line ---
    let args = App::parse();

    // The directory path is the second argument (index 1)
    let image_directory_path = Path::new(&args.directory); // Borrow the string from the vector
    let video_name = image_directory_path.file_name().unwrap().to_str().unwrap().to_string() + ".mp4";
    println!("Input path: {}\nOutput video name: {}", image_directory_path.to_str().unwrap(), video_name);

    let (mut rl, thread) = raylib::init()
        .size(RENDER_WIDTH / 2, RENDER_HEIGHT / 2)
        .title("Photo Wall Slideshow")
        .vsync()
        .resizable()
        .build();
    rl.set_target_fps(FPS);
    rl.set_trace_log(TraceLogLevel::LOG_ERROR);

    let mut framebuffer = rl.load_render_texture(&thread, RENDER_WIDTH as u32, RENDER_HEIGHT as u32)
        .expect("Failed to create render frame buffer");

    // --- Load Slides ---
    let image_paths = match load_sorted_image_paths(image_directory_path.to_str().unwrap()) {
        Ok(paths) => paths,
        Err(e) => {
            display_error(&mut rl, &thread, &format!("Error loading images from '{}': {}", image_directory_path.to_str().unwrap(), e));
            return;
        }
    };

    // keep first 5 images for testing
    // let image_paths = image_paths.into_iter().take(5).collect::<Vec<_>>();

    let mut engine = SpiralEngine::new();
    
    if !engine.initialize(&mut rl, &thread, image_paths) {
        display_error(&mut rl, &thread, "No slides were created successfully.");
        return;
    }

    let mut ffmpeg = Ffmpeg::new(RENDER_WIDTH, RENDER_HEIGHT, FPS, &video_name);

    // --- Main Loop ---
    while !rl.window_should_close() {
        // let dt = rl.get_frame_time(); // realtime rendering
        let dt = FRAME_TIME;

        if !engine.render_frame(dt, &mut rl, &thread, &mut framebuffer) {
            drop(ffmpeg);
            break;
        }

        // Draw inverted copy of framebuffer to the screen for feedback

        let mut d = rl.begin_drawing(&thread);
        
        let sw = d.get_screen_width() as f32;
        let sh = d.get_screen_height() as f32;

        d.draw_texture_pro(
            &framebuffer,
            Rectangle::new(0.0, 0.0, framebuffer.width() as f32, -(framebuffer.height() as f32)),
            Rectangle::new(0.0, 0.0, sw, sh),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE
        );

        // Grab rendered texture pixels as an Image
        let image = &framebuffer.load_image().expect("Failed to load image from framebuffer");

        // Write the image to the ffmpeg pipe
        ffmpeg.write(&image);
    } // End main loop
} // End main