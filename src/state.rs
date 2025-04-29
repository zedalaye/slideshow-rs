#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SlideshowState {
    Displaying,    // Showing the current slide prominently
    Transitioning, // Current slide is animating to the background
    Cleanup,       // Making background slides disappear
    Finished,      // All slides processed
}