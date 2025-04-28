use std::any::type_name;

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa_bench::{with_capacity::WithCapacity, Big, Large, Medium, Small, Tiny, Zero};

use super::names::*;

fn with_capacity<T>(c: &mut Criterion)
where
    T: WithCapacity,
{
    const KB: usize = 1024;
    const CAPACITY_RANGE: [usize; 8] = [0, 1, 10, 100, KB, KB * 2, KB * 4, KB * 8];

    let mut group = c.benchmark_group(format!("With capacity for `{}`", type_name::<T>()));
    for capacity in CAPACITY_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_SLF_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| T::soa_slf_with_capacity(capacity)),
        );
        #[cfg(feature = "erased")]
        group.bench_with_input(
            BenchmarkId::new(SOA_SER_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| T::soa_ser_with_capacity(capacity)),
        );
        group.bench_with_input(
            BenchmarkId::new(SOA_STD_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| T::soa_std_with_capacity(capacity)),
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_STD_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| T::aos_std_with_capacity(capacity)),
        );
    }
}

criterion_group!(
    benches,
    with_capacity::<Zero>,
    with_capacity::<Tiny>,
    with_capacity::<Small>,
    with_capacity::<Medium>,
    with_capacity::<Big>,
    with_capacity::<Large>,
);
