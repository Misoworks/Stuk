use stuk_actions::{ActionDescriptor, ActionRegistry, Modifiers, Shortcut, validate_action_id};

fn bench_action_registry(criterion: &mut criterion::Criterion) {
    criterion.bench_function("actions/register_100_actions", |b| {
        b.iter(|| {
            let mut registry = ActionRegistry::new();
            for i in 0..100 {
                let _ = registry.register(
                    ActionDescriptor::new(format!("action.{i}"), format!("Action {i}"))
                        .category("Test"),
                );
            }
            registry
        })
    });

    criterion.bench_function("actions/validate_100_ids", |b| {
        b.iter(|| {
            for i in 0..100 {
                let _ = validate_action_id(&format!("action.{i}"));
            }
        })
    });
}

fn bench_shortcut_parse(criterion: &mut criterion::Criterion) {
    criterion.bench_function("actions/parse_100_shortcuts", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = Shortcut::parse("Ctrl+Shift+N");
            }
        })
    });
}

criterion::criterion_group!(benches, bench_action_registry, bench_shortcut_parse);
criterion::criterion_main!(benches);
