use std::any::type_name;

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa_bench::{work::Work, Big, Large, Small, Tiny};

use super::names::*;

fn work<T>(c: &mut Criterion)
where
    T: Work,
{
    const COUNT_RANGE: [usize; 5] = [10, 100, 1_000, 10_000, 100_000];

    let mut group = c.benchmark_group(format!("Work for `{}`", type_name::<T>()));
    for count in COUNT_RANGE {
        let vec = T::soa_slf_prepare_vec(count);
        let iter = T::soa_slf_prepare_iter(vec.slices());
        group.bench_with_input(
            BenchmarkId::new(SOA_SLF_FUNCTION_NAME, count),
            &count,
            |b, _| b.iter(|| T::soa_slf_work(iter.clone())),
        );

        let vec = T::soa_ser_prepare_vec(count);
        let iter = T::soa_ser_prepare_iter(vec.slices());
        group.bench_with_input(
            BenchmarkId::new(SOA_SER_FUNCTION_NAME, count),
            &count,
            |b, _| b.iter(|| T::soa_ser_work(iter.clone())),
        );

        let vec = T::soa_std_prepare_vec(count);
        let iter = T::soa_std_prepare_iter(&vec);
        group.bench_with_input(
            BenchmarkId::new(SOA_STD_FUNCTION_NAME, count),
            &count,
            |b, _| b.iter(|| T::soa_std_work(iter.clone())),
        );

        let vec = T::aos_std_prepare_vec(count);
        let iter = T::aos_std_prepare_iter(&vec);
        group.bench_with_input(
            BenchmarkId::new(AOS_STD_FUNCTION_NAME, count),
            &count,
            |b, _| b.iter(|| T::aos_std_work(iter.clone())),
        );
    }
}

criterion_group!(
    benches,
    work::<Tiny>,
    work::<Small>,
    // work::<Medium>,
    work::<Big>,
    work::<Large>,
);
