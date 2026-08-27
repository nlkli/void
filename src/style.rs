use vello::{
    kurbo::Stroke,
    peniko::{Color, Fill},
};

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fill: Option<(Color, Fill)>,
    pub stroke: Option<(Color, Stroke)>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filled(color: Color) -> Self {
        Self {
            fill: Some((color, Fill::NonZero)),
            stroke: None,
        }
    }

    pub fn stroked(color: Color, stroke: Stroke) -> Self {
        Self {
            fill: None,
            stroke: Some((color, stroke)),
        }
    }

    pub fn filled_and_stroked(fill_color: Color, stroke_color: Color, stroke: Stroke) -> Self {
        Self {
            fill: Some((fill_color, Fill::NonZero)),
            stroke: Some((stroke_color, stroke)),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.set_fill_color(color);
        self.set_stroke_color(color);
    }

    #[inline(always)]
    pub fn set_fill_color(&mut self, color: Color) {
        if let Some((fill_color, _)) = &mut self.fill {
            *fill_color = color;
        }
    }

    #[inline(always)]
    pub fn set_stroke_color(&mut self, color: Color) {
        if let Some((stroke_color, _)) = &mut self.stroke {
            *stroke_color = color;
        }
    }

    #[inline(always)]
    pub fn is_filled(&self) -> bool {
        self.fill.is_some()
    }

    #[inline(always)]
    pub fn is_stroked(&self) -> bool {
        self.stroke.is_some()
    }

    #[inline(always)]
    pub fn is_filled_and_stroked(&self) -> bool {
        self.is_filled() && self.is_stroked()
    }
}
