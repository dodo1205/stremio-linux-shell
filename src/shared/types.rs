#[derive(Debug, Clone, Copy)]
pub enum Cursor {
    Default,
    Pointer,
    Text,
    Move,
    ZoomIn,
    ZoomOut,
    Wait,
    None,
}

#[derive(Default, Clone, Copy)]
pub struct MousePosition(pub i32, pub i32);

#[derive(Default, Clone)]
pub struct MouseDelta(pub i32, pub i32);

#[derive(Debug, Clone, Copy)]
pub struct WindowSize(pub i32, pub i32);
