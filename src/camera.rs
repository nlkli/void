use vello::kurbo::{Affine, Point, Rect, Size, Vec2};

pub const MIN_ZOOM: f64 = 1e-6;
pub const MAX_ZOOM: f64 = 1e6;

#[derive(Debug, Clone, Copy)]
pub struct CameraPose {
    pub viewport: Size,
    pub position: Point,
    pub zoom: f64,
}

impl Default for CameraPose {
    fn default() -> Self {
        Self {
            viewport: Size::ZERO,
            position: Point::ORIGIN,
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    transform: Affine,
    visible_world_rect: Rect,
    state: CameraPose,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            transform: Affine::IDENTITY,
            visible_world_rect: Rect::default(),
            state: CameraPose::default(),
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> CameraBuilder {
        CameraBuilder::new()
    }

    #[inline]
    pub fn pose(&self) -> &CameraPose {
        &self.state
    }

    #[inline]
    pub fn on_pose<F>(&mut self, f: F)
    where
        F: FnOnce(&mut CameraPose),
    {
        f(&mut self.state);
        self.recompute();
    }

    #[inline]
    fn recompute(&mut self) {
        let state = &mut self.state;

        state.zoom = state.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        let half_viewport = state.viewport * 0.5;
        let position = state.position;
        let zoom = state.zoom;

        self.transform = Affine::translate(half_viewport.to_vec2())
            * Affine::scale(zoom)
            * Affine::translate(-position.to_vec2());

        let half_width = half_viewport.width / zoom;
        let half_height = half_viewport.height / zoom;

        self.visible_world_rect = Rect::new(
            position.x - half_width,
            position.y - half_height,
            position.x + half_width,
            position.y + half_height,
        );
    }

    #[inline]
    pub fn transform(&self) -> Affine {
        self.transform
    }

    #[inline]
    pub fn visible_world_rect(&self) -> Rect {
        self.visible_world_rect
    }

    #[inline]
    pub fn pan_by_screen_delta(&mut self, screen_delta: Vec2) {
        self.on_pose(|state| {
            state.position -= screen_delta / state.zoom;
        });
    }

    pub fn zoom_at(&mut self, screen_point: Point, new_zoom: f64, bias: f64) {
        let new_zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        self.on_pose(|state| {
            let offset = (screen_point.to_vec2() - (state.viewport * 0.5).to_vec2()) * bias;
            let world_under_point = state.position + offset / state.zoom;

            state.zoom = new_zoom;
            state.position = world_under_point - offset / new_zoom;
        });
    }

    #[inline]
    pub fn zoom_by_at(&mut self, screen_point: Point, factor: f64, bias: f64) {
        self.zoom_at(screen_point, self.state.zoom * factor, bias);
    }

    pub fn reset_with_viewport(&mut self) {
        *self = Self::builder().with_viewport(self.state.viewport).build();

        self.recompute();
    }
}

#[derive(Debug, Clone)]
pub struct CameraBuilder {
    c: Camera,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            c: Camera::default(),
        }
    }
}

impl CameraBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_position(mut self, position: Point) -> Self {
        self.c.state.position = position;
        self
    }

    pub fn with_position_point(mut self, x: f64, y: f64) -> Self {
        self.c.state.position = Point::new(x, y);
        self
    }

    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.c.state.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self
    }

    pub fn with_viewport(mut self, viewport: Size) -> Self {
        self.c.state.viewport = viewport;
        self
    }

    pub fn with_viewport_size(mut self, width: f64, height: f64) -> Self {
        self.c.state.viewport = Size::new(width, height);
        self
    }

    pub fn build(self) -> Camera {
        self.c
    }
}
