use editor::Editor;

mod terminal;
mod editor;
mod document;
mod row;
pub use editor::Position;
fn main(){
    let mut editor=Editor::default();
    editor.run();
}