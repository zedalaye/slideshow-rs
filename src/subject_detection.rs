use std::path::Path;
use anyhow::Result;
use usls::{models::YOLO, DataLoader, Options, Task, Scale, DType, Device /* */};
use raylib::prelude::*;


#[derive(Debug)]
pub struct Detection {
    pub box_: Rectangle,
    pub confidence: f32,
}

pub struct DetectionModel {
    model: YOLO,
    filter_classes: Vec<usize>,
}

impl DetectionModel {
    pub fn new(filter_classes: Vec<usize>) -> Result<Self> {
        let options = Options::yolo()
            .with_model_file("yolo/v8-head-fp16.onnx")
            .with_model_task(Task::ObjectDetection)
            .with_model_version(8.into())
            .with_model_scale(Scale::S)
            .with_model_dtype(DType::Fp16)
            .with_model_device(Device::Auto(0))
            .with_trt_fp16(true)
            .with_model_ixx(
                0,
                0,
                (1, 1, 4).into(),
            )
            .with_model_ixx(
                0,
                2,
                (224,640,1280).into(),
            )
            .with_model_ixx(
                0,
                3,
                (224,640,1280).into(),
            )
            .with_class_confs(&[0.2, 0.15])
            .with_keypoint_confs(&[0.5])
            .with_topk(5)
            .retain_classes(&filter_classes)
            .exclude_classes(&[]);
            // .with_class_names(&NAMES_COCO_80);

        let model = YOLO::try_from(options.commit()?)?;

        Ok(Self { model, filter_classes })
    }

    pub fn detect(&mut self, image_path: &Path) -> Result<Vec<Detection>> {
        let dl = DataLoader::new(image_path.to_str().unwrap())?
            .with_batch(self.model.batch() as _)
            .build()?;

        let mut detections = Vec::new();

        for xs in &dl {
            let ys = self.model.forward(&xs)?;
            println!("ys: {:?}", ys);

            for y in ys.iter() {
                if let Some(hbbs) = &y.hbbs() {
                    for hbb in hbbs.iter() {
                        let meta = hbb.meta();
                        if let Some(id) = meta.id() {
                            if self.filter_classes.contains(&id) {
                                if let Some(confidence) = meta.confidence() {        
                                    let detection = Detection {
                                        box_: Rectangle::new(hbb.x(), hbb.y(), hbb.xmax(), hbb.ymax()),
                                        confidence,
                                    };
                                    detections.push(detection);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(detections)        
    }
}
