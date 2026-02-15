use std::{hint::black_box, mem::MaybeUninit};

use gpecs_soa_erased::{
    erased::{BoxedErasedSoa, ErasedSoaContext},
    ptr::slice::CoreSliceItemPtrs,
    soa::prelude::*,
};

use crate::{Big, Large, Medium, Small, Tiny, Zero, soa_vecs::SoaVecs};

pub trait WithCapacity: SoaVecs<Context: Default> + Sized {
    fn soa_slf_with_capacity(capacity: usize) -> SoaVec<Self> {
        let capacity = black_box(capacity);
        let context = Default::default();
        let vec = SoaVec::<Self>::with_context_and_capacity(context, capacity);
        black_box(vec)
    }

    fn soa_ser_with_capacity(
        capacity: usize,
    ) -> SoaVec<BoxedErasedSoa<CoreSliceItemPtrs<MaybeUninit<u8>>>> {
        let capacity = black_box(capacity);
        let context = Default::default();
        let context = ErasedSoaContext::of::<Self>(&context).expect("descriptors should be valid");
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
