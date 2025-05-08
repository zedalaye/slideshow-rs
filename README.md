# The PhotoWall Slideshow Renderer in Rust

```bash
$ cargo run -e <ENGINE> -d <DIRECTORY>
```

The rendered slideshow is a 1920x1080 @60fps MP4 video named after the directory name.

```bash
$ cargo run -e spiral -d /home/pierre/Images/Family
```

Will produce a `Family.mp4` file in the current directory.

### Engines

* `spiral` : Renders a Photowall by dispatching photos randomly rotated and scaled in a spiral pattern with a nice "cleanup" effect at the end.
* `push-box` : Renders a "Push Box" slideshow where photos come from left to right and are displayed full screen with a slight "Ken Burns" (Zoom & Pan) effect towards people heads.

Photos are loaded from directory and sorted by name. EXIF data (when available) is used to rotate the photos to their correct orientation.

### Dependencies

* Rust 1.86.0
* FFMPEG
* Raylib 5.5.x will be installed and compiled by [crates.io/raylib](https://crates.io/crates/raylib)
* USLS dependencies (https://github.com/jamjamjon/usls)
* The YOLO v8 "head" model from [Jamjamjon](https://github.com/jamjamjon)
  * Direct Download :https://github.com/jamjamjon/assets/releases/download/yolo/v8-head-fp16.onnx
  * More information : https://github.com/jamjamjon/usls/tree/main/examples/yolo

Bonus if you want to play with the `yolo` CLI :
* ONNX Runtime (`pip install onnx onnxslim onnxruntime-gpu`) to run ONNX models
* Ultralytics (`pip install ultralytics`) to download YOLO models and export them to ONNX format (https://docs.ultralytics.com/usage/export/)

### Notes

Earlier versions of this project used `OpenCV` (and the Rust crate `opencv`) with `YOLO` for subject detection but I never got it to work so I switched to `USLS` after I found an [example](https://github.com/ultralytics/ultralytics/tree/main/examples/YOLO-Series-ONNXRuntime-Rust) in the Ultralytics repository on how to use the `YOLO` models with Rust.

For those interrested, [this is how to install](https://gist.github.com/andmax/7cbff81af9b685a83f74faa476f33699) the latest version of OpenCV on Ubuntu.

### Notice

Largely assisted with various AIs : Gemini Code Assist in RustRover for the very first version then improvements were made using Winsurf AI with Claude and Grok 3.

(I had to fix so many issues with the code...) 

For instance, all AIs are outdated regarding versions of compilers and dependencies : Gemini insisted on using deprecated `rand` methods and never accepted that `kamadak-exif` is now available within the `Exif` module.