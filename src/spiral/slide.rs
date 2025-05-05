use raylib::prelude::*;
use crate::constants::*;

pub struct Slide {
    pub image: Texture2D,

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

    tween_position_x: Option<ease::Tween>,
    tween_position_y: Option<ease::Tween>,
    tween_scale: Option<ease::Tween>,
    tween_rotation: Option<ease::Tween>,
}

impl Slide {
    pub fn new(
        image: Texture2D, // Accept pre-loaded (and potentially rotated) texture
        initial_position: Vector2,
        initial_scale: f32,
        initial_rotation: f32   
    ) -> Result<Self, String> {
        Ok(Self {
            image, // Use the passed texture
            visible: true,

            position:       initial_position,
            scale:          initial_scale,
            rotation:       initial_rotation,

            start_position: initial_position,
            start_scale:    initial_scale,
            start_rotation: initial_rotation,
            
            end_position:   Vector2::new(0.0, 0.0),
            end_scale:      0.0,
            end_rotation:   0.0,
            
            animation_timer: 0.0,
            is_animating:    false,

            tween_position_x: None,
            tween_position_y: None,
            tween_scale:      None,
            tween_rotation:   None,
        })
    }

    pub fn set_final_position(&mut self, final_position: Vector2, final_scale: f32, final_rotation: f32) {
        self.end_position = final_position;
        self.end_scale = final_scale;
        self.end_rotation = final_rotation;
        
        self.tween_position_x = Some(ease::Tween::new(ease::cubic_out, self.start_position.x, final_position.x, ANIMATION_DURATION));
        self.tween_position_y = Some(ease::Tween::new(ease::cubic_out, self.start_position.y, final_position.y, ANIMATION_DURATION));
        self.tween_scale      = Some(ease::Tween::new(ease::back_in, self.start_scale, final_scale, ANIMATION_DURATION));
        self.tween_rotation   = Some(ease::Tween::new(ease::sine_in_out, self.start_rotation, final_rotation, ANIMATION_DURATION));
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

        self.position.x = self.tween_position_x.as_mut().expect("Tween should be initialized").apply(dt);
        self.position.y = self.tween_position_y.as_mut().expect("Tween should be initialized").apply(dt);
        self.scale      = self.tween_scale.as_mut().expect("Tween should be initialized").apply(dt);
        self.rotation   = self.tween_rotation.as_mut().expect("Tween should be initialized").apply(dt);
        
        self.animation_timer += dt;
        if self.animation_timer >= ANIMATION_DURATION {
            self.is_animating = false;
            self.position     = self.end_position;
            self.scale        = self.end_scale;
            self.rotation     = self.end_rotation;
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