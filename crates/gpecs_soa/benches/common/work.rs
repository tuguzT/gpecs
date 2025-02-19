use std::{any::type_name, array, convert::identity, hint::black_box, slice};

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa::{prelude::*, slice as soa_slice};

use super::*;

trait Work: Soa {
    fn work_item() -> Self;

    type SoaIter<'a>: Iterator + Clone;
    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_>;
    fn soa_work(iter: Self::SoaIter<'_>);

    type AosIter<'a>: Iterator + Clone;
    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_>;
    fn aos_work(iter: Self::AosIter<'_>);
}

impl Work for Tiny {
    #[inline]
    fn work_item() -> Self {
        (1,)
    }

    type SoaIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_> {
        data.iter()
    }

    fn soa_work(iter: Self::SoaIter<'_>) {
        let mut result = 0;
        for (i,) in iter {
            result += *i;
        }
        black_box(result);
    }

    type AosIter<'a> = slice::Iter<'a, Self>;

    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_> {
        data.iter()
    }

    fn aos_work(iter: Self::AosIter<'_>) {
        let mut result = 0;
        for (i,) in iter {
            result += *i;
        }
        black_box(result);
    }
}

impl Work for Small {
    fn work_item() -> Self {
        (1.0, 0.2, -2.3)
    }

    type SoaIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_> {
        data.iter()
    }

    fn soa_work(iter: Self::SoaIter<'_>) {
        let mut result = 0.0;
        for (x, y, _) in iter {
            result += *x + *y;
        }
        black_box(result);
    }

    type AosIter<'a> = slice::Iter<'a, Self>;

    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_> {
        data.iter()
    }

    fn aos_work(iter: Self::AosIter<'_>) {
        let mut result = 0.0;
        for (x, y, _) in iter {
            result += *x + *y;
        }
        black_box(result);
    }
}

impl Work for Big {
    fn work_item() -> Self {
        let small = Small::work_item();
        (
            small,
            small,
            array::from_fn(identity),
            "".to_owned(),
            "Hello, World".to_owned(),
        )
    }

    type SoaIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_> {
        data.iter()
    }

    fn soa_work(iter: Self::SoaIter<'_>) {
        let mut result = 0;
        for (index, (_, _, array, _, hello)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + hello.len();
        }
        black_box(result);
    }

    type AosIter<'a> = slice::Iter<'a, Self>;

    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_> {
        data.iter()
    }

    fn aos_work(iter: Self::AosIter<'_>) {
        let mut result = 0;
        for (index, (_, _, array, _, hello)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + hello.len();
        }
        black_box(result);
    }
}

impl Work for Large {
    fn work_item() -> Self {
        (
            Default::default(),
            array::from_fn(|_| 17),
            array::from_fn(|i| i.try_into().unwrap()),
            array::from_fn(|i| (i % 5 + 1).try_into().unwrap()),
            array::from_fn(|i| i.pow(2).try_into().unwrap()),
            Default::default(),
            array::from_fn(|_| 17),
            array::from_fn(|i| i.try_into().unwrap()),
            array::from_fn(|i| (i % 5 + 1).try_into().unwrap()),
            array::from_fn(|i| i.pow(2).try_into().unwrap()),
        )
    }

    type SoaIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_> {
        data.iter()
    }

    fn soa_work(iter: Self::SoaIter<'_>) {
        let mut result = 0;
        for (_, b, _, _, e, f, _, _, i, _) in iter {
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| acc - item << 3);
        }
        black_box(result);
    }

    type AosIter<'a> = slice::Iter<'a, Self>;

    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_> {
        data.iter()
    }

    fn aos_work(iter: Self::AosIter<'_>) {
        let mut result = 0;
        for (_, b, _, _, e, f, _, _, i, _) in iter {
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| acc - item << 3);
        }
        black_box(result);
    }
}

fn work<T>(c: &mut Criterion)
where
    T: Work,
{
    const COUNT_RANGE: [usize; 5] = [10, 100, 1_000, 10_000, 100_000];

    let mut group = c.benchmark_group(format!("Work for `{}`", type_name::<T>()));
    for count in COUNT_RANGE {
        let mut vec = SoaVec::<T>::with_capacity(count);
        for _ in 0..count {
            let value = black_box(T::work_item());
            vec.push(value);
        }
        let iter = T::soa_prepare_iter(&vec);
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, count),
            &count,
            |b, _| b.iter(|| T::soa_work(iter.clone())),
        );

        let mut vec = Vec::<T>::with_capacity(count);
        for _ in 0..count {
            let value = black_box(T::work_item());
            vec.push(value);
        }
        let iter = T::aos_prepare_iter(&vec);
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, count),
            &count,
            |b, _| b.iter(|| T::aos_work(iter.clone())),
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
