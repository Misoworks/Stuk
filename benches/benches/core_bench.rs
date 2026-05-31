use stuk_core::{Element, ElementKind, Signal, reconcile, signal};

fn bench_reconcile_unchanged(criterion: &mut criterion::Criterion) {
    let old: Vec<Element> = (0..100)
        .map(|i| Element {
            key: Some(format!("item-{i}")),
            kind: ElementKind::Text(format!("Line {i}")),
            ..Default::default()
        })
        .collect();

    let new: Vec<Element> = (0..100)
        .map(|i| Element {
            key: Some(format!("item-{i}")),
            kind: ElementKind::Text(format!("Line {i}")),
            ..Default::default()
        })
        .collect();

    criterion.bench_function("reconcile/100_elements_unchanged", |b| {
        b.iter(|| reconcile(&old, &new))
    });
}

fn bench_reconcile_changed(criterion: &mut criterion::Criterion) {
    let old: Vec<Element> = (0..100)
        .map(|i| Element {
            key: Some(format!("item-{i}")),
            kind: ElementKind::Text(format!("Line {i}")),
            ..Default::default()
        })
        .collect();

    let new: Vec<Element> = (0..100)
        .map(|i| Element {
            key: Some(format!("item-{i}")),
            kind: ElementKind::Text(format!("Updated {i}")),
            ..Default::default()
        })
        .collect();

    criterion.bench_function("reconcile/100_elements_changed", |b| {
        b.iter(|| reconcile(&old, &new))
    });
}

fn bench_signal_operations(criterion: &mut criterion::Criterion) {
    criterion.bench_function("signal/set_1000_times", |b| {
        let mut value = signal(0i32);
        b.iter(|| {
            for i in 0..1000 {
                value.set(i);
            }
        })
    });

    criterion.bench_function("signal/get_1000_times", |b| {
        let value = signal(42i32);
        b.iter(|| {
            for _ in 0..1000 {
                let _ = value.get();
            }
        })
    });
}

criterion::criterion_group!(
    benches,
    bench_reconcile_unchanged,
    bench_reconcile_changed,
    bench_signal_operations
);
criterion::criterion_main!(benches);
