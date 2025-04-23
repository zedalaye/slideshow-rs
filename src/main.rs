use raylib::prelude::*;
use rand::Rng;
use std::env;
use std::fs;
use std::io::Cursor; // Needed for reading EXIF from memory buffer
use std::path::Path;
use std::process;
use std::time::Duration;

// --- EXIF Reading ---
// Use kamadak_exif for reading orientation
use exif::{Reader, Tag, Value, In};

// --- Constants ---
// MODIFIED: Faster animation
const ANIMATION_DURATION: f32 = 0.5; // Duration for background animation (seconds)
const DISPLAY_DURATION: f32 = 2.0;   // Duration each slide is shown prominently (seconds)
const CLEANUP_INTERVAL: f32 = 0.2;   // Time between background slides disappearing (seconds)

// --- Slideshow State ---
// MODIFIED: Added Transitioning state
#[derive(Debug, PartialEq, Clone, Copy)] // Added derive for Debug/Copy
enum SlideshowState {
    Displaying,    // Showing the current slide prominently
    Transitioning, // Current slide is animating to the background
    Cleanup,       // Making background slides disappear
    Finished,      // All slides processed
}

// --- Slide Struct (Modified) ---
// No changes needed here structurally, but `new` will change
struct Slide {
    image: Texture2D,
    visible: bool,
    position: Vector2,
    scale: f32,
    rotation: f32,
    start_position: Vector2,
    start_scale: f32,
    start_rotation: f32,
    end_position: Vector2,
    end_scale: f32,
    end_rotation: f32,
    animation_timer: f32,
    is_animating: bool,
    initial_prominent_position: Vector2,
    initial_prominent_scale: f32,
    initial_prominent_rotation: f32,
}

impl Slide {
    // MODIFIED: Accepts Texture2D directly, doesn't load from path anymore
    pub fn new(
        image: Texture2D, // Accept pre-loaded (and potentially rotated) texture
        screen_width: f32,
        screen_height: f32,
    ) -> Result<Self, String> {

        // --- Initial Prominent State ---
        // Calculate scale based on the *actual* texture dimensions now
        let initial_scale = if image.width() > image.height() {
            if image.width() as f32 > screen_width * 0.9 {
                (screen_width * 0.9) / image.width() as f32
            } else {
                1.0
            }
        } else {
            if image.height() as f32 > screen_height * 0.9 {
                (screen_height * 0.9) / image.height() as f32
            } else {
                1.0
            }
        };

        let initial_position = Vector2::new(0.5, 0.5); // Centered
        let initial_rotation = 0.0; // Rotation from EXIF is baked into the texture

        // --- Random Final Background State ---
        let mut rng = rand::rng(); // Correct way to get thread_rng
        let final_position = Vector2::new(
            rng.random_range(0.05..0.95),
            rng.random_range(0.05..0.95),
        );
        let final_scale = initial_scale * rng.random_range(0.20..0.30);
        let final_rotation = rng.random_range(-15.0..15.0);

        Ok(Self {
            image, // Use the passed texture
            visible: true,
            position: initial_position,
            scale: initial_scale,
            rotation: initial_rotation,
            start_position: initial_position,
            start_scale: initial_scale,
            start_rotation: initial_rotation,
            end_position: final_position,
            end_scale: final_scale,
            end_rotation: final_rotation,
            animation_timer: 0.0,
            is_animating: false,
            initial_prominent_position: initial_position,
            initial_prominent_scale: initial_scale,
            initial_prominent_rotation: initial_rotation,
        })
    }

    // start_background_animation remains the same
    fn start_background_animation(&mut self) {
        if !self.is_animating {
            self.start_position = self.position;
            self.start_scale = self.scale;
            self.start_rotation = self.rotation;
            self.animation_timer = 0.0;
            self.is_animating = true;
        }
    }

    // update remains the same
    fn update(&mut self, dt: f32) {
        if !self.is_animating {
            return;
        }
        self.animation_timer += dt;
        let t = (self.animation_timer / ANIMATION_DURATION).min(1.0);
        // Optional Easing:
        // let t = 1.0 - (1.0 - t).powi(3); // easeOutCubic

        self.position = self.start_position.lerp(self.end_position, t);
        self.scale = raylib::core::math::lerp(self.start_scale, self.end_scale, t);
        self.rotation = raylib::core::math::lerp(self.start_rotation, self.end_rotation, t);

        if self.animation_timer >= ANIMATION_DURATION {
            self.is_animating = false;
            self.position = self.end_position;
            self.scale = self.end_scale;
            self.rotation = self.end_rotation;
        }
    }

    // draw remains the same
    fn draw(&self, d: &mut RaylibDrawHandle) {
        if self.visible {
            let screen_width = d.get_screen_width() as f32;
            let screen_height = d.get_screen_height() as f32;
            // Use texture dimensions directly
            let tex_width = self.image.width() as f32;
            let tex_height = self.image.height() as f32;

            let scaled_width = tex_width * self.scale;
            let scaled_height = tex_height * self.scale;

            let draw_pos = Vector2::new(
                screen_width * self.position.x - scaled_width * 0.5,
                screen_height * self.position.y - scaled_height * 0.5,
            );

            let origin = Vector2::new(scaled_width / 2.0, scaled_height / 2.0);

            d.draw_texture_pro(
                &self.image,
                Rectangle::new(0.0, 0.0, tex_width, tex_height), // Source rect uses original texture size
                Rectangle::new(draw_pos.x + origin.x, draw_pos.y + origin.y, scaled_width, scaled_height), // Dest rect uses scaled size
                origin,
                self.rotation,
                Color::WHITE,
            );
        }
    }
}

// --- Helper: Load and Sort Image Paths (remains the same) ---
fn load_sorted_image_paths(dir_path: &str) -> Result<Vec<std::path::PathBuf>, String> {
    // ... (implementation is the same as before) ...
    let mut paths = Vec::new();
    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory {}: {}", dir_path, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                match ext.to_lowercase().as_str() {
                    "png" | "jpg" | "jpeg" | "bmp" | "gif" => {
                        paths.push(path);
                    }
                    _ => {}
                }
            }
        }
    }
    paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    if paths.is_empty() {
        Err(format!("No image files found in directory: {}", dir_path))
    } else {
        Ok(paths)
    }
}

// --- NEW Helper: Load Image, Apply EXIF Rotation, Create Texture ---
fn load_texture_with_exif_rotation(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    image_path: &Path,
) -> Result<Texture2D, String> {
    let file_bytes = fs::read(image_path)
        .map_err(|e| format!("Failed to read file {:?}: {}", image_path, e))?;

    let mut orientation = 1; // Default: no rotation

    // Attempt to read EXIF data (only works reliably for JPEG)
    let extension = image_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    if extension == "jpg" || extension == "jpeg" {
        match Reader::new().read_from_container(&mut Cursor::new(&file_bytes)) {
            Ok(exif) => {
                if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
                    if let Value::Short(values) = &field.value {
                        if !values.is_empty() {
                            orientation = values[0];
                            // println!("Image {:?} EXIF Orientation: {}", image_path.file_name().unwrap(), orientation); // Debug
                        }
                    }
                }
            }
            Err(e) => {
                // Log non-critical error: EXIF reading failed, proceed without rotation
                eprintln!("Warning: Could not read EXIF data for {:?}: {}", image_path.file_name().unwrap_or_else(|| image_path.as_os_str()), e);
            }
        }
    }


    // Load image data into memory (Image struct)
    // Provide extension hint for loading from memory
    let mut image = Image::load_image_from_mem(&(".".to_string() + &extension), &file_bytes)
        .map_err(|e| format!("Failed to load image data for {:?}: {}", image_path, e))?; // Use map_err for RaylibError

    // Apply rotation based on orientation value
    // 1 = Top-left (Normal)
    // 3 = Bottom-right (180 deg)
    // 6 = Top-right (90 deg clockwise)
    // 8 = Bottom-left (270 deg clockwise / 90 deg counter-clockwise)
    // Others involve flips, ignored for simplicity here.
    match orientation {
        3 => {
            image.rotate_cw();
            image.rotate_cw(); // 180 deg
            println!("Applied 180 deg rotation"); // Debug
        }
        6 => {
            image.rotate_cw(); // 90 deg clockwise
            println!("Applied 90 deg CW rotation"); // Debug
        }
        8 => {
            image.rotate_ccw(); // 90 deg counter-clockwise
            println!("Applied 90 deg CCW rotation"); // Debug
        }
        _ => { /* No rotation needed for 1 or others */ }
    }

    // Create Texture2D from the potentially rotated Image data
    let texture = rl.load_texture_from_image(thread, &image)
        .map_err(|e| format!("Failed to create texture for {:?}: {}", image_path, e))?; // Use map_err

    // Unload the Image data from CPU memory (important!)
    drop(image); // rl.unload_image(image);

    Ok(texture)
}


// --- Main Function ---
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
        .size(1920 / 2, 1080 / 2)
        .title("Photo Wall Slideshow")
        .vsync()
        .resizable()
        .build();
    rl.set_target_fps(60);

    // --- Load Slides ---
    let image_paths = match load_sorted_image_paths(image_directory_path) {
        Ok(paths) => paths,
        Err(e) => {
            // Handle error (same as before)
            eprintln!("Error loading images from '{}': {}", image_directory_path, e);
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            d.draw_text(&format!("Error: {}", e), 20, 20, 20, Color::RED);
            drop(d);
            std::thread::sleep(Duration::from_secs(5));
            return;
        }
    };

    let mut slides: Vec<Slide> = Vec::new();
    for path in image_paths.iter() {
        let sw = rl.get_screen_width() as f32;
        let sh = rl.get_screen_height() as f32;

        // MODIFIED: Use the new loading function
        match load_texture_with_exif_rotation(&mut rl, &thread, path) {
            Ok(texture) => {
                // Now create the Slide with the loaded texture
                match Slide::new(texture, sw, sh) {
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
        // Handle error (same as before)
        eprintln!("No slides were created successfully.");
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_text("Error: No slides loaded.", 20, 20, 20, Color::RED);
        drop(d);
        std::thread::sleep(Duration::from_secs(5));
        return;
    }


    // --- Slideshow State Variables ---
    let mut current_slide_index = 0;
    let mut display_timer = 0.0;
    let mut cleanup_timer = 0.0;
    let mut cleanup_index = if slides.len() > 0 { slides.len() - 1 } else { 0 };
    let mut slideshow_state = SlideshowState::Displaying;

    // --- Main Loop ---
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // --- Update Logic ---

        // 1. Update all individual slide animations
        for slide in slides.iter_mut() {
            slide.update(dt);
        }

        // 2. Update slideshow state machine (MODIFIED)
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
                if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_R) {
                    println!("Restarting slideshow");
                    // Reset state variables
                    current_slide_index = 0;
                    display_timer = 0.0;
                    cleanup_timer = 0.0;
                    cleanup_index = if slides.len() > 0 { slides.len() - 1 } else { 0 };
                    // Reset all slides
                    for slide in slides.iter_mut() {
                        slide.visible = true;
                        slide.is_animating = false;
                        slide.position = slide.initial_prominent_position;
                        slide.scale = slide.initial_prominent_scale;
                        slide.rotation = slide.initial_prominent_rotation;
                        slide.animation_timer = 0.0;
                    }
                    slideshow_state = SlideshowState::Displaying; // Start over
                }
            }
        }

        // --- Drawing ---
        let mut d = rl.begin_drawing(&thread);
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

        // --- Optional Debug Info ---
        // let state_text = format!("State: {:?}", slideshow_state);
        // let current_text = format!("Current Idx: {}", current_slide_index);
        // let anim_text = if current_slide_index < slides.len() { format!("Animating: {}", slides[current_slide_index].is_animating) } else { "".to_string() };
        // d.draw_text(&state_text, 10, 10, 20, Color::WHITE);
        // d.draw_text(&current_text, 10, 30, 20, Color::WHITE);
        // d.draw_text(&anim_text, 10, 50, 20, Color::WHITE);
        if slideshow_state == SlideshowState::Finished {
            d.draw_text("Slideshow Finished. Press 'R' to restart.", 10, 10, 20, Color::LIME);
        }

    } // End main loop
} // End main