use crate::{
    editor::{Editor, EditorCommand},
    view::{View, ViewCommand},
};

// User -> Command -> UserState
//                       |-> Editor -> DocumentCommand -> Document
//                       |-> View

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClientId(u64);

pub enum SessionCommand {
    None,
    Editor(EditorCommand),
    View(ViewCommand),
}

pub struct Session {
    editor: Editor,
    view: Vec<(ClientId, View)>,
    id_acc: u64,
}

impl Session {
    pub fn create_client(&mut self) -> ClientId {
        let id = ClientId(self.id_acc);
        self.id_acc += 1;

        self.view.push((id, View::new()));

        id
    }

    pub fn delete_client(&mut self, id: ClientId) -> bool {
        if let Some(index) = self.view.iter().position(|v| v.0 == id) {
            self.view.remove(index);
            return true;
        }
        false
    }

    pub fn dispatch(&mut self, user_id: ClientId, command: SessionCommand) {
        todo!();
    }
}
