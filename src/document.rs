use crate::element::Element;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use vello::{Scene, kurbo as K};

pub enum DocummentCommand {
    Push {
    },
}

pub type ElementId = u64;

#[derive(Debug, Clone, Default)]
pub struct Document {
    elements: IndexMap<ElementId, Element, FxBuildHasher>,
    id_acc: ElementId,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn render(&self, scene: &mut Scene, visible_bounds: K::Rect, transform: K::Affine) {
        self.elements
            .values()
            .filter(|e| visible_bounds.overlaps(e.world_bbox()))
            .for_each(|e| e.render(scene, transform));
    }

    #[inline]
    pub fn group_bounds(&self, group: &[ElementId]) -> Option<K::Rect> {
        group
            .iter()
            .filter_map(|id| self.elements.get(id))
            .map(|el| el.world_bbox())
            .reduce(|acc, bbox| acc.union(bbox))
    }

    #[inline]
    pub fn hit_test(&self, point: K::Point) -> Option<ElementId> {
        self.elements
            .iter()
            .rev()
            .find(|(_, e)| e.world_bbox().contains(point))
            .map(|(id, _)| *id)
    }

    #[inline]
    pub fn push(&mut self, e: Element) -> ElementId {
        self.elements.insert(self.id_acc, e);
        self.id_acc += 1;
        self.id_acc
    }

    #[inline]
    pub fn get(&self, id: ElementId) -> Option<&Element> {
        self.elements.get(&id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: ElementId) -> Option<&mut Element> {
        self.elements.get_mut(&id)
    }

    #[inline]
    pub fn contains(&self, id: ElementId) -> bool {
        self.elements.contains_key(&id)
    }

    #[inline]
    pub fn position_of(&self, id: ElementId) -> Option<usize> {
        self.elements.get_index_of(&id)
    }

    #[inline]
    pub fn get_by_position(&self, pos: usize) -> Option<(&ElementId, &Element)> {
        self.elements.get_index(pos)
    }

    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&ElementId, &Element)> {
        self.elements.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (&ElementId, &mut Element)> {
        self.elements.iter_mut()
    }

    #[inline]
    pub fn values(&self) -> impl DoubleEndedIterator<Item = &Element> {
        self.elements.values()
    }

    #[inline]
    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Element> {
        self.elements.values_mut()
    }

    pub fn move_to_front(&mut self, id: ElementId) -> bool {
        match self.elements.get_index_of(&id) {
            Some(pos) if pos > 0 => {
                self.elements.move_index(pos, 0);
                true
            }
            Some(_) => true,
            None => false,
        }
    }

    pub fn move_to_back(&mut self, id: ElementId) -> bool {
        let Some(pos) = self.elements.get_index_of(&id) else {
            return false;
        };
        let last = self.elements.len() - 1;
        if pos < last {
            self.elements.move_index(pos, last);
        }
        true
    }

    pub fn move_up(&mut self, id: ElementId) -> bool {
        if let Some(pos) = self.elements.get_index_of(&id) {
            if pos + 1 < self.elements.len() {
                self.elements.swap_indices(pos, pos + 1);
                return true;
            }
        }
        false
    }

    pub fn move_down(&mut self, id: ElementId) -> bool {
        if let Some(pos) = self.elements.get_index_of(&id) {
            if pos > 0 {
                self.elements.swap_indices(pos, pos - 1);
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn remove(&mut self, id: ElementId) -> Option<Element> {
        self.elements.shift_remove(&id)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}
