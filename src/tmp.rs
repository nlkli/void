use std::sync::Arc;

use crate::{camera::Camera, element::Element};
use vello as V;
use vello::kurbo as K;
use winit::{
    dpi::PhysicalPosition,
    event as WE, keyboard as WK,
    window::{CursorIcon, Window},
};

#[derive(Debug, Clone, Copy)]
struct MouseState {
    cursor_pos: K::Point,
    // cursor_delta: K::Vec2,
    // [0] left; [1] right
    pressed: [bool; 3],
    // pressed_pos: [K::Point; 3],
    // ptime: [std::time::Instant; 3],
    // ptime_prev: [std::time::Instant; 3],
    // pressed_prev: [bool; 3],
}

impl Default for MouseState {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId(u64);

// pub struct Document {
//     elements: Vec<(ElementId, Element)>,
//     id_acc: u64,
// }

// #[derive(Debug, Clone, Copy)]
// pub enum Command {
//     None,
//     SetTool(Tool),
//     MoveCamera(K::Vec2),
// }

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Selection,
    Hand,
    // Rectangle,
}

#[derive(Debug, Clone, Default)]
pub struct Editor {
    elements: Vec<(ElementId, Element)>,
    id_acc: u64,

    tool: Tool,
    mouse: MouseState,
    camera: Vec<Camera>,
    camera_idx: usize,
}

impl Editor {
    pub fn new() -> Self {
        let mut editor = Self::default();
        editor.camera.push(Camera::default());
        editor
    }

    pub fn dispatch_window_event(&mut self, event: WE::WindowEvent, window: &mut Arc<Window>) {
        match event {
            WE::WindowEvent::KeyboardInput {
                event,
                // is_synthetic,
                ..
            } => self.keyboard_input_event(event, window),
            WE::WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_input_event(state, button, window);
            }
            WE::WindowEvent::CursorMoved { position, .. } => {
                self.mouse_cursor_moved_event(position, window);
            }
            _ => {}
        }
    }

    fn set_cursor_by_tool(&mut self, window: &mut Arc<Window>) {
        match self.tool {
            Tool::Selection => window.set_cursor(CursorIcon::Default),
            Tool::Hand => window.set_cursor(CursorIcon::Grab),
            _ => {}
        }
    }

    fn change_tool(&mut self, tool: Tool, window: &mut Arc<Window>) {
        if self.tool == tool {
            return;
        }
        self.tool = tool;
        self.set_cursor_by_tool(window);
    }

    #[inline]
    fn keyboard_input_event(&mut self, event: WE::KeyEvent, window: &mut Arc<Window>) {
        match event.physical_key {
            WK::PhysicalKey::Code(key_code) => match key_code {
                WK::KeyCode::KeyH => {
                    self.change_tool(Tool::Hand, window);
                }
                WK::KeyCode::KeyV => {
                    self.change_tool(Tool::Selection, window);
                }
                _ => {}
            },
            WK::PhysicalKey::Unidentified(_native_key_code) => {}
        }
    }

    #[inline]
    fn mouse_input_event(
        &mut self,
        state: WE::ElementState,
        btn: WE::MouseButton,
        window: &mut Arc<Window>,
    ) {
        let mut nbtn = match btn {
            WE::MouseButton::Left => {
                if self.tool == Tool::Hand {
                    if state == WE::ElementState::Pressed {
                        window.set_cursor(CursorIcon::Grabbing)
                    } else {
                        self.set_cursor_by_tool(window);
                    }
                }
                1
            }
            WE::MouseButton::Right => 2,
            WE::MouseButton::Middle => {
                if state == WE::ElementState::Pressed {
                    window.set_cursor(CursorIcon::Grabbing)
                } else {
                    self.set_cursor_by_tool(window);
                }
                3
            }
            _ => 0,
        };

        if nbtn > 0 {
            nbtn -= 1;
            // self.mouse.pressed_prev[nbt] = self.mouse.pressed[nbt];
            self.mouse.pressed[nbtn] = state == WE::ElementState::Pressed;
            // self.mouse.pressed_pos[nbt] = self.mouse.cursor_pos;
            // self.mouse.ptime_prev[nbt] = self.mouse.ptime[nbt];
            // self.mouse.ptime[nbt] = std::time::Instant::now();
        }
    }

    #[inline]
    fn mouse_cursor_moved_event(
        &mut self,
        position: PhysicalPosition<f64>,
        window: &mut Arc<Window>,
    ) {
        let new_cursor_pos = K::Point::new(position.x, position.y);
        let cursor_delta = self.mouse.cursor_pos - new_cursor_pos;
        self.mouse.cursor_pos = new_cursor_pos;

        if (self.tool == Tool::Hand && self.mouse.pressed[0]) || self.mouse.pressed[2] {
            self.current_camera().pan_by_screen_delta(-cursor_delta);
            window.request_redraw();
        }
    }

    #[inline(always)]
    pub fn add_element(&mut self, e: Element) {
        self.elements.push((ElementId(self.id_acc), e));
        self.id_acc += 1;
    }

    #[inline(always)]
    fn current_camera(&mut self) -> &mut Camera {
        &mut self.camera[self.camera_idx]
    }

    #[inline(always)]
    pub fn set_viewport(&mut self, width: f64, height: f64) {
        self.current_camera().state_mut().viewport = K::Size::new(width, height);
    }

    pub fn render(&mut self, scene: &mut V::Scene) {
        let camera = self.current_camera();
        let camera_transform = camera.transform();
        let visible = camera.visible_world_rect();

        for (_, el) in self.elements.iter_mut() {
            let bbox = el.world_bounding_box();
            let r = visible.intersect(bbox);
            if r.width() > 0.0 && r.height() > 0.0 {
                el.render_with_base(scene, camera_transform);
            }
        }
    }
}
