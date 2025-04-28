use std::{array, hint::black_box, iter::Zip, slice};

use gpecs_soa::{prelude::*, slice as soa_slice};

#[cfg(feature = "erased")]
use gpecs_soa_erased::erased::ErasedSoa;

use crate::{push::Push, with_capacity::WithCapacity, Big, Large, Small, Tiny};

pub trait Work: WithCapacity + Push {
    fn work_item(index: usize) -> Self;

    fn soa_slf_prepare_vec(count: usize) -> SoaVec<Self> {
        let mut vec = Self::soa_slf_with_capacity(count);
        for index in 0..count {
            let value = black_box(Self::work_item(index));
            Self::soa_slf_push(&mut vec, value);
        }
        black_box(vec)
    }

    type SoaSlfIter<'a>: Iterator + Clone;
    fn soa_slf_prepare_iter(data: SoaSlices<Self>) -> Self::SoaSlfIter<'_>;
    fn soa_slf_work(iter: Self::SoaSlfIter<'_>);

    #[cfg(feature = "erased")]
    fn soa_ser_prepare_vec(count: usize) -> SoaVec<ErasedSoa> {
        let mut vec = Self::soa_ser_with_capacity(count);
        for index in 0..count {
            let value = black_box(Self::work_item(index));
            let value = ErasedSoa::from(&Default::default(), value);
            Self::soa_ser_push(&mut vec, value);
        }
        black_box(vec)
    }

    #[cfg(feature = "erased")]
    type SoaSerIter<'a>: Iterator + Clone;
    #[cfg(feature = "erased")]
    fn soa_ser_prepare_iter(data: SoaSlices<ErasedSoa>) -> Self::SoaSerIter<'_>;
    #[cfg(feature = "erased")]
    fn soa_ser_work(iter: Self::SoaSerIter<'_>);

    fn soa_std_prepare_vec(count: usize) -> Self::Vecs {
        let mut vecs = Self::soa_std_with_capacity(count);
        for index in 0..count {
            let value = black_box(Self::work_item(index));
            Self::soa_std_push(&mut vecs, value);
        }
        black_box(vecs)
    }

    type SoaStdIter<'a>: Iterator + Clone;
    fn soa_std_prepare_iter(data: &Self::Vecs) -> Self::SoaStdIter<'_>;
    fn soa_std_work(iter: Self::SoaStdIter<'_>);

    fn aos_std_prepare_vec(count: usize) -> Vec<Self> {
        let mut vec = Self::aos_std_with_capacity(count);
        for index in 0..count {
            let value = black_box(Self::work_item(index));
            Self::aos_std_push(&mut vec, value);
        }
        black_box(vec)
    }

    type AosStdIter<'a>: Iterator + Clone;
    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_>;
    fn aos_std_work(iter: Self::AosStdIter<'_>);
}

impl Work for Tiny {
    #[inline]
    fn work_item(index: usize) -> Self {
        (index.try_into().unwrap(),)
    }

    type SoaSlfIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_slf_prepare_iter(data: SoaSlices<Self>) -> Self::SoaSlfIter<'_> {
        data.into_iter()
    }

    fn soa_slf_work(iter: Self::SoaSlfIter<'_>) {
        let mut result = 0;
        for (i,) in iter {
            result += *i;
        }
        black_box(result);
    }

    #[cfg(feature = "erased")]
    type SoaSerIter<'a> = soa_slice::Iter<'a, ErasedSoa>;

    #[cfg(feature = "erased")]
    fn soa_ser_prepare_iter(data: SoaSlices<ErasedSoa>) -> Self::SoaSerIter<'_> {
        data.into_iter()
    }

    #[cfg(feature = "erased")]
    fn soa_ser_work(iter: Self::SoaSerIter<'_>) {
        let mut result = 0;
        for refs in iter {
            let (i,) = unsafe { refs.into::<Self>(&()) }.unwrap();
            result += *i;
        }
        black_box(result);
    }

    type SoaStdIter<'a> = slice::Iter<'a, u32>;

    fn soa_std_prepare_iter(data: &Self::Vecs) -> Self::SoaStdIter<'_> {
        let (values,) = data;
        values.iter()
    }

    fn soa_std_work(iter: Self::SoaStdIter<'_>) {
        let mut result = 0;
        for i in iter {
            result += *i;
        }
        black_box(result);
    }

    type AosStdIter<'a> = slice::Iter<'a, Self>;

    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_> {
        data.iter()
    }

    fn aos_std_work(iter: Self::AosStdIter<'_>) {
        let mut result = 0;
        for (i,) in iter {
            result += *i;
        }
        black_box(result);
    }
}

impl Work for Small {
    fn work_item(index: usize) -> Self {
        let index = (index + 1) as f64;
        (1.0 * index, 0.2 * index, -2.3 * index)
    }

    type SoaSlfIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_slf_prepare_iter(data: SoaSlices<Self>) -> Self::SoaSlfIter<'_> {
        data.into_iter()
    }

    fn soa_slf_work(iter: Self::SoaSlfIter<'_>) {
        let mut result = 0.0;
        for (x, y, _) in iter {
            result += *x + *y;
        }
        black_box(result);
    }

    #[cfg(feature = "erased")]
    type SoaSerIter<'a> = soa_slice::Iter<'a, ErasedSoa>;

    #[cfg(feature = "erased")]
    fn soa_ser_prepare_iter(data: SoaSlices<ErasedSoa>) -> Self::SoaSerIter<'_> {
        data.into_iter()
    }

    #[cfg(feature = "erased")]
    fn soa_ser_work(iter: Self::SoaSerIter<'_>) {
        let mut result = 0.0;
        for refs in iter {
            let (x, y, _) = unsafe { refs.into::<Self>(&()) }.unwrap();
            result += *x + *y;
        }
        black_box(result);
    }

    type SoaStdIter<'a> =
        Zip<Zip<slice::Iter<'a, f64>, slice::Iter<'a, f64>>, slice::Iter<'a, f64>>;

    fn soa_std_prepare_iter(data: &Self::Vecs) -> Self::SoaStdIter<'_> {
        let (xs, ys, zs) = data;
        xs.iter().zip(ys.iter()).zip(zs.iter())
    }

    fn soa_std_work(iter: Self::SoaStdIter<'_>) {
        let mut result = 0.0;
        for ((x, y), _) in iter {
            result += *x + *y;
        }
        black_box(result);
    }

    type AosStdIter<'a> = slice::Iter<'a, Self>;

    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_> {
        data.iter()
    }

    fn aos_std_work(iter: Self::AosStdIter<'_>) {
        let mut result = 0.0;
        for (x, y, _) in iter {
            result += *x + *y;
        }
        black_box(result);
    }
}

impl Work for Big {
    fn work_item(index: usize) -> Self {
        let small = Small::work_item(index);
        (
            small,
            small,
            array::from_fn(|i| i + index),
            "".to_owned(),
            "Hello, World\n".to_owned(),
        )
    }

    type SoaSlfIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_slf_prepare_iter(data: SoaSlices<Self>) -> Self::SoaSlfIter<'_> {
        data.into_iter()
    }

    fn soa_slf_work(iter: Self::SoaSlfIter<'_>) {
        let mut result = 0;
        for (index, (_, _, array, _, str)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + str.len();
        }
        black_box(result);
    }

    #[cfg(feature = "erased")]
    type SoaSerIter<'a> = soa_slice::Iter<'a, ErasedSoa>;

    #[cfg(feature = "erased")]
    fn soa_ser_prepare_iter(data: SoaSlices<ErasedSoa>) -> Self::SoaSerIter<'_> {
        data.into_iter()
    }

    #[cfg(feature = "erased")]
    fn soa_ser_work(iter: Self::SoaSerIter<'_>) {
        let mut result = 0;
        for (index, refs) in iter.enumerate() {
            let (_, _, array, _, str) = unsafe { refs.into::<Self>(&()) }.unwrap();
            result += index + array.iter().sum::<usize>() + str.len();
        }
        black_box(result);
    }

    type SoaStdIter<'a> = Zip<
        Zip<
            Zip<Zip<slice::Iter<'a, Small>, slice::Iter<'a, Small>>, slice::Iter<'a, [usize; 18]>>,
            slice::Iter<'a, String>,
        >,
        slice::Iter<'a, String>,
    >;

    fn soa_std_prepare_iter(data: &Self::Vecs) -> Self::SoaStdIter<'_> {
        let (smalls1, smalls2, arrays, strs1, strs2) = data;
        smalls1
            .iter()
            .zip(smalls2.iter())
            .zip(arrays.iter())
            .zip(strs1.iter())
            .zip(strs2.iter())
    }

    fn soa_std_work(iter: Self::SoaStdIter<'_>) {
        let mut result = 0;
        for (index, ((((_, _), array), _), hello)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + hello.len();
        }
        black_box(result);
    }

    type AosStdIter<'a> = slice::Iter<'a, Self>;

    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_> {
        data.iter()
    }

    fn aos_std_work(iter: Self::AosStdIter<'_>) {
        let mut result = 0;
        for (index, (_, _, array, _, hello)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + hello.len();
        }
        black_box(result);
    }
}

impl Work for Large {
    fn work_item(index: usize) -> Self {
        (
            Default::default(),
            array::from_fn(|_| index.try_into().unwrap()),
            array::from_fn(|i| i.try_into().unwrap()),
            array::from_fn(|i| (i % 5 + 1).try_into().unwrap()),
            array::from_fn(|i| i.pow(2).try_into().unwrap()),
            Default::default(),
            array::from_fn(|_| index.try_into().unwrap()),
            array::from_fn(|i| i.try_into().unwrap()),
            array::from_fn(|i| (i % 5 + 1).try_into().unwrap()),
            array::from_fn(|i| i.pow(2).try_into().unwrap()),
        )
    }

    type SoaSlfIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_slf_prepare_iter(data: SoaSlices<Self>) -> Self::SoaSlfIter<'_> {
        data.into_iter()
    }

    fn soa_slf_work(iter: Self::SoaSlfIter<'_>) {
        let mut result = 0;
        for (_, b, _, _, e, f, _, _, i, _) in iter {
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| acc - item << 3);
        }
        black_box(result);
    }

    #[cfg(feature = "erased")]
    type SoaSerIter<'a> = soa_slice::Iter<'a, ErasedSoa>;

    #[cfg(feature = "erased")]
    fn soa_ser_prepare_iter(data: SoaSlices<ErasedSoa>) -> Self::SoaSerIter<'_> {
        data.into_iter()
    }

    #[cfg(feature = "erased")]
    fn soa_ser_work(iter: Self::SoaSerIter<'_>) {
        let mut result = 0;
        for refs in iter {
            let (_, b, _, _, e, f, _, _, i, _) = unsafe { refs.into::<Self>(&()) }.unwrap();
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| acc - item << 3);
        }
        black_box(result);
    }

    type SoaStdIter<'a> = Zip<
        Zip<
            Zip<
                Zip<
                    Zip<
                        Zip<
                            Zip<
                                Zip<
                                    Zip<slice::Iter<'a, [u32; 32]>, slice::Iter<'a, [u32; 32]>>,
                                    slice::Iter<'a, [u32; 32]>,
                                >,
                                slice::Iter<'a, [u32; 32]>,
                            >,
                            slice::Iter<'a, [u32; 32]>,
                        >,
                        slice::Iter<'a, [u32; 32]>,
                    >,
                    slice::Iter<'a, [u32; 32]>,
                >,
                slice::Iter<'a, [u32; 32]>,
            >,
            slice::Iter<'a, [u32; 32]>,
        >,
        slice::Iter<'a, [u32; 32]>,
    >;

    fn soa_std_prepare_iter(data: &Self::Vecs) -> Self::SoaStdIter<'_> {
        let (a, b, c, d, e, f, g, h, i, j) = data;
        a.iter()
            .zip(b.iter())
            .zip(c.iter())
            .zip(d.iter())
            .zip(e.iter())
            .zip(f.iter())
            .zip(g.iter())
            .zip(h.iter())
            .zip(i.iter())
            .zip(j.iter())
    }

    fn soa_std_work(iter: Self::SoaStdIter<'_>) {
        let mut result = 0;
        for (((((((((_, b), _), _), e), f), _), _), i), _) in iter {
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| acc - item << 3);
        }
        black_box(result);
    }

    type AosStdIter<'a> = slice::Iter<'a, Self>;

    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_> {
        data.iter()
    }

    fn aos_std_work(iter: Self::AosStdIter<'_>) {
        let mut result = 0;
        for (_, b, _, _, e, f, _, _, i, _) in iter {
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| acc - item << 3);
        }
        black_box(result);
    }
}
