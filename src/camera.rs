use vello::{
    Scene,
    kurbo::{Affine, Point, Rect, Size, Vec2},
};

pub const MIN_ZOOM: f64 = 1e-6;
pub const MAX_ZOOM: f64 = 1e6;

#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    pub viewport: Size,
    pub position: Point,
    pub zoom: f64,
    pub rotation: f64,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            viewport: Size::ZERO,
            position: Point::ORIGIN,
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    transform: Affine,
    /// Axis-aligned world-space rect covering the viewport. Valid iff `!dirty`.
    visible_world_rect: Rect,
    state: CameraState,
    dirty: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            transform: Affine::IDENTITY,
            visible_world_rect: Rect::default(), // overwritten on first ensure_updated(), dirty = true
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

    /// Recomputes `transform` and `visible_world_rect` together — both depend
    /// on the same clamped zoom and rotation, so keeping them in one place
    /// rules out desync (previously the zoom clamp only happened inside
    /// `transform()`, so `visible_world_rect()` could read an unclamped zoom
    /// if called first).
    fn recompute(&mut self) {
        self.state.zoom = self.state.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let s = &self.state;

        self.transform = Affine::translate((s.viewport * 0.5).to_vec2())
            * Affine::rotate(s.rotation)
            * Affine::scale(s.zoom)
            * Affine::translate(-s.position.to_vec2());

        let inv_zoom = 1.0 / s.zoom.abs();
        let hw = s.viewport.width * 0.5 * inv_zoom;
        let hh = s.viewport.height * 0.5 * inv_zoom;

        let (half_x, half_y) = if s.rotation == 0.0 {
            (hw, hh)
        } else {
            let (sin_t, cos_t) = s.rotation.sin_cos();
            let (sin_t, cos_t) = (sin_t.abs(), cos_t.abs());
            // FMA-friendly form: cos_t*hw + sin_t*hh
            (cos_t.mul_add(hw, sin_t * hh), sin_t.mul_add(hw, cos_t * hh))
        };

        self.visible_world_rect = Rect::new(
            s.position.x - half_x,
            s.position.y - half_y,
            s.position.x + half_x,
            s.position.y + half_y,
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

    /// O(1) when nothing changed since the last call.
    /// NOTE: now `&mut self` — the value is cached lazily, same as `transform()`.
    #[inline]
    pub fn visible_world_rect(&mut self) -> Rect {
        self.ensure_updated();
        self.visible_world_rect
    }

    pub fn pan_by_screen_delta(&mut self, screen_delta: Vec2) {
        let s = &self.state;
        let inv_zoom = 1.0 / s.zoom;

        let (dx, dy) = if s.rotation == 0.0 {
            (screen_delta.x * inv_zoom, screen_delta.y * inv_zoom)
        } else {
            let (sin_t, cos_t) = s.rotation.sin_cos();
            (
                cos_t.mul_add(screen_delta.x, sin_t * screen_delta.y) * inv_zoom,
                (-sin_t).mul_add(screen_delta.x, cos_t * screen_delta.y) * inv_zoom,
            )
        };

        let s = self.state_mut();

        s.position.x -= dx;
        s.position.y -= dy;
    }

    pub fn zoom_at(&mut self, screen_point: Point, new_zoom: f64) {
        let new_zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let s = &self.state;

        let center = (s.viewport * 0.5).to_vec2();
        let offset = screen_point.to_vec2() - center;

        let (rx, ry) = if s.rotation == 0.0 {
            (offset.x, offset.y)
        } else {
            let (sin_t, cos_t) = s.rotation.sin_cos();
            (
                cos_t.mul_add(offset.x, sin_t * offset.y),
                (-sin_t).mul_add(offset.x, cos_t * offset.y),
            )
        };

        let old_zoom = s.zoom;
        let position = s.position;

        let inv_old = 1.0 / old_zoom;
        let inv_new = 1.0 / new_zoom;

        let world_x = position.x + rx * inv_old;
        let world_y = position.y + ry * inv_old;

        let new_position = Point::new(world_x - rx * inv_new, world_y - ry * inv_new);

        self.state.zoom = new_zoom;
        self.state.position = new_position;
        self.dirty = true;
    }

    #[inline(always)]
    pub fn zoom_by_at(&mut self, screen_point: Point, factor: f64) {
        self.zoom_at(screen_point, self.state.zoom * factor);
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

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.c.state.rotation = rotation;
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
