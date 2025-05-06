#[derive(Clone, PartialEq)]
pub enum PushBoxState {
    Entering,
    ZoomingIn,
    Displaying,
    ZoomingOut,
    Exiting,
}