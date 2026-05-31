use stuk_layout::{
    self, Axis, EdgeInsets, FlexItem, FlexLayout, GridItem, GridLayout, Length, Rect, Size,
    stack_size,
};

fn bench_stack_size(criterion: &mut criterion::Criterion) {
    let children: Vec<Size> = (0..100)
        .map(|i| Size {
            width: 100.0 + i as f32,
            height: 30.0,
        })
        .collect();
    let padding = EdgeInsets {
        top: 12.0,
        left: 12.0,
        bottom: 12.0,
        right: 12.0,
    };

    criterion.bench_function("stack_size/100_items_vertical", |b| {
        b.iter(|| stack_size(Axis::Vertical, padding, 8.0, &children))
    });

    criterion.bench_function("stack_size/100_items_horizontal", |b| {
        b.iter(|| stack_size(Axis::Horizontal, padding, 8.0, &children))
    });
}

fn bench_flex_layout(criterion: &mut criterion::Criterion) {
    let layout = FlexLayout::column()
        .padding(EdgeInsets {
            top: 16.0,
            left: 16.0,
            bottom: 16.0,
            right: 16.0,
        })
        .gap(8.0);

    let items: Vec<FlexItem> = (0..50)
        .map(|i| {
            FlexItem::new(Size {
                width: 200.0,
                height: 40.0 + i as f32 * 0.5,
            })
        })
        .collect();

    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 800.0,
    };

    criterion.bench_function("flex_layout/50_items_column", |b| {
        b.iter(|| stuk_layout::flex_layout(layout.clone(), bounds, &items))
    });
}

fn bench_grid_layout(criterion: &mut criterion::Criterion) {
    let layout = GridLayout::new(
        vec![
            stuk_layout::GridTrack::fixed(200.0),
            stuk_layout::GridTrack::fixed(200.0),
            stuk_layout::GridTrack::Fit,
        ],
        vec![
            stuk_layout::GridTrack::fixed(40.0),
            stuk_layout::GridTrack::fixed(40.0),
        ],
    )
    .gap(8.0)
    .padding(EdgeInsets {
        top: 12.0,
        left: 12.0,
        bottom: 12.0,
        right: 12.0,
    });

    let items: Vec<GridItem> = (0..6)
        .map(|i| {
            GridItem::new(
                i % 3,
                i / 3,
                Size {
                    width: 180.0,
                    height: 36.0,
                },
            )
        })
        .collect();

    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 640.0,
        height: 480.0,
    };

    criterion.bench_function("grid_layout/6_items_3col_2row", |b| {
        b.iter(|| stuk_layout::grid_layout(&layout, bounds, &items))
    });
}

criterion::criterion_group!(
    benches,
    bench_stack_size,
    bench_flex_layout,
    bench_grid_layout
);
criterion::criterion_main!(benches);
