use crate::any_shape::AnyShape;
use crate::style::Style;
use vello::Scene;
use vello::kurbo::{Affine, Point, Rect, Shape as _, Vec2};

#[derive(Debug, Clone, Default)]
pub enum ElementInner {
    #[default]
    None,
    Shape {
        value: AnyShape,
        style: Style,
    },
    // Text {
    //     content: String,
    //     align: ()
    // },
    Group(Vec<Element>),
}

/// Returns a point inside `rect` using normalized coordinates `(u, v)`.
#[inline]
fn point_at(rect: Rect, u: f64, v: f64) -> Point {
    Point::new(rect.x0 + rect.width() * u, rect.y0 + rect.height() * v)
}

/// Rotates a vector by `angle` radians.
#[inline]
fn rotate_vec(v: Vec2, angle: f64) -> Vec2 {
    (Affine::rotate(angle) * v.to_point()).to_vec2()
}

#[derive(Debug, Clone, Copy)]
pub struct ElementPose {
    pub position: Point,
    pub rotation: f64,
    pub scale: Vec2,
    pub anchor: Vec2,
}

impl Default for ElementPose {
    fn default() -> Self {
        Self {
            position: Point::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            anchor: Vec2::new(0.5, 0.5),
        }
    }
}

impl ElementPose {
    #[inline]
    fn build_transform(&self, bbox: Rect) -> Affine {
        let anchor = point_at(bbox, self.anchor.x, self.anchor.y);

        Affine::translate(self.position.to_vec2())
            * Affine::rotate(self.rotation)
            * Affine::scale_non_uniform(self.scale.x, self.scale.y)
            * Affine::translate(-anchor.to_vec2())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeHandle {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ResizeHandle {
    #[inline]
    const fn uv(self) -> (f64, f64) {
        use ResizeHandle::*;

        match self {
            Left => (0.0, 0.5),
            Right => (1.0, 0.5),
            Top => (0.5, 0.0),
            Bottom => (0.5, 1.0),
            TopLeft => (0.0, 0.0),
            TopRight => (1.0, 0.0),
            BottomLeft => (0.0, 1.0),
            BottomRight => (1.0, 1.0),
        }
    }

    #[inline]
    const fn fixed_uv(self) -> (f64, f64) {
        let (u, v) = self.uv();
        (1.0 - u, 1.0 - v)
    }

    #[inline]
    const fn active_axes(self) -> (bool, bool) {
        use ResizeHandle::*;

        match self {
            Left | Right => (true, false),
            Top | Bottom => (false, true),
            TopLeft | TopRight | BottomLeft | BottomRight => (true, true),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    inner: ElementInner,
    pose: ElementPose,
    local_bbox: Rect,
    world_bbox: Rect,
    transform: Affine,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            inner: ElementInner::default(),
            pose: ElementPose::default(),
            local_bbox: Rect::default(),
            world_bbox: Rect::default(),
            transform: Affine::IDENTITY,
        }
    }
}

impl Element {
    pub fn new(inner: ElementInner) -> Self {
        let mut element = Self::default();
        element.set_inner(inner);
        element
    }

    #[inline]
    pub fn inner(&self) -> &ElementInner {
        &self.inner
    }

    pub fn set_inner(&mut self, inner: ElementInner) {
        self.local_bbox = match &inner {
            ElementInner::Shape { value, .. } => value.bounding_box(),
            ElementInner::Group(elements) => elements
                .iter()
                .map(|element| element.world_bbox)
                .reduce(|acc, bbox| acc.union(bbox))
                .unwrap_or(Rect::ZERO),
            ElementInner::None => Rect::ZERO,
        };

        self.inner = inner;
        self.recompute();
    }

    #[inline]
    pub fn pose(&self) -> &ElementPose {
        &self.pose
    }

    #[inline]
    pub fn local_bbox(&self) -> Rect {
        self.local_bbox
    }

    #[inline]
    pub fn on_pose<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ElementPose),
    {
        f(&mut self.pose);
        self.recompute();
    }

    #[inline]
    fn recompute(&mut self) {
        self.transform = self.pose.build_transform(self.local_bbox);
        self.world_bbox = self.transform.transform_rect_bbox(self.local_bbox);
    }

    #[inline]
    pub fn transform(&self) -> Affine {
        self.transform
    }

    #[inline]
    pub fn world_bbox(&self) -> Rect {
        self.world_bbox
    }

    /// Resizes the element from `handle` to `pointer_world`.
    /// `start_pose` must be the pose captured when the drag started.
    pub fn resize_by_handle(
        &mut self,
        handle: ResizeHandle,
        pointer_world: Point,
        start_pose: &ElementPose,
        min_scale: f64,
    ) {
        let bbox = self.local_bbox;

        let (hu, hv) = handle.uv();
        let (fu, fv) = handle.fixed_uv();
        let (active_x, active_y) = handle.active_axes();

        let handle_local = point_at(bbox, hu, hv);
        let fixed_local = point_at(bbox, fu, fv);
        let anchor_local = point_at(bbox, start_pose.anchor.x, start_pose.anchor.y);

        let d_local = handle_local - fixed_local;

        let start_transform = start_pose.build_transform(bbox);
        let fixed_world = start_transform * fixed_local;

        let local_delta = rotate_vec(pointer_world - fixed_world, -start_pose.rotation);

        let clamp = |value: f64| {
            if value.abs() < min_scale {
                min_scale.copysign(if value == 0.0 { 1.0 } else { value.signum() })
            } else {
                value
            }
        };

        let mut new_scale = start_pose.scale;

        if active_x && d_local.x.abs() > f64::EPSILON {
            new_scale.x = clamp(local_delta.x / d_local.x);
        }

        if active_y && d_local.y.abs() > f64::EPSILON {
            new_scale.y = clamp(local_delta.y / d_local.y);
        }

        let offset = rotate_vec(
            Vec2::new(
                (fixed_local.x - anchor_local.x) * new_scale.x,
                (fixed_local.y - anchor_local.y) * new_scale.y,
            ),
            start_pose.rotation,
        );

        let new_position = fixed_world - offset;

        self.on_pose(|pose| {
            pose.position = new_position;
            pose.scale = new_scale;
        });
    }

    pub fn render(&self, scene: &mut Scene, base: Affine) {
        let transform = base * self.transform;

        match &self.inner {
            ElementInner::Shape { value, style } => {
                if let Some((color, fill)) = style.fill {
                    scene.fill(fill, transform, color, None, value);
                }

                if let Some((color, stroke)) = &style.stroke {
                    scene.stroke(stroke, transform, color, None, value);
                }
            }

            ElementInner::Group(elements) => {
                elements
                    .iter()
                    .for_each(|element| element.render(scene, transform));
            }

            ElementInner::None => {}
        }
    }
}
