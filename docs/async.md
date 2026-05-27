# Async State Views

Use `resource` for async reads and `mutation` for async writes. `ResourceView` and `MutationView` turn those handles into regular view-tree elements with explicit loading, empty, error, and success states.

```rust
use stuk::prelude::*;

struct NotesWindow {
    recent_notes: Resource<Vec<String>, String>,
    save_note: Mutation<String, String, String>,
}

impl Default for NotesWindow {
    fn default() -> Self {
        Self {
            recent_notes: resource("notes.recent", || async {
                Ok::<_, String>(vec!["Project outline".to_string()])
            }),
            save_note: mutation("notes.save", |body: String| async move {
                Ok::<_, String>(format!("Saved {body}"))
            }),
        }
    }
}
```

Render a resource by mapping each state to the widget that belongs in the interface:

```rust
ResourceView::new(self.recent_notes.clone())
    .loading(|| Spinner::new("Loading recent notes"))
    .empty_when(|notes| notes.is_empty())
    .empty(|| EmptyState::new("No recent notes"))
    .error(|error| ErrorView::new(error.clone()))
    .data(|notes| {
        let mut list = List::new().spacing(6.0);

        for note in notes {
            list = list.child(Text::new(note.clone()).muted());
        }

        list
    });
```

Render a mutation near the control that submits it:

```rust
MutationView::new(self.save_note.clone())
    .idle(|| Text::new("No save yet").muted())
    .pending(|| Spinner::new("Saving note"))
    .success(|message| Badge::new(message.clone()))
    .error(|error| ErrorView::new(error.clone()));
```

Submit mutations from action handling:

```rust
fn handle_action(&mut self, action_id: &str, _cx: &mut Cx) {
    if action_id == "notes.save" {
        self.save_note.submit("draft body".to_string());
    }
}
```
