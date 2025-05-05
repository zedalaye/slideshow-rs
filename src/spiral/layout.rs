use raylib::prelude::*;
use crate::constants::*;
use crate::spiral::slide::Slide;
use rand::Rng;

pub struct Layout {
    pub slides: Vec<Slide>,
}

impl Layout {
    pub fn new() -> Layout {
        Layout { slides: Vec::new() }
    }

    pub fn add_image(&mut self, image: Texture2D) {

        // Scale too big images to fit the screen
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

        self.slides.push(Slide::new(
            image,
            initial_position,
            initial_scale,
            initial_rotation
        ).expect("Failed to create slide"));
    }

    pub fn compute_layout(&mut self) {
        let mut rng = rand::rng();

        // Compute grid dimensions based on images count
        // n * (n / display ratio) = images count

        let images_count = self.slides.len();
        let display_ratio = RENDER_WIDTH as f32 / RENDER_HEIGHT as f32;

        let grid_height = (images_count as f32 / display_ratio).sqrt().ceil() as i32;
        let grid_width = (images_count as f32 / grid_height as f32).ceil() as i32;

        let target_width = (RENDER_WIDTH as f32 / grid_width as f32) * 1.5;
        // println!("Target size: {}", target_width);
        
        let grid_step_x = 1.0 / grid_width as f32;
        let grid_step_y = 1.0 / grid_height as f32;
        
        let grid_offset_x = grid_step_x * 0.5;
        let grid_offset_y = grid_step_y * 0.5;

        let mut line = 0;
        let mut column = 0;
        let mut line_low_bound = 0;
        let mut line_high_bound = grid_height -1;
        let mut column_low_bound = 0;
        let mut column_high_bound = grid_width -1;

        let mut direction = 0; // 0: right, 1: down, 2: left, 3: up

        for slide in &mut self.slides {
            match direction {
                0 => {
                    // Going right, reach the end of the row
                    if column > column_high_bound {
                        direction = 1; // go down
                        line_low_bound += 1; // skip the processed line
                        line = line_low_bound;
                        column = column_high_bound;
                    }
                }
                1 => {
                    // Going down, reach the end of the column
                    if line > line_high_bound {
                        direction = 2; // go left
                        column_high_bound -= 1; // skip the processed column
                        column = column_high_bound;
                        line = line_high_bound;
                    }
                }
                2 => {
                    // Going left, reach the start of the row
                    if column < column_low_bound {
                        direction = 3; // go up
                        line_high_bound -= 1; // skip the processed line
                        line = line_high_bound;
                        column = column_low_bound;
                    }
                }
                3 => {
                    // Going up, reach the start of the column
                    if line < line_low_bound {
                        direction = 0; // go right
                        column_low_bound += 1; // skip the processed column
                        column = column_low_bound;
                        line = line_low_bound;
                    }
                }
                _ => {}
            }

            // Décalage aléatoire de +/- 1% de la position finale par rapport à la trajectoire de la spirale
            let end_pos_offset = Vector2::new(
                rng.random_range(-0.01..0.01),
                rng.random_range(-0.01..0.01)
            );

            // Calcule la position finale
            let final_position = Vector2::new(
                grid_offset_x + column as f32 * grid_step_x + end_pos_offset.x,
                grid_offset_y + line   as f32 * grid_step_y + end_pos_offset.y
            );

            // Compute final_scale so that the image is between 190px and 210px
            let image_ref_dimension = slide.image.width().max(slide.image.height());
            let final_scale = target_width / image_ref_dimension as f32 * (1.0 + rng.random_range(-0.05..0.05));
     
            let final_rotation = rng.random_range(-15.0..15.0);

            slide.set_final_position(final_position, final_scale, final_rotation);

            // Update grid position based on direction
            match direction {   
                0 => column += 1, // Right
                1 => line   += 1, // Down
                2 => column -= 1, // Left
                3 => line   -= 1, // Up
                _ => {}
            }
        }
    }
}