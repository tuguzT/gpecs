use std::{alloc::Layout, hint::black_box, mem::MaybeUninit};

use arrayvec::ArrayVec;
use gpecs_soa::{prelude::*, traits::FieldDescriptor};
use gpecs_soa_erased::{
    aligned_bytes::AlignedUninitByteSlice,
    erased::{BoxedErasedSoa, ErasedSoa},
};

use crate::{Big, Large, Medium, Small, Tiny, Zero, soa_vecs::SoaVecs};

pub trait Push: SoaVecs<Context: Default> {
    fn soa_slf_push(vec: &mut SoaVec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = Default::default();
        let value = ErasedSoa::from_value(&context, black_box(value)).unwrap();
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
    #[allow(clippy::let_unit_value, reason = "reference for other manual impls")]
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let value = black_box(value);
        vecs.push(value);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = Default::default();
        let value = black_box(value);

        let bytes = [MaybeUninit::<u8>::zeroed(); size_of::<Self>() * 2];
        let bytes = AlignedUninitByteSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<1>>::from_bytes_value(bytes, &context, value).unwrap();

        vec.push_from(|_, dst| unsafe {
            let ptrs = value.as_refs().into_ptrs();
            dst.copy_from(&ptrs, 1)
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
        let context = Default::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitByteSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<1>>::from_bytes_value(bytes, &context, value).unwrap();

        vec.push_from(|_, dst| unsafe {
            let ptrs = value.as_refs().into_ptrs();
            dst.copy_from(&ptrs, 1)
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
        let context = Default::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitByteSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<3>>::from_bytes_value(bytes, &context, value).unwrap();

        vec.push_from(|_, dst| unsafe {
            let ptrs = value.as_refs().into_ptrs();
            dst.copy_from(&ptrs, 1)
        });
    }
}

impl Push for Medium {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (smalls1, smalls2, smalls3) = vecs;
        let (small1, small2, small3) = black_box(value);
        smalls1.push(small1);
        smalls2.push(small2);
        smalls3.push(small3);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = Default::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitByteSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<3>>::from_bytes_value(bytes, &context, value).unwrap();

        vec.push_from(|_, dst| unsafe {
            let ptrs = value.as_refs().into_ptrs();
            dst.copy_from(&ptrs, 1)
        });
    }
}

impl Push for Big {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (smalls1, smalls2, arrays, strs1, strs2) = vecs;
        let (small1, small2, array, str1, str2) = black_box(value);
        smalls1.push(small1);
        smalls2.push(small2);
        arrays.push(array);
        strs1.push(str1);
        strs2.push(str2);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = Default::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitByteSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<5>>::from_bytes_value(bytes, &context, value).unwrap();

        vec.push_from(|_, dst| unsafe {
            let ptrs = value.as_refs().into_ptrs();
            dst.copy_from(&ptrs, 1)
        });
    }
}

impl Push for Large {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (
            arrays1,
            arrays2,
            arrays3,
            arrays4,
            arrays5,
            arrays6,
            arrays7,
            arrays8,
            arrays9,
            arrays10,
        ) = vecs;
        let (array1, array2, array3, array4, array5, array6, array7, array8, array9, array10) =
            black_box(value);
        arrays1.push(array1);
        arrays2.push(array2);
        arrays3.push(array3);
        arrays4.push(array4);
        arrays5.push(array5);
        arrays6.push(array6);
        arrays7.push(array7);
        arrays8.push(array8);
        arrays9.push(array9);
        arrays10.push(array10);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = Default::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedUninitByteSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ErasedSoa::<_, ArrayDescriptors<10>>::from_bytes_value(bytes, &context, value).unwrap();

        vec.push_from(|_, dst| unsafe {
            let ptrs = value.as_refs().into_ptrs();
            dst.copy_from(&ptrs, 1)
        });
    }
}
