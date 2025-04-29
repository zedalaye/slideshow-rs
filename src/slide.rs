use rand::Rng;
use raylib::prelude::*;
use crate::constants::*;

pub struct Slide {
    image: Texture2D,

    pub visible: bool,

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
    pub is_animating: bool,
    
    // initial_prominent_position: Vector2,
    // initial_prominent_scale: f32,
    // initial_prominent_rotation: f32,
}

impl Slide {
    pub fn new(
        image: Texture2D, // Accept pre-loaded (and potentially rotated) texture
    ) -> Result<Self, String> {

        // --- Initial Prominent State ---
        // Calculate scale based on the *actual* texture dimensions now
        let initial_scale = if image.width() > image.height() {
            if image.width() as f32 > RENDER_WIDTH as f32 * 0.9 {
                (RENDER_WIDTH as f32 * 0.9) / image.width() as f32
            } else {
                1.0
            }
        } else {
            if image.height() as f32 > RENDER_HEIGHT as f32 * 0.9 {
                (RENDER_HEIGHT as f32 * 0.9) / image.height() as f32
            } else {
                1.0
            }
        };

        let initial_position = Vector2::new(0.5, 0.5); // Centered
        let initial_rotation = 0.0; // Rotation from EXIF is baked into the texture

        // --- Random Final Background State ---
        let mut rng = rand::rng();
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
            // initial_prominent_position: initial_position,
            // initial_prominent_scale: initial_scale,
            // initial_prominent_rotation: initial_rotation,
        })
    }

    pub fn start_background_animation(&mut self) {
        if !self.is_animating {
            self.start_position = self.position;
            self.start_scale = self.scale;
            self.start_rotation = self.rotation;
            self.animation_timer = 0.0;
            self.is_animating = true;
        }
    }

    pub fn update(&mut self, dt: f32) {
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

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        if self.visible {
            let screen_width = RENDER_WIDTH as f32;
            let screen_height = RENDER_HEIGHT as f32;

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