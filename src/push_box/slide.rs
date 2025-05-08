use raylib::prelude::*;
use crate::constants::*;
use crate::push_box::state::PushBoxState;

pub struct Slide {
    pub image: Texture2D,

    pub visible: bool,
    pub state: PushBoxState,

    pub is_animating: bool,
    animation_timer: f32,

    initial_scale: f32, // how the image appears from the left
    final_scale: f32,   // scale factor to fit the screen

    // Current values during animation, those are computed by Tweens below
    position: Vector2,
    scale: f32,
    
    tween_entering: ease::Tween,
    tween_zooming_in: ease::Tween,
    tween_zooming_out: ease::Tween,
    tween_exiting: ease::Tween,
    
    // Ken Burns effect parameters for Displaying state, those are computed by Tweens below
    ken_burns_scale: f32,
    ken_burns_pan: Vector2,

    tween_ken_burns_scale: ease::Tween,
    tween_ken_burns_pan_x: ease::Tween,
    tween_ken_burns_pan_y: ease::Tween,
}

impl Slide {
    pub fn new(image: Texture2D, subject_rect: Rectangle) -> Self {
        // Scale images too big to fit the screen
        let final_scale = if image.width() > image.height() {
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

        // Initial scale is half of final scale
        let initial_scale = final_scale * 0.5;

        /* Ken Burns effect parameters */

        // If no subject rect, use the whole image
        let subject_rect = if subject_rect.width == 0.0 || subject_rect.height == 0.0 {
            Rectangle::new(0.0, 0.0, image.width() as f32, image.height() as f32)
        } else {
            subject_rect
        };  

        // Zoom-in to subject rect
        let subject_size = subject_rect.width.max(subject_rect.height) as f32;
        let image_size   = image.width().max(image.height()) as f32;
        let base_scale   = subject_size / image_size;

        // Base scale is clamped between 0.7 and 1.0
        let ken_burns_scale = (base_scale * 0.5 + 0.5).clamp(0.7, 1.0);

        // Calculate the center of the bounding rectangle
        let subject_center = Vector2::new(
            subject_rect.x + subject_rect.width / 2.0,
            subject_rect.y + subject_rect.height / 2.0,
        );

        // Define the final position of the move (in pixels, relative to the image)
        let ken_burns_end_pos = Vector2::new(
            subject_center.x - (image.width() as f32 / 2.0),
            subject_center.y - (image.height() as f32 / 2.0),
        );

        // println!("ken_burns_scale: {}", ken_burns_scale);
        // println!("ken_burns_end_pos: ({}, {})", ken_burns_end_pos.x, ken_burns_end_pos.y);

        Self {
            image,

            visible: false,
            state: PushBoxState::Entering,

            is_animating: false,
            animation_timer: 0.0,

            initial_scale,
            final_scale,

            // Initial position is outside the left of the screen
            position: Vector2::new(-0.5, 0.5),            
            scale: initial_scale,

            tween_entering:    ease::Tween::new(ease::cubic_out, -0.5, 0.5, ANIMATION_DURATION),
            tween_zooming_in:  ease::Tween::new(ease::cubic_out, initial_scale, final_scale, ANIMATION_DURATION),
            tween_zooming_out: ease::Tween::new(ease::cubic_out, final_scale, initial_scale, ANIMATION_DURATION),
            tween_exiting:     ease::Tween::new(ease::cubic_out, 0.5, 1.5, ANIMATION_DURATION),
            
            // Ken Burns effect initialization
            ken_burns_scale: 1.0,
            ken_burns_pan: Vector2::new(0.0, 0.0),
            
            tween_ken_burns_scale: ease::Tween::new(ease::linear_none, 1.0, ken_burns_scale, DISPLAY_DURATION),
            tween_ken_burns_pan_x: ease::Tween::new(ease::linear_none, 0.0, ken_burns_end_pos.x, DISPLAY_DURATION),
            tween_ken_burns_pan_y: ease::Tween::new(ease::linear_none, 0.0, ken_burns_end_pos.y, DISPLAY_DURATION),           
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_animating {
            return;
        }

        match self.state {
            PushBoxState::Entering => {
                self.scale = self.initial_scale;
                self.position.x = self.tween_entering.apply(dt);
            }
            PushBoxState::ZoomingIn => {
                self.position = Vector2::new(0.5, 0.5);
                self.scale = self.tween_zooming_in.apply(dt);
            }
            PushBoxState::Displaying => {
                self.position = Vector2::new(0.5, 0.5);
                self.scale = self.final_scale;

                // Animate Ken Burns effect
                self.ken_burns_scale = self.tween_ken_burns_scale.apply(dt);
                self.ken_burns_pan.x = self.tween_ken_burns_pan_x.apply(dt);
                self.ken_burns_pan.y = self.tween_ken_burns_pan_y.apply(dt);
            }
            PushBoxState::ZoomingOut => {
                self.position = Vector2::new(0.5, 0.5);
                self.scale = self.tween_zooming_out.apply(dt);
            }
            PushBoxState::Exiting => {
                self.scale = self.initial_scale;
                self.position.x = self.tween_exiting.apply(dt);
            }
        }

        self.animation_timer += dt;
        let expected_duration = match self.state {
            PushBoxState::Displaying => DISPLAY_DURATION,
            _ => ANIMATION_DURATION,
        };

        if self.animation_timer >= expected_duration {
            self.animation_timer = 0.0;
            match self.state {
                PushBoxState::Entering   => self.state = PushBoxState::ZoomingIn,
                PushBoxState::ZoomingIn  => self.state = PushBoxState::Displaying,
                PushBoxState::Displaying => self.state = PushBoxState::ZoomingOut,
                PushBoxState::ZoomingOut => self.state = PushBoxState::Exiting,
                PushBoxState::Exiting    => { 
                    self.is_animating = false; 
                    self.visible = false; 
                },
            }
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

            // Relative to the dest rectangle (ie. the center of the image)
            let origin = Vector2::new(scaled_width * 0.5, scaled_height * 0.5);

            // Adjust source rectangle for Ken Burns effect during Displaying state
            let source_rec = if self.state >= PushBoxState::Displaying {                                             
                let scaled_ken_burns_width = tex_width * self.ken_burns_scale;
                let scaled_ken_burns_height = tex_height * self.ken_burns_scale;

                let pan_origin = Vector2::new(
                    (tex_width - scaled_ken_burns_width) * 0.5,
                    (tex_height - scaled_ken_burns_height) * 0.5
                );

                Rectangle::new(pan_origin.x + self.ken_burns_pan.x, pan_origin.y + self.ken_burns_pan.y, 
                    scaled_ken_burns_width, scaled_ken_burns_height
                )
            } else {
                Rectangle::new(0.0, 0.0, tex_width, tex_height)
            };

            d.draw_texture_pro(
                &self.image,
                source_rec,
                Rectangle::new(draw_pos.x + origin.x, draw_pos.y + origin.y, scaled_width, scaled_height),
                origin,
                0.0,
                Color::WHITE,
            );
        }
    }
}