use criterion::{criterion_group, criterion_main, Criterion};
use kjql::walker;
use serde_json::Value;

const DATA: &str = r#"{
   "array": [[[[["c", "a", "c"]]]], "g", [[["a", ["t"]]]]],
   "flatten-array": [[[[["c", "a", "c"]]]], "g", [[["a", ["t"]]]]],
   "props": { "a": { "b": { "c" : 777 } } },
   "nested-filter": [
      {
         "laptop": {
            "brand": "Apple"
         }
      },
      {
         "laptop": {
            "brand": "Asus"
         }
      }
   ]
}"#;

fn access_properties_benchmark(c: &mut Criterion) {
    let json: Value = serde_json::from_str(DATA).unwrap();

    let selector = Some(r#""props"."a"."b"."c""#);
    c.bench_function("Access properties", move |b| {
        b.iter(|| walker(&json, selector))
    });
}

fn filter_array_benchmark(c: &mut Criterion) {
    let json: Value = serde_json::from_str(DATA).unwrap();
    let selector = Some(r#""nested-filter"|"laptop"|"brand""#);
    c.bench_function("Filter an array", move |b| {
        b.iter(|| walker(&json, selector))
    });
}

fn flatten_array_benchmark(c: &mut Criterion) {
    let json: Value = serde_json::from_str(DATA).unwrap();

    let selector = Some(r#".."flatten-array""#);
    c.bench_function("Flatten an array", move |b| {
        b.iter(|| walker(&json, selector))
    });
}

fn range_array_benchmark(c: &mut Criterion) {
    let json: Value = serde_json::from_str(DATA).unwrap();
    let selector = Some(r#""array".[5:3]"#);
    c.bench_function("Get the range of an array", move |b| {
        b.iter(|| walker(&json, selector))
    });
}

fn group_benchmark(c: &mut Criterion) {
    let json: Value = serde_json::from_str(DATA).unwrap();
    let selector = Some(r#""array", "flatten-array", "props""#);
    c.bench_function("Get multiple groups", move |b| {
        b.iter(|| walker(&json, selector))
    });
}

criterion_group!(
    benches,
    flatten_array_benchmark,
    filter_array_benchmark,
    flatten_array_benchmark,
    range_array_benchmark,
    group_benchmark,
);

criterion_main!(benches);
