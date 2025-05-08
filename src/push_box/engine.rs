use std::path::PathBuf;
use raylib::prelude::*;
use crate::load_texture_with_exif_rotation;
use crate::push_box::slide::Slide;
use crate::push_box::state::PushBoxState;
use crate::subject_detection::DetectionModel;

pub struct PushBoxEngine {
    slides: Vec<Slide>,
    current_slide_index: usize,
}

impl crate::engine::Engine for PushBoxEngine {
    fn new() -> Self {
        Self {
            slides: Vec::new(),
            current_slide_index: 0,
        }
    }

    fn initialize(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, paths: Vec<PathBuf>) -> bool {
        let mut detection_model = DetectionModel::new(vec![0]).unwrap();

        for path in paths {
            match load_texture_with_exif_rotation(rl, thread, &path) {
                Ok(image) => {
                    let detections = detection_model.detect(&path).unwrap();
                   
                    println!("{}: {:?}", path.to_str().unwrap(), detections);

                    let merged_box = if !detections.is_empty() {
                        // merge all boxes into one raylib Rectangle
                        let mut merged_box = detections[0].box_.clone();

                        // Returned width and heights are absolute values
                        for detection in detections.iter() {
                            if detection.confidence >= 0.8 {
                                merged_box.x      = merged_box.x.min(detection.box_.x as f32);
                                merged_box.y      = merged_box.y.min(detection.box_.y as f32);
                                merged_box.width  = merged_box.width.max(detection.box_.width as f32);
                                merged_box.height = merged_box.height.max(detection.box_.height as f32);
                            }
                        }

                        // Convert absolute width and height to relative values
                        merged_box.width = merged_box.width - merged_box.x;
                        merged_box.height = merged_box.height - merged_box.y;
                        
                        println!("Merged Box: {:?}", merged_box);
                        merged_box
                    } else {
                        Rectangle::new(0.0, 0.0, 0.0, 0.0)
                    };

                    /* DEBUG ! Draws the image with a red rectangle around the merged box
                       We have to draw the image twice because "Texture Mode" in Raylib produces
                       inverted images (flipped horizontally) */
                    
                    // let mut tmp_texture = rl.load_render_texture(&thread, 
                    //     image.width() as u32, image.height() as u32
                    // ).expect("Failed to create render frame buffer");

                    // rl.draw_texture_mode(thread, &mut tmp_texture, |mut tmd| {
                    //     let mut d = tmd.begin_drawing(thread);
                    //     d.draw_texture_pro(
                    //         &image,
                    //         Rectangle::new(0.0, 0.0, image.width() as f32, image.height() as f32),
                    //         Rectangle::new(0.0, 0.0, image.width() as f32, image.height() as f32),
                    //         Vector2::new(0.0, 0.0),
                    //         0.0,
                    //         Color::WHITE
                    //     );
                    //     d.draw_rectangle_lines_ex(merged_box, 8.0, Color::RED);
                    // }); 

                    // let mut inv_texture = rl.load_render_texture(&thread, 
                    //     tmp_texture.width() as u32, tmp_texture.height() as u32
                    // ).expect("Failed to create render frame buffer");

                    // rl.draw_texture_mode(thread, &mut inv_texture, |mut tmd| {
                    //     let mut d = tmd.begin_drawing(thread);
                    //     d.draw_texture_rec(&tmp_texture, 
                    //         Rectangle::new(0.0, 0.0, tmp_texture.width() as f32, tmp_texture.height() as f32),
                    //         Vector2::new(0.0, 0.0), 
                    //         Color::WHITE
                    //     );
                    // });

                    /* I have not found a more direct way to use the rendered texture */
                    
                    // let tmp_image = inv_texture.load_image()
                    //     .expect("Failed to load image from render texture");
                    // let tmp_texture = rl.load_texture_from_image(&thread, &tmp_image)
                    //     .expect("Failed to create texture from image");

                    self.slides.push(Slide::new(image, merged_box));
                }
                Err(e) => {
                    println!("Failed to load image: {}", e);
                }
            }
        }

        // If there are slides, start the first one
        if !self.slides.is_empty() {
            self.slides[0].visible = true;
            self.slides[0].is_animating = true;
            return true;
        }

        false
    }

    fn render_frame(&mut self, dt: f32, rl: &mut RaylibHandle, thread: &RaylibThread, framebuffer: &mut RenderTexture2D) -> bool {      

        // Iterate only current, previous and next slides
        let mut slides_to_update = vec![self.current_slide_index];
        if self.current_slide_index > 0 {
            slides_to_update.push(self.current_slide_index - 1);
        }
        if self.current_slide_index < self.slides.len() - 1 {
            slides_to_update.push(self.current_slide_index + 1);
        }

        let mut is_animating = false;
        let mut animate_next_slide = false;

        for slide_index in slides_to_update.iter() {
            let slide = &mut self.slides[*slide_index];

            let slide_state_before = slide.state.clone();
            slide.update(dt);
            let slide_state_after = slide.state.clone();

            // If current slide is exiting, move to next slide
            if *slide_index == self.current_slide_index && slide_state_before == PushBoxState::ZoomingOut && slide_state_after == PushBoxState::Exiting {
                animate_next_slide = true;
                is_animating = true;
            }    

            is_animating = slide.is_animating || is_animating;
        }

        if !is_animating {
            return false;
        }

        // Activate next slide
        if animate_next_slide && self.current_slide_index < self.slides.len() - 1 {
            self.current_slide_index += 1;
            self.slides[self.current_slide_index].visible = true;
            self.slides[self.current_slide_index].is_animating = true;
        }

        rl.draw_texture_mode(thread, framebuffer,  |mut tmd| {
            let mut d = tmd.begin_drawing(thread);
            d.clear_background(Color::BLACK);
            
            for slide in self.slides.iter() {
                if slide.visible {
                    slide.draw(&mut d);
                }
            }
        });   

        true
    }
}
