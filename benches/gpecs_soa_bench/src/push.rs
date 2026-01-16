use std::{alloc::Layout, hint::black_box, mem::MaybeUninit};

use arrayvec::ArrayVec;
use gpecs_soa_erased::{
    erased::{BoxedErasedSoa, ErasedSoa},
    soa::{
        field::FieldDescriptor,
        prelude::*,
        traits::{SoaWrite, TupleContext},
    },
    storage::AlignedUninitSlice,
};

use crate::{Big, Large, Medium, Small, Tiny, Zero, soa_vecs::SoaVecs};

pub trait Push: SoaVecs<Context: Default> + SoaWrite {
    fn soa_slf_push(vec: &mut SoaVec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = Default::default();
        let value = ErasedSoa::try_from(&context, black_box(value)).unwrap();
        vec.push(value);
    }

    fn soa_std_push(vecs: &mut Self::Vecs, value: Self);

    fn aos_std_push(vec: &mut Vec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }
}

type ArrayDescriptors<const CAP: usize> = ArrayVec<FieldDescriptor, CAP>;

impl Push for Zero {
    #[expect(clippy::let_unit_value, reason = "reference for other manual impls")]
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let value = black_box(value);
        vecs.push(value);
    }

    #[expect(clippy::let_unit_value, reason = "reference for other manual impls")]
    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = &Default::default();
        let value = black_box(value);

        let bytes = [MaybeUninit::<u8>::zeroed(); size_of::<Self>() * 2];
        let bytes = AlignedUninitSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<1>>::try_from_bytes_value(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_fields().into_ptrs();
            dst.copy_from(&ptrs, 1);
        });
    }
}

impl Push for Tiny {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (values,) = vecs;
        let (value,) = black_box(value);
        values.push(value);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = &TupleContext::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<1>>::try_from_bytes_value(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_fields().into_ptrs();
            dst.copy_from(&ptrs, 1);
        });
    }
}

impl Push for Small {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (xs, ys, zs) = vecs;
        let (x, y, z) = black_box(value);
        xs.push(x);
        ys.push(y);
        zs.push(z);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = &TupleContext::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<3>>::try_from_bytes_value(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_fields().into_ptrs();
            dst.copy_from(&ptrs, 1);
        });
    }
}

impl Push for Medium {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (a_s, b_s, c_s) = vecs;
        let (a, b, c) = black_box(value);
        a_s.push(a);
        b_s.push(b);
        c_s.push(c);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = &TupleContext::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<3>>::try_from_bytes_value(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_fields().into_ptrs();
            dst.copy_from(&ptrs, 1);
        });
    }
}

impl Push for Big {
    #[expect(clippy::many_single_char_names)]
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (a_s, b_s, c_s, d_s, e_s) = vecs;
        let (a, b, c, d, e) = black_box(value);
        a_s.push(a);
        b_s.push(b);
        c_s.push(c);
        d_s.push(d);
        e_s.push(e);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = &TupleContext::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<5>>::try_from_bytes_value(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_fields().into_ptrs();
            dst.copy_from(&ptrs, 1);
        });
    }
}

impl Push for Large {
    #[expect(clippy::many_single_char_names)]
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (a_s, b_s, c_s, d_s, e_s, f_s, g_s, h_s, i_s, j_s) = vecs;
        let (a, b, c, d, e, f, g, h, i, j) = black_box(value);
        a_s.push(a);
        b_s.push(b);
        c_s.push(c);
        d_s.push(d);
        e_s.push(e);
        f_s.push(f);
        g_s.push(g);
        h_s.push(h);
        i_s.push(i);
        j_s.push(j);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = &TupleContext::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<10>>::try_from_bytes_value(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_fields().into_ptrs();
            dst.copy_from(&ptrs, 1);
        });
    }
}
