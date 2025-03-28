use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa::prelude::*;
use gpecs_soa_erased::erased::{ErasedSoa, ErasedSoaContext};

use super::*;

pub(super) trait WithCapacity: Soa<Context: Default + 'static> {
    fn soa_slf_with_capacity(capacity: usize) -> SoaVec<Self> {
        let capacity = black_box(capacity);
        let vec = SoaVec::<Self>::with_capacity(capacity);
        black_box(vec)
    }

    fn soa_ser_with_capacity(capacity: usize) -> SoaVec<ErasedSoa<Self::Fields>> {
        let capacity = black_box(capacity);
        let context = ErasedSoaContext::of::<Self>(&Default::default()).expect("should not fail");
        let vec = SoaVec::with_context_and_capacity(context, capacity);
        black_box(vec)
    }

    fn soa_std_with_capacity(capacity: usize) -> Self::Vecs;

    fn aos_std_with_capacity(capacity: usize) -> Vec<Self> {
        let capacity = black_box(capacity);
        let vec = Vec::<Self>::with_capacity(capacity);
        black_box(vec)
    }
}

impl WithCapacity for Zero {
    fn soa_std_with_capacity(capacity: usize) -> Self::Vecs {
        Self::aos_std_with_capacity(capacity)
    }
}

impl WithCapacity for Tiny {
    fn soa_std_with_capacity(capacity: usize) -> Self::Vecs {
        let capacity = black_box(capacity);
        let vecs = (Vec::with_capacity(capacity),);
        black_box(vecs)
    }
}

impl WithCapacity for Small {
    fn soa_std_with_capacity(capacity: usize) -> Self::Vecs {
        let capacity = black_box(capacity);
        let vecs = (
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
        );
        black_box(vecs)
    }
}

impl WithCapacity for Medium {
    fn soa_std_with_capacity(capacity: usize) -> Self::Vecs {
        let capacity = black_box(capacity);
        let vecs = (
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
        );
        black_box(vecs)
    }
}

impl WithCapacity for Big {
    fn soa_std_with_capacity(capacity: usize) -> Self::Vecs {
        let capacity = black_box(capacity);
        let vecs = (
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
        );
        black_box(vecs)
    }
}

impl WithCapacity for Large {
    fn soa_std_with_capacity(capacity: usize) -> Self::Vecs {
        let capacity = black_box(capacity);
        let vecs = (
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
        );
        black_box(vecs)
    }
}

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
