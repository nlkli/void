use std::sync::Arc;

use crate::{
    camera::Camera,
    document::{Document, ElementId},
    element::Element,
    style::Style,
};
use vello::kurbo as K;
use vello::{self as V, kurbo::Shape};
use winit::{
    dpi::PhysicalPosition,
    event as WE, keyboard as WK,
    window::{CursorIcon, Window},
};

#[derive(Debug, Clone, Copy)]
struct MouseState {
    /// Index into `pressed`/`ptime`: 0 = left, 1 = right, 2 = middle.
    cursor_pos: K::Point,
    pressed: [bool; 3],
    ptime: [std::time::Instant; 3],
}

impl Default for MouseState {
    fn default() -> Self {
        // `Instant` cannot be safely zero-initialized.
        let now = std::time::Instant::now();

        Self {
            cursor_pos: K::Point::ZERO,
            pressed: [false; 3],
            ptime: [now; 3],
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Selection,
    Hand,
    Rectangle,
}

#[derive(Debug, Clone, Default)]
pub struct Editor {
    pub document: Document,

    selected: Vec<ElementId>,
    dragging_selection: bool,

    // Placeholder for marquee selection.
    selection: Option<K::Rect>,
    drag_start: Option<K::Point>,

    hit_element: Option<ElementId>,

    tool: Tool,
    mouse: MouseState,
    camera: Vec<Camera>,
    camera_idx: usize,

    is_super_pressed: bool,
    is_shift_pressed: bool,
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
        let is_pressed = event.state == WE::ElementState::Pressed;

        match event.physical_key {
            WK::PhysicalKey::Code(key_code) => match key_code {
                WK::KeyCode::KeyH => {
                    if is_pressed {
                        self.change_tool(Tool::Hand, window);
                    }
                }

                WK::KeyCode::KeyV => {
                    if is_pressed {
                        self.change_tool(Tool::Selection, window);
                    }
                }

                WK::KeyCode::SuperLeft | WK::KeyCode::SuperRight => {
                    self.is_super_pressed = is_pressed;
                }

                WK::KeyCode::ShiftLeft | WK::KeyCode::ShiftRight => {
                    self.is_shift_pressed = is_pressed;
                }

                WK::KeyCode::KeyZ => {
                    if is_pressed {
                        self.current_camera_mut().reset_with_viewport();
                        window.request_redraw();
                    }
                }

                WK::KeyCode::BracketLeft if self.is_shift_pressed => {
                    if is_pressed {
                        self.selected.iter().for_each(|id| {
                            self.document.move_down(*id);
                        });
                        window.request_redraw();
                    }
                }

                WK::KeyCode::BracketRight if self.is_shift_pressed => {
                    if is_pressed {
                        self.selected.iter().for_each(|id| {
                            self.document.move_up(*id);
                        });
                        window.request_redraw();
                    }
                }

                WK::KeyCode::BracketLeft => {
                    if is_pressed {
                        self.selected.iter().for_each(|id| {
                            self.document.move_to_front(*id);
                        });
                        window.request_redraw();
                    }
                }

                WK::KeyCode::BracketRight => {
                    if is_pressed {
                        self.selected.iter().for_each(|id| {
                            self.document.move_to_back(*id);
                        });
                        window.request_redraw();
                    }
                }

                WK::KeyCode::Delete | WK::KeyCode::Backspace => {
                    if is_pressed {
                        self.selected.iter().for_each(|id| {
                            self.document.remove(*id);
                        });
                        window.request_redraw();
                    }
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

        let button_index = match btn {
            WE::MouseButton::Left => {
                match self.tool {
                    Tool::Selection => {
                        if is_pressed {
                            if let Some(id) = self.hit_element {
                                let pos = self.selected.iter().position(|v| *v == id);
                                if let Some(pos) = pos {
                                    if self.is_super_pressed {
                                        self.selected.remove(pos);
                                    }
                                } else {
                                    if !self.is_super_pressed {
                                        self.selected.clear();
                                    }
                                    self.selected.push(id);
                                }

                                self.dragging_selection = true;
                            } else {
                                self.dragging_selection = false;
                                self.drag_start = Some(self.mouse.cursor_pos);
                                self.selection = None;
                            }
                        } else {
                            self.dragging_selection = false;

                            if let Some(rect) = self.selection.take() {
                                let transform = self.current_camera_mut().transform().inverse();
                                let world_rect = transform.transform_rect_bbox(rect);

                                self.selected.clear();
                                self.document
                                    .iter()
                                    .filter(|(_, e)| world_rect.contains(e.world_bbox().center()))
                                    .for_each(|(id, _)| self.selected.push(*id));
                            }

                            self.drag_start = None;
                        }

                        window.request_redraw();
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
        let prev_cursor_pos = self.mouse.cursor_pos;
        self.mouse.cursor_pos = new_cursor_pos;

        let camera = self.current_camera_mut();
        let transform = camera.transform();
        let transform_inv = transform.inverse();
        // let visible = camera.visible_world_rect();

        let world_cursor_pos = transform_inv * new_cursor_pos;
        self.hit_element = self.document.hit_test(world_cursor_pos);

        if self.tool == Tool::Selection && self.mouse.pressed[0] {
            let prev_world_cursor_pos = transform_inv * prev_cursor_pos;
            let world_cursor_delta = world_cursor_pos - prev_world_cursor_pos;

            if self.dragging_selection {
                self.selected.iter().for_each(|id| {
                    self.document
                        .get_mut(*id)
                        .map(|e| e.on_pose(|p| p.position += world_cursor_delta));
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

        // Hand tool with left-button drag or middle-button drag pans the camera.
        if (self.tool == Tool::Hand && self.mouse.pressed[0]) || self.mouse.pressed[2] {
            let screen_cursor_delta = new_cursor_pos - prev_cursor_pos;
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

            let screen_point = self.mouse.cursor_pos;

            self.current_camera_mut()
                .zoom_by_at(screen_point, factor, 1.0);
        } else {
            self.current_camera_mut().on_pose(|s| {
                s.position.x -= dx;
                s.position.y -= dy;
            });
        }

        window.request_redraw();
    }

    #[inline]
    fn current_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera[self.camera_idx]
    }

    #[inline]
    fn current_camera(&self) -> &Camera {
        &self.camera[self.camera_idx]
    }

    #[inline]
    pub fn set_viewport(&mut self, width: f64, height: f64) {
        self.current_camera_mut()
            .on_pose(|s| s.viewport = K::Size::new(width, height));
    }

    pub fn render(&self, scene: &mut V::Scene) {
        let camera = self.current_camera();

        let camera_transform = camera.transform();
        let visible = camera.visible_world_rect();

        self.document.render(scene, visible, camera_transform);

        if !self.selected.is_empty() {
            let style = Style::filled_and_stroked(
                V::peniko::color::palette::css::CYAN.with_alpha(0.1),
                V::peniko::color::palette::css::CYAN.with_alpha(0.9),
                K::Stroke::new(2.),
            );

            self.selected
                .iter()
                .filter_map(|id| self.document.get(*id))
                .for_each(|e| draw(scene, &style, camera_transform, &e.world_bbox()));
        }

        if let Some(selection) = self.selection.as_ref() {
            let style = Style::filled_and_stroked(
                V::peniko::color::palette::css::GRAY.with_alpha(0.1),
                V::peniko::color::palette::css::GRAY.with_alpha(0.9),
                K::Stroke::new(2.),
            );

            draw(scene, &style, K::Affine::IDENTITY, selection);
        }

        if self.selected.len() > 1 {
            if let Some(selection_bounds) = self.document.group_bounds(&self.selected) {
                let style = Style::stroked(
                    V::peniko::color::palette::css::CYAN.with_alpha(0.9),
                    K::Stroke::new(2.),
                );

                draw(scene, &style, camera_transform, &selection_bounds);
            }
        }
    }
}

#[inline]
fn draw(scene: &mut V::Scene, style: &Style, transform: K::Affine, shape: &impl Shape) {
    if let Some((color, fill)) = style.fill {
        scene.fill(fill, transform, color, None, shape);
    }

    if let Some((color, stroke)) = &style.stroke {
        scene.stroke(stroke, transform, color, None, shape);
    }
}
