use raylib::prelude::*;
use std::path::PathBuf;

pub trait Engine {
    fn new() -> Self;
    fn initialize(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread,paths: Vec<PathBuf>) -> bool;
    fn render_frame(&mut self, dt: f32, rl: &mut RaylibHandle, thread: &RaylibThread, framebuffer: &mut RenderTexture2D) -> bool;
}
