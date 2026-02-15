use std::{array, hint::black_box, iter::Zip, mem::MaybeUninit, slice};

use gpecs_soa_erased::{
    erased::BoxedErasedSoa,
    ptr::slice::CoreSliceItemPtrs,
    soa::{prelude::*, slice as soa_slice, traits::TupleContext},
};
use num_traits::ToPrimitive;

use crate::{Big, Large, Small, Tiny, push::Push, with_capacity::WithCapacity};

pub trait Work: WithCapacity + Push {
    type Output;

    fn work_item(index: usize) -> Self;

    fn soa_slf_prepare_vec(count: usize) -> SoaVec<Self> {
        let mut vec = Self::soa_slf_with_capacity(count);
        for index in 0..count {
            let value = black_box(Self::work_item(index));
            Self::soa_slf_push(&mut vec, value);
        }
        black_box(vec)
    }

    type SoaSlfIter<'ctx, 'a>: Iterator + Clone;

    fn soa_slf_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, Self>,
    ) -> Self::SoaSlfIter<'ctx, 'a>;

    fn soa_slf_work(iter: Self::SoaSlfIter<'_, '_>) -> Self::Output;

    fn soa_ser_prepare_vec(
        count: usize,
    ) -> SoaVec<BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>> {
        let mut vec = Self::soa_ser_with_capacity(count);
        for index in 0..count {
            let value = black_box(Self::work_item(index));
            Self::soa_ser_push(&mut vec, value);
        }
        black_box(vec)
    }

    type SoaSerIter<'ctx, 'a>: Iterator + Clone;

    fn soa_ser_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>,
    ) -> Self::SoaSerIter<'ctx, 'a>;

    fn soa_ser_work(iter: Self::SoaSerIter<'_, '_>) -> Self::Output;

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

    fn soa_std_work(iter: Self::SoaStdIter<'_>) -> Self::Output;

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

    fn aos_std_work(iter: Self::AosStdIter<'_>) -> Self::Output;
}

impl Work for Tiny {
    type Output = u32;

    #[inline]
    fn work_item(index: usize) -> Self {
        (index.try_into().unwrap(),)
    }

    type SoaSlfIter<'ctx, 'a> = soa_slice::Iter<'ctx, 'a, Self>;

    fn soa_slf_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, Self>,
    ) -> Self::SoaSlfIter<'ctx, 'a> {
        data.into_iter()
    }

    fn soa_slf_work(iter: Self::SoaSlfIter<'_, '_>) -> Self::Output {
        let mut result = 0;
        for (i,) in iter {
            result += *i;
        }
        black_box(result)
    }

    type SoaSerIter<'ctx, 'a> =
        soa_slice::Iter<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>;

    fn soa_ser_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>,
    ) -> Self::SoaSerIter<'ctx, 'a> {
        data.into_iter()
    }

    fn soa_ser_work(iter: Self::SoaSerIter<'_, '_>) -> Self::Output {
        let context = TupleContext::default();
        let mut result = 0;
        for refs in iter {
            let (i,) = unsafe { refs.downcast::<Self>(&context) }.unwrap();
            result += *i;
        }
        black_box(result)
    }

    type SoaStdIter<'a> = slice::Iter<'a, u32>;

    fn soa_std_prepare_iter(data: &Self::Vecs) -> Self::SoaStdIter<'_> {
        let (values,) = data;
        values.iter()
    }

    fn soa_std_work(iter: Self::SoaStdIter<'_>) -> Self::Output {
        let mut result = 0;
        for i in iter {
            result += *i;
        }
        black_box(result)
    }

    type AosStdIter<'a> = slice::Iter<'a, Self>;

    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_> {
        data.iter()
    }

    fn aos_std_work(iter: Self::AosStdIter<'_>) -> Self::Output {
        let mut result = 0;
        for (i,) in iter {
            result += *i;
        }
        black_box(result)
    }
}

impl Work for Small {
    type Output = f64;

    fn work_item(index: usize) -> Self {
        let index = (index + 1)
            .to_f64()
            .expect("index should be convertible to f64");
        (1.0 * index, 0.2 * index, -2.3 * index)
    }

    type SoaSlfIter<'ctx, 'a> = soa_slice::Iter<'ctx, 'a, Self>;

    fn soa_slf_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, Self>,
    ) -> Self::SoaSlfIter<'ctx, 'a> {
        data.into_iter()
    }

    fn soa_slf_work(iter: Self::SoaSlfIter<'_, '_>) -> Self::Output {
        let mut result = 0.0;
        for (x, y, _) in iter {
            result += *x + *y;
        }
        black_box(result)
    }

    type SoaSerIter<'ctx, 'a> =
        soa_slice::Iter<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>;

    fn soa_ser_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>,
    ) -> Self::SoaSerIter<'ctx, 'a> {
        data.into_iter()
    }

    fn soa_ser_work(iter: Self::SoaSerIter<'_, '_>) -> Self::Output {
        let context = TupleContext::default();
        let mut result = 0.0;
        for refs in iter {
            let (x, y, _) = unsafe { refs.downcast::<Self>(&context) }.unwrap();
            result += *x + *y;
        }
        black_box(result)
    }

    type SoaStdIter<'a> =
        Zip<Zip<slice::Iter<'a, f64>, slice::Iter<'a, f64>>, slice::Iter<'a, f64>>;

    fn soa_std_prepare_iter(data: &Self::Vecs) -> Self::SoaStdIter<'_> {
        let (xs, ys, zs) = data;
        xs.iter().zip(ys.iter()).zip(zs.iter())
    }

    fn soa_std_work(iter: Self::SoaStdIter<'_>) -> Self::Output {
        let mut result = 0.0;
        for ((x, y), _) in iter {
            result += *x + *y;
        }
        black_box(result)
    }

    type AosStdIter<'a> = slice::Iter<'a, Self>;

    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_> {
        data.iter()
    }

    fn aos_std_work(iter: Self::AosStdIter<'_>) -> Self::Output {
        let mut result = 0.0;
        for (x, y, _) in iter {
            result += *x + *y;
        }
        black_box(result)
    }
}

impl Work for Big {
    type Output = usize;

    fn work_item(index: usize) -> Self {
        let small = Small::work_item(index);
        (
            small,
            small,
            array::from_fn(|i| i + index),
            String::new(),
            "Hello, World\n".to_owned(),
        )
    }

    type SoaSlfIter<'ctx, 'a> = soa_slice::Iter<'ctx, 'a, Self>;

    fn soa_slf_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, Self>,
    ) -> Self::SoaSlfIter<'ctx, 'a> {
        data.into_iter()
    }

    fn soa_slf_work(iter: Self::SoaSlfIter<'_, '_>) -> Self::Output {
        let mut result = 0;
        for (index, (_, _, array, _, str)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + str.len();
        }
        black_box(result)
    }

    type SoaSerIter<'ctx, 'a> =
        soa_slice::Iter<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>;

    fn soa_ser_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>,
    ) -> Self::SoaSerIter<'ctx, 'a> {
        data.into_iter()
    }

    fn soa_ser_work(iter: Self::SoaSerIter<'_, '_>) -> Self::Output {
        let context = TupleContext::default();
        let mut result = 0;
        for (index, refs) in iter.enumerate() {
            let (_, _, array, _, str) = unsafe { refs.downcast::<Self>(&context) }.unwrap();
            result += index + array.iter().sum::<usize>() + str.len();
        }
        black_box(result)
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

    fn soa_std_work(iter: Self::SoaStdIter<'_>) -> Self::Output {
        let mut result = 0;
        for (index, ((((_, _), array), _), hello)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + hello.len();
        }
        black_box(result)
    }

    type AosStdIter<'a> = slice::Iter<'a, Self>;

    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_> {
        data.iter()
    }

    fn aos_std_work(iter: Self::AosStdIter<'_>) -> Self::Output {
        let mut result = 0;
        for (index, (_, _, array, _, hello)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + hello.len();
        }
        black_box(result)
    }
}

impl Work for Large {
    type Output = u32;

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

    type SoaSlfIter<'ctx, 'a> = soa_slice::Iter<'ctx, 'a, Self>;

    fn soa_slf_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, Self>,
    ) -> Self::SoaSlfIter<'ctx, 'a> {
        data.into_iter()
    }

    fn soa_slf_work(iter: Self::SoaSlfIter<'_, '_>) -> Self::Output {
        let mut result = 0;
        for (_, b, _, _, e, f, _, _, i, _) in iter {
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| (acc - item) << 3);
        }
        black_box(result)
    }

    type SoaSerIter<'ctx, 'a> =
        soa_slice::Iter<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>;

    fn soa_ser_prepare_iter<'ctx, 'a>(
        data: SoaSlices<'ctx, 'a, BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>>,
    ) -> Self::SoaSerIter<'ctx, 'a> {
        data.into_iter()
    }

    fn soa_ser_work(iter: Self::SoaSerIter<'_, '_>) -> Self::Output {
        let context = TupleContext::default();
        let mut result = 0;
        for refs in iter {
            let (_, b, _, _, e, f, _, _, i, _) =
                unsafe { refs.downcast::<Self>(&context) }.unwrap();
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| (acc - item) << 3);
        }
        black_box(result)
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

    #[expect(clippy::many_single_char_names)]
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

    fn soa_std_work(iter: Self::SoaStdIter<'_>) -> Self::Output {
        let mut result = 0;
        for (((((((((_, b), _), _), e), f), _), _), i), _) in iter {
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| (acc - item) << 3);
        }
        black_box(result)
    }

    type AosStdIter<'a> = slice::Iter<'a, Self>;

    fn aos_std_prepare_iter(data: &[Self]) -> Self::AosStdIter<'_> {
        data.iter()
    }

    fn aos_std_work(iter: Self::AosStdIter<'_>) -> Self::Output {
        let mut result = 0;
        for (_, b, _, _, e, f, _, _, i, _) in iter {
            result += b.iter().max().unwrap() + e.iter().sum::<u32>()
                - f.iter().min().unwrap()
                - i.iter().fold(u32::MAX, |acc, item| (acc - item) << 3);
        }
        black_box(result)
    }
}
