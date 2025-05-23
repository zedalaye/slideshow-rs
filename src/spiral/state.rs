#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SpiralState {
    Displaying,    // Showing the current slide prominently
    Transitioning, // Current slide is animating to the background
    Cleanup,       // Making background slides disappear
    Finished,      // All slides processed
}