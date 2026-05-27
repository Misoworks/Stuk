use super::*;

#[test]
fn replaces_selected_text() {
    let mut state = TextInputState::new("search");
    state.set_selection(TextSelection::new(1, 5));

    assert!(state.insert_text("can"));

    assert_eq!(state.text(), "scanh");
    assert_eq!(state.selection(), TextSelection::caret(4));
}

#[test]
fn edits_without_breaking_utf8() {
    let mut state = TextInputState::new("aé日");
    state.move_left(false);

    assert!(state.delete_backward());
    assert_eq!(state.text(), "a日");
    assert_eq!(state.selection(), TextSelection::caret(1));

    assert!(state.insert_text("é"));
    assert_eq!(state.text(), "aé日");
}

#[test]
fn selection_movement_can_extend_or_collapse() {
    let mut state = TextInputState::new("notes");

    state.move_left(true);
    state.move_left(true);
    assert_eq!(state.selection(), TextSelection::new(5, 3));

    state.move_left(false);
    assert_eq!(state.selection(), TextSelection::caret(3));
}

#[test]
fn word_movement_skips_words_and_separators() {
    let mut state = TextInputState::new("first, second_name 日記");

    state.move_word_left(false);
    assert_eq!(state.selection(), TextSelection::caret(19));

    state.move_word_left(false);
    assert_eq!(state.selection(), TextSelection::caret(7));

    state.move_word_right(false);
    assert_eq!(state.selection(), TextSelection::caret(18));

    state.move_word_right(true);
    assert_eq!(state.selection(), TextSelection::new(18, 21));
}

#[test]
fn line_and_document_movement_support_home_end_behavior() {
    let mut state = TextInputState::new("one\ntwo three\nfour");
    state.set_selection(TextSelection::caret(9));

    state.move_to_line_start(false);
    assert_eq!(state.selection(), TextSelection::caret(4));

    state.move_to_line_end(true);
    assert_eq!(state.selection(), TextSelection::new(4, 13));

    state.move_to_start(false);
    assert_eq!(state.selection(), TextSelection::caret(0));

    state.move_to_end(true);
    assert_eq!(state.selection(), TextSelection::new(0, 18));
}

#[test]
fn word_deletion_uses_undoable_word_boundaries() {
    let mut state = TextInputState::new("alpha beta-gamma");
    state.set_selection(TextSelection::caret(10));

    assert!(state.delete_word_backward());
    assert_eq!(state.text(), "alpha -gamma");
    assert_eq!(state.selection(), TextSelection::caret(6));

    assert!(state.delete_word_forward());
    assert_eq!(state.text(), "alpha ");

    assert!(state.undo());
    assert_eq!(state.text(), "alpha -gamma");
    assert!(state.undo());
    assert_eq!(state.text(), "alpha beta-gamma");
}

#[test]
fn secure_display_masks_text() {
    let mut state = TextInputState::new("hush");
    state.set_secure(true);

    assert_eq!(state.display_text(), "****");
}

#[test]
fn disabled_state_ignores_edits() {
    let mut state = TextInputState::new("locked");
    state.set_disabled(true);

    assert!(!state.insert_text("!"));
    assert!(!state.delete_backward());
    assert_eq!(state.text(), "locked");
}

#[test]
fn commits_composition_into_original_selection() {
    let mut state = TextInputState::new("input");
    state.set_selection(TextSelection::new(0, 2));

    assert!(state.start_composition("ou"));
    assert!(state.update_composition("out"));
    assert!(state.commit_composition());

    assert_eq!(state.text(), "output");
    assert_eq!(state.composition(), None);
    assert_eq!(state.selection(), TextSelection::caret(3));
}

#[test]
fn undo_and_redo_restore_text_and_selection() {
    let mut state = TextInputState::new("note");
    state.set_selection(TextSelection::new(1, 3));

    assert!(state.insert_text("am"));
    assert_eq!(state.text(), "name");
    assert_eq!(state.selection(), TextSelection::caret(3));
    assert!(state.can_undo());
    assert_eq!(state.undo_depth(), 1);

    assert!(state.undo());
    assert_eq!(state.text(), "note");
    assert_eq!(state.selection(), TextSelection::new(1, 3));
    assert!(state.can_redo());

    assert!(state.redo());
    assert_eq!(state.text(), "name");
    assert_eq!(state.selection(), TextSelection::caret(3));
}

#[test]
fn new_edit_clears_redo_stack() {
    let mut state = TextInputState::new("abc");

    assert!(state.insert_text("d"));
    assert!(state.undo());
    assert!(state.can_redo());
    assert!(state.insert_text("z"));

    assert!(!state.can_redo());
    assert_eq!(state.text(), "abcz");
}

#[test]
fn copy_cut_and_paste_use_selection() {
    let mut state = TextInputState::new("search");
    state.set_selection(TextSelection::new(1, 5));

    assert_eq!(state.copy_selection().as_deref(), Some("earc"));
    assert_eq!(state.cut_selection().as_deref(), Some("earc"));
    assert_eq!(state.text(), "sh");
    assert!(state.paste_text("can"));
    assert_eq!(state.text(), "scanh");
}

#[test]
fn secure_fields_do_not_expose_selected_text_to_clipboard() {
    let mut state = TextInputState::new("secret");
    state.set_secure(true);
    state.select_all();

    assert_eq!(state.copy_selection(), None);
    assert_eq!(state.cut_selection(), None);
    assert_eq!(state.text(), "secret");
}
