use crate::element::Element;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId(u64);

pub enum DocumentCommand {
    None,
}

pub struct Document {
    elements: Vec<(ElementId, Element)>,
    id_acc: u64,
}
