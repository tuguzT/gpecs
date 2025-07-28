use std::any::type_name;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use gpecs_soa::traits::{Soa, buffer_layout};
use gpecs_soa_bench::{
    Big, Large, Medium, Small, Tiny, Zero, names::*, with_capacity::WithCapacity,
};
use gpecs_soa_erased::erased::{BoxedErasedSoa, ErasedSoaContext};

fn with_capacity<T>(c: &mut Criterion)
where
    T: WithCapacity,
{
    const KB: usize = 1024;
    const CAPACITY_RANGE: [usize; 8] = [0, 1, 10, 100, KB, KB * 2, KB * 4, KB * 8];
    let group_name = format!("With capacity for `{}`", type_name::<T>());

    let mut group = c.benchmark_group(&group_name);
    for capacity in CAPACITY_RANGE {
        let context = ErasedSoaContext::of::<T>(&Default::default());
        let fields = <BoxedErasedSoa as Soa>::field_descriptors(&context);
        let buffer_layout = buffer_layout(fields, capacity).unwrap();
        let bytes = buffer_layout.size();
        group
            .throughput(Throughput::Bytes(bytes.try_into().unwrap()))
            .bench_with_input(
                BenchmarkId::new(SOA_SER_FUNCTION_NAME, capacity),
                &capacity,
                |b, &capacity| b.iter(|| T::soa_ser_with_capacity(capacity)),
            );
    }
    group.finish();

    let mut group = c.benchmark_group(&group_name);
    for capacity in CAPACITY_RANGE {
        let context = Default::default();
        let buffer_layout = buffer_layout(T::field_descriptors(&context), capacity).unwrap();
        let bytes = buffer_layout.size();
        group
            .throughput(Throughput::Bytes(bytes.try_into().unwrap()))
            .bench_with_input(
                BenchmarkId::new(SOA_SLF_FUNCTION_NAME, capacity),
                &capacity,
                |b, &capacity| b.iter(|| T::soa_slf_with_capacity(capacity)),
            );

        let bytes = T::field_descriptors(&context)
            .into_iter()
            .map(|desc| capacity * desc.as_ref().layout().size())
            .sum::<usize>();
        group
            .throughput(Throughput::Bytes(bytes.try_into().unwrap()))
            .bench_with_input(
                BenchmarkId::new(SOA_STD_FUNCTION_NAME, capacity),
                &capacity,
                |b, &capacity| b.iter(|| T::soa_std_with_capacity(capacity)),
            );

        let bytes = capacity * size_of::<T>();
        group
            .throughput(Throughput::Bytes(bytes.try_into().unwrap()))
            .bench_with_input(
                BenchmarkId::new(AOS_STD_FUNCTION_NAME, capacity),
                &capacity,
                |b, &capacity| b.iter(|| T::aos_std_with_capacity(capacity)),
            );
    }
    group.finish();
}

criterion_group!(
    with_capacity_benches,
    with_capacity::<Zero>,
    with_capacity::<Tiny>,
    with_capacity::<Small>,
    with_capacity::<Medium>,
    with_capacity::<Big>,
    with_capacity::<Large>,
);

criterion_main!(with_capacity_benches);
