#[derive(Clone, PartialEq, PartialOrd)]
pub enum PushBoxState {
    Entering,
    ZoomingIn,
    Displaying,
    ZoomingOut,
    Exiting,
}