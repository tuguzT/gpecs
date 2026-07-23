use core::{
    alloc::{Layout, LayoutError},
    array,
    marker::PhantomData,
    ptr,
};

use crate::traits::{AllocSoaContext, AllocSoaTrusted, FieldLayouts};

// https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#enum-counting
#[macro_export]
#[doc(hidden)]
macro_rules! count_idents {
    ($($idents:ident),* $(,)*) => {
        {
            #[expect(dead_code)]
            #[repr(usize)]
            enum Idents { $($idents,)* __CountIdentsLast }

            const COUNT: usize = Idents::__CountIdentsLast as usize;
            COUNT
        }
    };
}

#[doc(hidden)]
pub use count_idents;

/// Helper type for [SoA](super::RawSoa) implementation of [tuples](prim@tuple).
pub struct TupleHelper<T>(PhantomData<fn() -> T>);

#[inline]
#[must_use]
#[doc(hidden)]
pub const fn permutation<const N: usize>() -> [usize; N] {
    let mut permutation = [0; _];
    let mut i = 0;
    while i < permutation.len() {
        permutation[i] = i;
        i += 1;
    }
    permutation
}

#[inline]
#[must_use]
#[doc(hidden)]
pub const fn layout_permutation<const N: usize>(layouts: [Layout; N]) -> [usize; N] {
    let mut permutation = permutation();
    let mut i = 1;
    while i < permutation.len() {
        let mut j = i;
        while j > 0 && layouts[permutation[j - 1]].align() > layouts[permutation[j]].align() {
            permutation.swap(j - 1, j);
            j -= 1;
        }
        i += 1;
    }
    permutation
}

macro_rules! soa_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        impl<$($types,)*> TupleHelper<($($types,)*)> {
            pub const SIZE: usize = count_idents!($($types,)*);
            pub const PERMUTATION: [usize; count_idents!($($types,)*)] = {
                let layouts = [$(Layout::new::<$types>(),)*];
                layout_permutation(layouts)
            };
            pub const FIELD_LAYOUTS: [Layout; count_idents!($($types,)*)] = {
                let permutation = Self::PERMUTATION;
                let layouts = [$(Layout::new::<$types>(),)*];
                [$(layouts[permutation[$indices]],)*]
            };
        }

        impl<'a, $($types,)*> FieldLayouts<'a, ($($types,)*)> for () {
            type Output = [Layout; count_idents!($($types,)*)];
            type OutputIter = array::IntoIter<Layout, { count_idents!($($types,)*) }>;
            type OutputItem = Layout;

            #[inline]
            fn field_layouts(&'a self) -> Self::Output {
                TupleHelper::<($($types,)*)>::FIELD_LAYOUTS
            }
        }

        unsafe impl<$($types,)*> AllocSoaContext<($($types,)*)> for () {
            #[inline]
            fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let regions = [$(Layout::array::<$types>(capacity)?,)*];
                $((layout, _) = layout.extend(regions[permutation[$indices]])?;)*

                Ok(layout)
            }

            #[inline]
            unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let mut offsets = [0; count_idents!($($types,)*)];

                let regions = unsafe { [$(Layout::array::<$types>(capacity).unwrap_unchecked(),)*] };
                $((layout, offsets[permutation[$indices]]) = unsafe { layout.extend(regions[permutation[$indices]]).unwrap_unchecked() };)*
                let _ = layout;

                let ptrs = unsafe { ($(buffer.add(offsets[$indices]).cast(),)*) };
                ptrs
            }

            #[inline]
            unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let mut offsets = [0; count_idents!($($types,)*)];

                let regions = unsafe { [$(Layout::array::<$types>(capacity).unwrap_unchecked(),)*] };
                $((layout, offsets[permutation[$indices]]) = unsafe { layout.extend(regions[permutation[$indices]]).unwrap_unchecked() };)*
                let _ = layout;

                let ptrs = unsafe { ($(buffer.add(offsets[$indices]).cast(),)*) };
                ptrs
            }

            #[inline]
            unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, count) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, count) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in (0..count_idents!($($types,)*)).rev() {
                    closures[permutation[index]]();
                }
            }
        }

        unsafe impl<$($types,)*> AllocSoaTrusted for ($($types,)*) {}
    };
}

soa_tuple_impl!(
    A index 0,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
    K index 10,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
    K index 10,
    L index 11,
);
