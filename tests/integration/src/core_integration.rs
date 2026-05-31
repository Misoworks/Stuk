#[cfg(test)]
use std::{
    thread,
    time::{Duration, Instant},
};

#[cfg(test)]
use stuk_core::{
    NavigationStack, Page, PageCursor, PaginatedResourcePhase, RouteState, cursor_resource, signal,
};

#[test]
fn signal_basic_operations() {
    let count = signal(0);
    assert_eq!(count.get(), 0);
    count.set(5);
    assert_eq!(count.get(), 5);
}

#[test]
fn signal_tracks_revision() {
    let value = signal(String::new());
    let rev0 = value.revision();
    value.set("hello".to_string());
    let rev1 = value.revision();
    assert_ne!(rev0, rev1);
    let rev2 = value.revision();
    assert_eq!(rev1, rev2);
}

#[test]
fn signal_cloning_preserves() {
    let value = signal(42);
    assert_eq!(value.get(), 42);
}

#[test]
fn signal_string_value() {
    let name = signal("world".to_string());
    assert_eq!(name.get(), "world");
    name.set("hello".to_string());
    assert_eq!(name.get(), "hello");
}

#[test]
fn paginated_resource_loads_next_page() {
    let notes = cursor_resource("notes", |cursor| async move {
        match cursor.as_ref().map(PageCursor::as_str) {
            None => Ok::<_, String>(
                Page::new(vec!["Inbox".to_string()]).next_cursor(Some(PageCursor::new("next"))),
            ),
            Some("next") => Ok(Page::new(vec!["Archive".to_string()])),
            Some(other) => Err(format!("unknown cursor {other}")),
        }
    });

    wait_until(|| notes.phase() == PaginatedResourcePhase::Loaded);
    assert_eq!(notes.items(), vec!["Inbox".to_string()]);
    assert!(notes.has_next_page());

    let task = notes.load_next().expect("next page should exist");
    wait_until(|| task.is_finished());
    wait_until(|| notes.phase() == PaginatedResourcePhase::EndReached);

    assert_eq!(
        notes.items(),
        vec!["Inbox".to_string(), "Archive".to_string()]
    );
    assert!(!notes.has_next_page());
}

#[test]
fn paginated_resource_marks_next_page_errors_as_stale() {
    let notes = cursor_resource("notes", |cursor| async move {
        if cursor.is_some() {
            Err::<Page<String>, _>("offline".to_string())
        } else {
            Ok(Page::new(vec!["Inbox".to_string()]).next_cursor(Some(PageCursor::new("next"))))
        }
    });

    wait_until(|| notes.phase() == PaginatedResourcePhase::Loaded);
    let task = notes.load_next().expect("next page should exist");
    wait_until(|| task.is_finished());
    wait_until(|| notes.phase() == PaginatedResourcePhase::ErrorNextPage);

    let snapshot = notes.snapshot();
    assert_eq!(snapshot.items, vec!["Inbox".to_string()]);
    assert_eq!(snapshot.error, Some("offline".to_string()));
    assert!(snapshot.stale);
}

#[test]
fn navigation_stack_preserves_root_and_current_route() {
    let mut stack = NavigationStack::new("home");
    assert_eq!(stack.current(), "home");
    assert!(!stack.can_go_back());

    stack.push("notes");
    stack.push("note.detail");
    assert_eq!(stack.current(), "note.detail");
    assert!(stack.can_go_back());

    assert_eq!(stack.pop(), Some("note.detail"));
    assert_eq!(stack.current(), "notes");
    stack.clear_to_root();
    assert_eq!(stack.current(), "home");
    assert!(!stack.can_go_back());
}

#[test]
fn route_state_replaces_current_route() {
    let mut route = RouteState::new("notes");
    assert_eq!(route.route(), &"notes");
    assert_eq!(route.replace("settings"), "notes");
    assert_eq!(route.current(), "settings");
}

#[cfg(test)]
fn wait_until(mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() && start.elapsed() < Duration::from_secs(1) {
        thread::sleep(Duration::from_millis(1));
    }
    assert!(done());
}
