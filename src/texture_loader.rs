use std::fs;
use std::io::Cursor;
use std::path::Path;
use raylib::prelude::*;
use exif::{Reader, Tag, Value, In};

// --- Helper: Load and Sort Image Paths ---
pub fn load_sorted_image_paths(dir_path: &str) -> Result<Vec<std::path::PathBuf>, String> {
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

// --- Load Image, Apply EXIF Rotation, Create Texture ---
pub fn load_texture_with_exif_rotation(
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
    drop(image);

    Ok(texture)
}