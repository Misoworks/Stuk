use stuk_text::TextInputState;

fn bench_text_insert(criterion: &mut criterion::Criterion) {
    criterion.bench_function("text_input/insert_100_chars", |b| {
        b.iter(|| {
            let mut state = TextInputState::new("");
            for i in 0..100 {
                state.insert_text(&format!("line {i}\n"));
            }
            state
        })
    });
}

fn bench_text_select_all(criterion: &mut criterion::Criterion) {
    let text = "hello world ".repeat(100);
    criterion.bench_function("text_input/select_all_1200_chars", |b| {
        b.iter(|| {
            let mut state = TextInputState::new(&text);
            state.select_all();
            state
        })
    });
}

fn bench_text_undo_redo(criterion: &mut criterion::Criterion) {
    criterion.bench_function("text_input/undo_redo_cycle", |b| {
        b.iter(|| {
            let mut state = TextInputState::new("initial");
            state.insert_text(" added");
            state.undo();
            state.redo();
            state
        })
    });
}

fn bench_text_movement(criterion: &mut criterion::Criterion) {
    criterion.bench_function("text_input/move_word_right_50_times", |b| {
        b.iter(|| {
            let mut state =
                TextInputState::new("the quick brown fox jumps over the lazy dog ".repeat(10));
            for _ in 0..50 {
                state.move_word_right(false);
            }
            state
        })
    });
}

criterion::criterion_group!(
    benches,
    bench_text_insert,
    bench_text_select_all,
    bench_text_undo_redo,
    bench_text_movement
);
criterion::criterion_main!(benches);
