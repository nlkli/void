use crate::camera::Camera;

pub enum ViewCommand {
    None,
}

pub struct View {
    camera: Camera,
}

impl View {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
        }
    }
}
