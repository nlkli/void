use std::sync::Arc;

use crate::{camera::Camera, element::Element, style::Style};
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
    dragging_selection: bool,
    // Placeholder for an in-progress marquee/rectangle selection.
    // Not yet used; will likely become a small struct (start/end points).
    selection: Option<K::Rect>,   // экранные координаты рамки
    drag_start: Option<K::Point>, // экранная точка начала drag

    hit_element: Option<ElementId>,

    id_acc: u64,

    tool: Tool,
    mouse: MouseState,
    camera: Vec<Camera>,
    camera_idx: usize,

    is_super_pressed: bool,
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
            WE::WindowEvent::MouseWheel { delta, .. } => {
                self.mouse_wheel_event(delta, window);
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
        println!("key: {:?}", event.physical_key);
        match event.physical_key {
            WK::PhysicalKey::Code(key_code) => match key_code {
                WK::KeyCode::KeyH => {
                    self.change_tool(Tool::Hand, window);
                }
                WK::KeyCode::KeyV => {
                    self.change_tool(Tool::Selection, window);
                }
                WK::KeyCode::SuperLeft | WK::KeyCode::SuperRight => {
                    self.is_super_pressed = event.state == WE::ElementState::Pressed;
                }
                WK::KeyCode::KeyZ => {
                    self.current_camera_mut().reset_with_viewport();
                    window.request_redraw();
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
                                if !self.selected.contains(&id) {
                                    self.selected.clear();
                                    self.selected.push(id);
                                }
                                self.dragging_selection = true;
                                window.request_redraw();
                            } else {
                                self.dragging_selection = false;
                                self.drag_start = Some(self.mouse.cursor_pos);
                                self.selection = None;
                            }
                        } else {
                            self.dragging_selection = false;

                            if let Some(rect) = self.selection.take() {
                                let transform = self.current_camera_mut().transform().inverse();
                                let p0 = transform * K::Point::new(rect.x0, rect.y0);
                                let p1 = transform * K::Point::new(rect.x1, rect.y1);
                                let world_rect = K::Rect::from_points(p0, p1);

                                self.selected.clear();
                                for (id, el) in self.elements.iter_mut() {
                                    if world_rect.contains(el.world_bounding_box().center()) {
                                        self.selected.push(*id);
                                    }
                                }
                            }
                            self.drag_start = None;
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
        let prev_cursor_pos = self.mouse.cursor_pos; // save before overwrite
        self.mouse.cursor_pos = new_cursor_pos;

        let camera = self.current_camera_mut();
        let transform = camera.transform();
        let transform_inv = transform.inverse();
        let visible = camera.visible_world_rect();

        let world_cursor_pos = transform_inv * new_cursor_pos;
        let prev_world_cursor_pos = transform_inv * prev_cursor_pos;
        let world_cursor_delta = world_cursor_pos - prev_world_cursor_pos;

        // screen-space delta, still needed for camera panning below
        let screen_cursor_delta = new_cursor_pos - prev_cursor_pos;

        self.hit_element = None;
        for (id, el) in self.elements.iter_mut() {
            if !visible.overlaps(el.world_bounding_box()) {
                continue;
            }
            if el.world_bounding_box().contains(world_cursor_pos) {
                self.hit_element = Some(*id);
            }
        }

        if self.tool == Tool::Selection && self.mouse.pressed[0] {
            if self.dragging_selection {
                self.elements
                    .iter_mut()
                    .filter(|(id, _)| self.selected.contains(id))
                    .for_each(|(_, el)| {
                        el.on_state(|s| s.position = s.position + world_cursor_delta)
                    });
                window.request_redraw();
            } else if let Some(start) = self.drag_start {
                self.selection = Some(K::Rect::from_points(start, self.mouse.cursor_pos));
                window.request_redraw();
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
        // Panning uses screen-space delta because pan_by_screen_delta expects it.
        if (self.tool == Tool::Hand && self.mouse.pressed[0]) || self.mouse.pressed[2] {
            self.current_camera_mut()
                .pan_by_screen_delta(screen_cursor_delta);
            window.request_redraw();
        }
    }

    fn mouse_wheel_event(&mut self, delta: WE::MouseScrollDelta, window: &mut Arc<Window>) {
        const LINE_HEIGHT: f64 = 16.0;
        let (dx, dy) = match delta {
            WE::MouseScrollDelta::LineDelta(x, y) => {
                (x as f64 * LINE_HEIGHT, y as f64 * LINE_HEIGHT)
            }
            WE::MouseScrollDelta::PixelDelta(p) => (p.x, p.y),
        };

        if self.is_super_pressed {
            const ZOOM_SPEED: f64 = 0.01;
            const MIN_ZOOM_FACTOR: f64 = 0.9;
            const MAX_ZOOM_FACTOR: f64 = 1.1;

            let factor = (1.0 + dy * ZOOM_SPEED).clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
            let screen_poin = self.mouse.cursor_pos;
            self.current_camera_mut()
                .zoom_by_at(screen_poin, factor, 1.);
        } else {
            let camera_state = self.current_camera_mut().state_mut();
            camera_state.position.x += dx;
            camera_state.position.y += dy;
        }

        window.request_redraw();
    }

    pub fn selection_bounds(&self) -> Option<K::Rect> {
        self.selected
            .iter()
            .filter_map(|id| self.element_by_id(*id))
            .map(|el| el.world_bounding_box())
            .reduce(|acc, bbox| acc.union(bbox))
    }

    #[inline]
    fn element_by_id(&self, id: ElementId) -> Option<&Element> {
        self.elements
            .iter()
            .find(|(eid, _)| *eid == id)
            .map(|(_, el)| el)
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
                el.render(scene, camera_transform);
            }
        }

        if !self.selected.is_empty() {
            let style = Style::filled_and_stroked(
                V::peniko::color::palette::css::CYAN.with_alpha(0.1),
                V::peniko::color::palette::css::CYAN.with_alpha(0.9),
                2.0,
            );
            for id in self.selected.iter() {
                if let Some((_, el)) = self.elements.iter_mut().find(|(v, _)| *v == *id) {
                    let bbox = el.world_bounding_box();
                    if let Some((color, fill)) = style.fill {
                        scene.fill(fill, camera_transform, color, None, &bbox);
                    }
                    if let Some((color, stroke)) = &style.stroke {
                        scene.stroke(stroke, camera_transform, color, None, &bbox);
                    }
                }
            }
        }

        if let Some(selection) = self.selection.as_ref() {
            let style = Style::filled_and_stroked(
                V::peniko::color::palette::css::GRAY.with_alpha(0.1),
                V::peniko::color::palette::css::GRAY.with_alpha(0.9),
                2.0,
            );
            if let Some((color, fill)) = style.fill {
                scene.fill(fill, K::Affine::IDENTITY, color, None, selection);
            }
            if let Some((color, stroke)) = &style.stroke {
                scene.stroke(stroke, K::Affine::IDENTITY, color, None, selection);
            }
        }
    }
}
