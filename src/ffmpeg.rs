use std::process::{Command, Stdio};
use std::io::Write;
use raylib::prelude::*;

pub struct Ffmpeg {
    process: std::process::Child,
    stdin: Option<std::process::ChildStdin>,   
}

impl Ffmpeg {
    pub fn new(width: i32, height: i32, fps: u32, video_name: &str) -> Ffmpeg {
        let mut process = Command::new("ffmpeg")
            .stdin(Stdio::piped())
            .args(["-loglevel", "verbose"])
            .arg("-y")
            .args(["-f", "rawvideo"])
            .args(["-pixel_format", "rgba"])
            .args(["-video_size", &format!("{}x{}", width, height)])
            .args(["-framerate", &format!("{}", fps)])
            .args(["-i", "-"])
            .args(["-c:v", "libx264"])
            // .args(["-preset", "ultrafast"])
            .args(["-pix_fmt", "yuv420p"])
            .arg(video_name)
            .spawn()
            .expect("Failed to start ffmpeg process");
        let stdin = process.stdin.take().expect("Failed to open ffmpeg stdin");
        Ffmpeg { process, stdin: Some(stdin) }
    }

    pub fn write(&mut self, image: &Image) {
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
                self.stdin.as_mut().unwrap().write_all(row_slice).expect("Failed to write to ffmpeg stdin");
            }
        }
    }
}

impl Drop for Ffmpeg {
    fn drop(&mut self) {
        // Close stdin pipe and wait for ffmpeg to finish
        self.stdin = None; // force drop
        self.process.wait().expect("Failed to wait for ffmpeg process");
    }
}