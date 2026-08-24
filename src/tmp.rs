use std::sync::Arc;

use crate::{camera::Camera, element::Element};
use vello as V;
use vello::kurbo as K;
use winit::{
    dpi::PhysicalPosition,
    event as WE, keyboard as WK,
    window::{CursorIcon, Window},
};

/// Default zoom applied to the initial camera on editor creation.
const DEFAULT_ZOOM: f64 = 1.9;

#[derive(Debug, Clone, Copy)]
struct MouseState {
    cursor_pos: K::Point,
    /// Index into `pressed`/`ptime`: 0 = left, 1 = right, 2 = middle.
    pressed: [bool; 3],
    ptime: [std::time::Instant; 3],
}

impl Default for MouseState {
    fn default() -> Self {
        // `Instant` has no guaranteed all-zero representation, so it can't be
        // safely zero-initialized. Build a real default explicitly instead.
        let now = std::time::Instant::now();
        Self {
            cursor_pos: K::Point::ZERO,
            pressed: [false; 3],
            ptime: [now; 3],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId(u64);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Selection,
    Hand,
    Rectangle,
}

#[derive(Debug, Clone, Default)]
pub struct Editor {
    elements: Vec<(ElementId, Element)>,
    selected: Vec<ElementId>,
    // Placeholder for an in-progress marquee/rectangle selection.
    // Not yet used; will likely become a small struct (start/end points).
    selection: Option<()>,
    hit_element: Option<ElementId>,

    id_acc: u64,

    tool: Tool,
    mouse: MouseState,
    camera: Vec<Camera>,
    camera_idx: usize,
}

impl Editor {
    pub fn new() -> Self {
        let mut editor = Self::default();
        editor
            .camera
            .push(Camera::builder().with_zoom(DEFAULT_ZOOM).build());
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
            Tool::Rectangle => window.set_cursor(CursorIcon::Crosshair),
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
        let is_pressed = state == WE::ElementState::Pressed;

        // Resolve the button to a slot in `mouse.pressed`/`mouse.ptime`,
        // running any tool-specific side effects along the way.
        // `None` means "a button we don't track" (e.g. back/forward).
        let button_index = match btn {
            WE::MouseButton::Left => {
                match self.tool {
                    Tool::Selection => {
                        if is_pressed {
                            if let Some(id) = self.hit_element {
                                self.selected.clear();
                                self.selected.push(id);
                                println!("hit: {:?}", id);
                            }
                        }
                    }
                    Tool::Hand => {
                        if is_pressed {
                            window.set_cursor(CursorIcon::Grabbing)
                        } else {
                            self.set_cursor_by_tool(window);
                        }
                    }
                    Tool::Rectangle => {}
                }
                Some(0)
            }
            WE::MouseButton::Right => Some(1),
            WE::MouseButton::Middle => {
                if is_pressed {
                    window.set_cursor(CursorIcon::Grabbing)
                } else {
                    self.set_cursor_by_tool(window);
                }
                Some(2)
            }
            _ => None,
        };

        if let Some(index) = button_index {
            self.mouse.pressed[index] = is_pressed;
            self.mouse.ptime[index] = std::time::Instant::now();
        }
    }

    #[inline]
    fn mouse_cursor_moved_event(
        &mut self,
        position: PhysicalPosition<f64>,
        window: &mut Arc<Window>,
    ) {
        let new_cursor_pos = K::Point::new(position.x, position.y);
        let cursor_delta = new_cursor_pos - self.mouse.cursor_pos;
        self.mouse.cursor_pos = new_cursor_pos;

        // Hit test against elements visible in the current viewport.
        let camera = self.current_camera_mut();
        let transform = camera.transform().inverse();
        let visible = camera.visible_world_rect();
        let world_cursor_pos = transform * self.mouse.cursor_pos;

        self.hit_element = None;
        for (id, el) in self.elements.iter_mut() {
            if !visible.overlaps(el.world_bounding_box()) {
                continue;
            }
            if el.world_bounding_box().contains(world_cursor_pos) {
                self.hit_element = Some(*id);
            }
        }

        if self.tool == Tool::Selection && !self.mouse.pressed[2] {
            if self.hit_element.is_some() {
                window.set_cursor(CursorIcon::Move)
            } else {
                self.set_cursor_by_tool(window);
            }
        }

        // Hand tool + left-button drag, or middle-button drag, pans the camera.
        if (self.tool == Tool::Hand && self.mouse.pressed[0]) || self.mouse.pressed[2] {
            self.current_camera_mut().pan_by_screen_delta(cursor_delta);
            window.request_redraw();
        }
    }

    #[inline(always)]
    pub fn add_element(&mut self, e: Element) {
        self.elements.push((ElementId(self.id_acc), e));
        self.id_acc += 1;
    }

    #[inline(always)]
    fn current_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera[self.camera_idx]
    }

    #[inline(always)]
    pub fn set_viewport(&mut self, width: f64, height: f64) {
        self.current_camera_mut().state_mut().viewport = K::Size::new(width, height);
    }

    pub fn render(&mut self, scene: &mut V::Scene) {
        let camera = self.current_camera_mut();
        let camera_transform = camera.transform();
        let visible = camera.visible_world_rect();

        for (_, el) in self.elements.iter_mut() {
            let bbox = el.world_bounding_box();
            if visible.overlaps(bbox) {
                el.render_with_base(scene, camera_transform);
            }
        }
    }
}
