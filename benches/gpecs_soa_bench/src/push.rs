use std::{alloc::Layout, hint::black_box, mem::MaybeUninit, ops::Deref, slice};

use arrayvec::{ArrayVec, IntoIter};
use gpecs_soa_erased::{
    CovariantFieldLayouts, ErasedSoa,
    error::FromValueError,
    ptr::slice::CoreSliceItemPtrs,
    soa::{
        field::{FieldLayouts, FieldLayoutsOutput},
        layout::WithLayout,
        prelude::*,
        traits::SoaWrite,
    },
    storage::AlignedStorageSlice,
};

use crate::{Big, Large, Medium, Small, Tiny, Zero, soa_vecs::SoaVecs};

type Ptrs = CoreSliceItemPtrs<MaybeUninit<u8>>;
type BoxedErasedSoa = gpecs_soa_erased::BoxedErasedSoa<Ptrs>;
type ArrayErasedSoa<T, const CAP: usize> = ErasedSoa<T, ArrayLayouts<Layout, CAP>, Ptrs>;

pub trait Push: SoaVecs<Context: Default> + SoaWrite<Self> + Sized {
    fn soa_slf_push(vec: &mut SoaVec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }

    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = Self::Context::default();
        let value = BoxedErasedSoa::try_from::<Self, _>(&context, black_box(value))
            .map_err(FromValueError::into_source)
            .expect("failed to convert value into erased SoA");
        vec.push(value);
    }

    fn soa_std_push(vecs: &mut Self::Vecs, value: Self);

    fn aos_std_push(vec: &mut Vec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }
}

impl Push for Zero {
    #[expect(clippy::let_unit_value, reason = "reference for other manual impls")]
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let value = black_box(value);
        vecs.push(value);
    }

    #[expect(clippy::let_unit_value, reason = "reference for other manual impls")]
    fn soa_ser_push(vec: &mut SoaVec<BoxedErasedSoa>, value: Self) {
        let context = &Self::Context::default();
        let value = black_box(value);

        let bytes = [MaybeUninit::<u8>::zeroed(); size_of::<Self>() * 2];
        let bytes = AlignedStorageSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ArrayErasedSoa::<_, 1>::try_from_storage_value::<Self, _>(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_ptrs();
            dst.copy_from_forward(&ptrs, 1);
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
        let context = &Self::Context::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedStorageSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ArrayErasedSoa::<_, 1>::try_from_storage_value::<Self, _>(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_ptrs();
            dst.copy_from_forward(&ptrs, 1);
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
        let context = &Self::Context::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedStorageSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ArrayErasedSoa::<_, 3>::try_from_storage_value::<Self, _>(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_ptrs();
            dst.copy_from_forward(&ptrs, 1);
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
        let context = &Self::Context::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedStorageSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ArrayErasedSoa::<_, 3>::try_from_storage_value::<Self, _>(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_ptrs();
            dst.copy_from_forward(&ptrs, 1);
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
        let context = &Self::Context::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedStorageSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ArrayErasedSoa::<_, 5>::try_from_storage_value::<Self, _>(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_ptrs();
            dst.copy_from_forward(&ptrs, 1);
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
        let context = &Self::Context::default();
        let value = black_box(value);

        let mut bytes = [0_u8; size_of::<Self>() * 2];
        let bytes = unsafe {
            let (_, bytes, _) = bytes.align_to_mut::<Self>();
            let (_, bytes, _) = bytes.align_to_mut();
            bytes
        };

        let bytes = AlignedStorageSlice::new(bytes, Layout::new::<Self>()).unwrap();
        let value =
            ArrayErasedSoa::<_, 10>::try_from_storage_value::<Self, _>(bytes, context, value)
                .unwrap();

        vec.push_from(|_, mut dst| unsafe {
            let ptrs = value.as_ptrs();
            dst.copy_from_forward(&ptrs, 1);
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct ArrayLayouts<T, const CAP: usize>(ArrayVec<T, CAP>);

impl<T, const CAP: usize> Default for ArrayLayouts<T, CAP> {
    fn default() -> Self {
        Self(ArrayVec::default())
    }
}

impl<T, const CAP: usize> Deref for ArrayLayouts<T, CAP> {
    type Target = ArrayVec<T, CAP>;

    fn deref(&self) -> &Self::Target {
        let Self(array_vec) = self;
        array_vec
    }
}

impl<T, const CAP: usize> IntoIterator for ArrayLayouts<T, CAP> {
    type Item = T;
    type IntoIter = IntoIter<T, CAP>;

    fn into_iter(self) -> Self::IntoIter {
        let Self(array_vec) = self;
        array_vec.into_iter()
    }
}

impl<A, T, const CAP: usize> FromIterator<A> for ArrayLayouts<T, CAP>
where
    T: From<A>,
{
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        let array_vec = iter.into_iter().map(From::from).collect();
        Self(array_vec)
    }
}

impl<A, T, const CAP: usize> Extend<A> for ArrayLayouts<T, CAP>
where
    T: From<A>,
{
    fn extend<I: IntoIterator<Item = A>>(&mut self, iter: I) {
        let Self(array_vec) = self;
        array_vec.extend(iter.into_iter().map(From::from));
    }
}

impl<'a, T, const CAP: usize> FieldLayouts<'a> for ArrayLayouts<T, CAP>
where
    T: WithLayout + 'a,
{
    type Output = &'a [T];
    type OutputIter = slice::Iter<'a, T>;
    type OutputItem = &'a T;

    fn field_layouts(&'a self) -> Self::Output {
        self
    }
}

impl<T, const CAP: usize> CovariantFieldLayouts for ArrayLayouts<T, CAP>
where
    T: WithLayout + 'static,
{
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}
