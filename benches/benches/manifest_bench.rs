use stuk_manifest::{parse, validate};

fn bench_manifest_parse(criterion: &mut criterion::Criterion) {
    let source = r#"
[app]
id = "com.example.notes"
name = "Notes"
version = "0.1.0"

[window.main]
title = "Notes"
width = 1100
height = 760
material = "maris"
chrome = "compact"

[permissions]
network = false
notifications = true

[actions.notes.new]
label = "New Note"
shortcut = "Ctrl+N"

[settings.appearance.theme]
type = "enum"
label = "Theme"
values = ["system", "light", "dark"]
default = "system"
"#;

    criterion.bench_function("manifest/parse_typical", |b| b.iter(|| parse(source)));

    criterion.bench_function("manifest/validate_typical", |b| {
        let manifest = parse(source).unwrap();
        b.iter(|| validate(&manifest))
    });
}

criterion::criterion_group!(benches, bench_manifest_parse);
criterion::criterion_main!(benches);
