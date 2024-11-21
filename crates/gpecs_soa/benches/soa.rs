use std::{any::type_name_of_val, hint::black_box};

use criterion::{criterion_group, criterion_main, Criterion};
use gpecs_soa::prelude::*;

type Zero = ();
type Tiny = (u32,);
type Small = (f64, f64, f64);
type Medium = (Small, Small, Small);
type Big = (Small, Small, [usize; 18], String, String);
type Large = ([u64; 64],);

fn with_capacity<T, const CAP: usize>()
where
    T: Soa,
{
    black_box(SoaVec::<T>::with_capacity(black_box(CAP)));
}

fn bench_with_capacity<T, const CAP: usize>(c: &mut Criterion)
where
    T: Soa,
{
    let function = with_capacity::<T, CAP>;
    let id = type_name_of_val(&function);
    c.bench_function(id, |b| b.iter(function));
}

criterion_group!(
    benches,
    bench_with_capacity::<Zero, 100>,
    bench_with_capacity::<Tiny, 100>,
    bench_with_capacity::<Small, 100>,
    bench_with_capacity::<Medium, 100>,
    bench_with_capacity::<Big, 100>,
    bench_with_capacity::<Large, 100>,
);

criterion_main!(benches);
