#[allow(unused_imports)]
#[allow(unused_imports)]
use stuk_text::TextInputState;

#[test]
fn text_input_inserts_and_reads() {
    let mut state = TextInputState::new("hello");
    assert_eq!(state.text(), "hello");
    state.insert_text(" world");
    assert_eq!(state.text(), "hello world");
}

#[test]
fn text_input_handles_backspace() {
    let mut state = TextInputState::new("abc");
    state.delete_backward();
    assert_eq!(state.text(), "ab");
}

#[test]
fn text_input_select_all() {
    let mut state = TextInputState::new("hello world");
    state.select_all();
    let range = state.selection().range();
    assert_eq!(range.start, 0);
    assert!(range.end > 0);
}

#[test]
fn text_input_set_text() {
    let mut state = TextInputState::new("old");
    state.set_text("new");
    assert_eq!(state.text(), "new");
}

#[test]
fn text_input_paste_and_cut() {
    let mut state = TextInputState::new("hello");
    state.select_all();
    let cut = state.cut_selection();
    assert_eq!(cut.as_deref(), Some("hello"));
    assert_eq!(state.text(), "");

    state.paste_text("world");
    assert_eq!(state.text(), "world");
}

#[test]
fn text_input_undo_redo() {
    let mut state = TextInputState::new("a");
    state.insert_text("b");
    assert_eq!(state.text(), "ab");
    state.undo();
    assert_eq!(state.text(), "a");
    state.redo();
    assert_eq!(state.text(), "ab");
}

#[test]
fn text_input_move_caret() {
    let mut state = TextInputState::new("hello");
    state.move_to_end(false);
    state.move_left(true);
    let range = state.selection().range();
    assert_eq!(range.end, 5);
    assert!(!state.selection().is_collapsed());
}

#[test]
fn text_input_secure_mode() {
    let mut state = TextInputState::new("secret");
    state.set_secure(true);
    assert!(state.is_secure());
    let display = state.display_text();
    assert_eq!(display, "******");
}
