use criterion::criterion_main;

mod common;

criterion_main!(
    common::with_capacity::benches,
    common::push::benches,
    common::work::benches,
);
