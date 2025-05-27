use std::hint::black_box;

use gpecs_soa::{prelude::*, traits::SoaVecs};
use gpecs_soa_erased::erased::{ErasedSoaContext, ErasedSoaVec};

use crate::{Big, Large, Medium, Small, Tiny, Zero};

pub trait WithCapacity: SoaVecs<Context: Default> {
    fn soa_slf_with_capacity(capacity: usize) -> SoaVec<Self> {
        let capacity = black_box(capacity);
        let context = Default::default();
        let vec = SoaVec::<Self>::with_context_and_capacity(context, capacity);
        black_box(vec)
    }

    fn soa_ser_with_capacity(capacity: usize) -> ErasedSoaVec {
        let capacity = black_box(capacity);
        let context = Default::default();
        let context = ErasedSoaContext::of::<Self>(&context);
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
