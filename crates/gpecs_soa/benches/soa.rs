use criterion::criterion_main;

mod common;

criterion_main!(
    common::with_capacity::benches,
    common::push_many::benches,
    common::work::benches,
);
