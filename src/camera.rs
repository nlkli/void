use vello::kurbo::{Affine, Point, Rect, Size, Vec2};

pub const MIN_ZOOM: f64 = 1e-6;
pub const MAX_ZOOM: f64 = 1e6;

#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    pub viewport: Size,
    pub position: Point,
    pub zoom: f64,
}

impl Default for CameraState {
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
    state: CameraState,
    dirty: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            transform: Affine::IDENTITY,
            visible_world_rect: Rect::default(),
            state: CameraState::default(),
            dirty: true,
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

    #[inline(always)]
    pub fn state(&self) -> &CameraState {
        &self.state
    }

    #[inline(always)]
    pub fn state_mut(&mut self) -> &mut CameraState {
        self.dirty = true;
        &mut self.state
    }

    fn recompute(&mut self) {
        let s = &mut self.state;
        s.zoom = s.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        self.transform = Affine::translate((s.viewport * 0.5).to_vec2())
            * Affine::scale(s.zoom)
            * Affine::translate(-s.position.to_vec2());

        let hx = s.viewport.width * 0.5 / s.zoom;
        let hy = s.viewport.height * 0.5 / s.zoom;

        self.visible_world_rect = Rect::new(
            s.position.x - hx,
            s.position.y - hy,
            s.position.x + hx,
            s.position.y + hy,
        );

        self.dirty = false;
    }

    #[inline(always)]
    fn ensure_updated(&mut self) {
        if self.dirty {
            self.recompute();
        }
    }

    #[inline]
    pub fn transform(&mut self) -> Affine {
        self.ensure_updated();
        self.transform
    }

    #[inline]
    pub fn visible_world_rect(&mut self) -> Rect {
        self.ensure_updated();
        self.visible_world_rect
    }

    pub fn pan_by_screen_delta(&mut self, screen_delta: Vec2) {
        let s = self.state_mut();
        s.position -= screen_delta / s.zoom;
    }

    pub fn zoom_at(&mut self, screen_point: Point, new_zoom: f64, bias: f64) {
        self.ensure_updated();

        let new_zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let s = self.state_mut();
        let full_offset = screen_point.to_vec2() - (s.viewport * 0.5).to_vec2();
        let offset = full_offset * bias;
        let world_under_point = s.position + offset / s.zoom;

        s.zoom = new_zoom;
        s.position = world_under_point - offset / new_zoom;
    }

    #[inline(always)]
    pub fn zoom_by_at(&mut self, screen_point: Point, factor: f64, bias: f64) {
        self.zoom_at(screen_point, self.state.zoom * factor, bias);
    }

    #[inline(always)]
    pub fn reset_with_viewport(&mut self) {
        *self = Self::builder().with_viewport(self.state.viewport).build();
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

    // pub fn with_rotation(mut self, rotation: f64) -> Self {
    //     self.c.state.rotation = rotation;
    //     self
    // }

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
