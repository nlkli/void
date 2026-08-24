use vello::Scene;
use vello::kurbo::{Affine, Point, Rect, Shape as _, Vec2};

use crate::any_shape::AnyShape;
use crate::style::Style;

#[derive(Debug, Clone, Default)]
pub enum ElementInner {
    #[default]
    None,
    Shape {
        value: AnyShape,
        style: Style,
    },
    Text(String),
    Group(Vec<Element>),
}

#[derive(Debug, Clone, Copy)]
pub struct ElementState {
    pub position: Point,
    pub rotation: f64,
    pub scale: Vec2,
    pub anchor: Vec2,
}

impl Default for ElementState {
    fn default() -> Self {
        Self {
            position: Point::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            anchor: Vec2::new(0.5, 0.5),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    inner: ElementInner,
    state: ElementState,

    local_bbox: Rect,
    world_bbox: Rect,

    transform: Affine,

    dirty: bool,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            inner: ElementInner::default(),
            state: ElementState::default(),
            local_bbox: Rect::default(),
            world_bbox: Rect::default(),
            transform: Affine::IDENTITY,
            dirty: true,
        }
    }
}

impl Element {
    pub fn new(inner: ElementInner) -> Self {
        let mut element = Self::default();
        element.set_inner(inner);
        element
    }

    #[inline(always)]
    pub fn inner(&self) -> &ElementInner {
        &self.inner
    }

    pub fn set_inner(&mut self, inner: ElementInner) {
        self.local_bbox = match &inner {
            ElementInner::Shape { value, .. } => value.bounding_box(),
            ElementInner::Text(_) => todo!(),
            ElementInner::Group(_elements) => todo!(),
            ElementInner::None => Rect::ZERO,
        };
        self.inner = inner;
        self.dirty = true;
    }

    #[inline(always)]
    pub fn state(&self) -> &ElementState {
        &self.state
    }

    #[inline(always)]
    pub fn state_mut(&mut self) -> &mut ElementState {
        self.dirty = true;
        &mut self.state
    }

    fn recompute(&mut self) {
        self.transform = Affine::translate(self.state.position.to_vec2())
            * Affine::rotate(self.state.rotation)
            * Affine::scale_non_uniform(self.state.scale.x, self.state.scale.y)
            * Affine::translate(-Vec2::new(
                self.local_bbox.x0 + self.local_bbox.width() * self.state.anchor.x,
                self.local_bbox.y0 + self.local_bbox.height() * self.state.anchor.y,
            ));

        let [a, b, c, d, e, f] = self.transform.as_coeffs();

        let local_bbox = self.local_bbox;

        let center_x = (local_bbox.x0 + local_bbox.x1) * 0.5;
        let center_y = (local_bbox.y0 + local_bbox.y1) * 0.5;
        let half_w = (local_bbox.x1 - local_bbox.x0) * 0.5;
        let half_h = (local_bbox.y1 - local_bbox.y0) * 0.5;

        let new_center_x = a * center_x + c * center_y + e;
        let new_center_y = b * center_x + d * center_y + f;
        let new_half_w = a.abs() * half_w + c.abs() * half_h;
        let new_half_h = b.abs() * half_w + d.abs() * half_h;

        self.world_bbox = Rect::new(
            new_center_x - new_half_w,
            new_center_y - new_half_h,
            new_center_x + new_half_w,
            new_center_y + new_half_h,
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
    pub fn world_bounding_box(&mut self) -> Rect {
        self.ensure_updated();
        self.world_bbox
    }

    #[inline]
    pub fn render(&mut self, scene: &mut Scene) {
        self.render_with_base(scene, Affine::IDENTITY);
    }

    pub fn render_with_base(&mut self, scene: &mut Scene, base: Affine) {
        let transform = base * self.transform();

        match &self.inner {
            ElementInner::Shape { value, style } => {
                if let Some((color, fill)) = style.fill {
                    scene.fill(fill, transform, color, None, &value);
                }
                if let Some((color, stroke)) = &style.stroke {
                    scene.stroke(stroke, transform, color, None, &value);
                }
            }
            ElementInner::Text(_) => todo!(),
            ElementInner::Group(_elements) => todo!(),
            _ => {}
        }
    }
}
