use core::{
    alloc::{Layout, LayoutError},
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::{
    field::FieldDescriptor,
    traits::{
        AllocSoaContext, AllocSoaTrusted, CloneToUninitSoaContext, FieldDescriptors, RawSoa,
        RawSoaContext, ReadSoaContext, SoaContext, WriteSoaContext,
    },
};

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

/// Helper type for [SoA](RawSoa) implementation of [tuples](prim@tuple).
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
            pub const FIELD_DESCRIPTORS: [FieldDescriptor; count_idents!($($types,)*)] = {
                let permutation = Self::PERMUTATION;
                let descriptors = [$(FieldDescriptor::of::<$types>(),)*];
                [$(descriptors[permutation[$indices]],)*]
            };
        }

        unsafe impl<$($types,)*> RawSoaContext<($($types,)*)> for () {
            type Ptrs<'a> = ($(*const $types,)*);

            #[inline]
            fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
                from
            }

            #[inline]
            fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
                let ptrs = ($(ptr::dangling::<$types>(),)*);
                ptrs
            }

            #[inline]
            unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
                let ptrs = unsafe { ($(ptrs.$indices.add(offset),)*) };
                ptrs
            }

            #[inline]
            unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            type MutPtrs<'a> = ($(*mut $types,)*);

            #[inline]
            fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
                from
            }

            #[inline]
            fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
                let ptrs = ($(ptr::dangling_mut::<$types>(),)*);
                ptrs
            }

            #[inline]
            unsafe fn ptrs_add_mut<'a>(
                &'a self,
                ptrs: Self::MutPtrs<'a>,
                offset: usize,
            ) -> Self::MutPtrs<'a> {
                let ptrs = unsafe { ($(ptrs.$indices.add(offset),)*) };
                ptrs
            }

            #[inline]
            unsafe fn ptrs_offset_from_mut(
                &self,
                ptrs: Self::MutPtrs<'_>,
                origin: Self::Ptrs<'_>,
            ) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline]
            fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
                let ptrs = ($(ptrs.$indices.cast_const(),)*);
                ptrs
            }

            #[inline]
            fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
                let ptrs = ($(ptrs.$indices.cast_mut(),)*);
                ptrs
            }

            #[inline]
            unsafe fn ptrs_swap(&self, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::swap(a.$indices, b.$indices) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in (0..count_idents!($($types,)*)).rev() {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_nonoverlapping(
                &self,
                src: Self::Ptrs<'_>,
                dst: Self::MutPtrs<'_>,
                len: usize,
            ) {
                // because source and destination are non-overlapping, we can copy them in any order
                unsafe { $(ptr::copy_nonoverlapping(src.$indices, dst.$indices, len);)* }
            }

            #[inline]
            unsafe fn ptrs_drop_in_place(&self, ptrs: Self::MutPtrs<'_>) {
                unsafe { $(ptr::drop_in_place(ptrs.$indices);)* }
            }

            type NonNullPtrs<'a> = ($(NonNull<$types>,)*);

            #[inline]
            fn upcast_nonnull_ptrs<'short, 'long: 'short>(
                from: Self::NonNullPtrs<'long>,
            ) -> Self::NonNullPtrs<'short> {
                from
            }

            #[inline]
            unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
                let ptrs = unsafe { ($(NonNull::new_unchecked(ptrs.$indices),)*) };
                ptrs
            }

            #[inline]
            fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
                let ptrs = ($(ptrs.$indices.as_ptr(),)*);
                ptrs
            }

            type SlicePtrs<'a> = ($(*const [$types],)*);

            #[inline]
            fn upcast_slice_ptrs<'short, 'long: 'short>(
                from: Self::SlicePtrs<'long>,
            ) -> Self::SlicePtrs<'short> {
                from
            }

            #[inline]
            fn slice_ptrs_from_raw_parts<'a>(
                &'a self,
                ptrs: Self::Ptrs<'a>,
                len: usize,
            ) -> Self::SlicePtrs<'a> {
                let slices = ($(ptr::slice_from_raw_parts(ptrs.$indices, len),)*);
                slices
            }

            #[inline]
            fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
                let slices = ($(slices.$indices.cast(),)*); // should be `slices.$indices.as_ptr()` but it's unstable
                slices
            }

            type SliceMutPtrs<'a> = ($(*mut [$types],)*);

            #[inline]
            fn upcast_mut_slice_ptrs<'short, 'long: 'short>(
                from: Self::SliceMutPtrs<'long>,
            ) -> Self::SliceMutPtrs<'short> {
                from
            }

            #[inline]
            fn mut_slice_ptrs_from_raw_parts<'a>(
                &'a self,
                ptrs: Self::MutPtrs<'a>,
                len: usize,
            ) -> Self::SliceMutPtrs<'a> {
                let slices = ($(ptr::slice_from_raw_parts_mut(ptrs.$indices, len),)*);
                slices
            }

            #[inline]
            fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
                let slices = ($(slices.$indices.cast(),)*); // should be `slices.$indices.as_mut_ptr()` but it's unstable
                slices
            }

            #[inline]
            fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
                let slices = ($(slices.$indices.cast_const(),)*);
                slices
            }

            #[inline]
            fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
                let slices = ($(slices.$indices.cast_mut(),)*);
                slices
            }

            #[inline]
            unsafe fn slices_drop_in_place(&self, slices: Self::SliceMutPtrs<'_>) {
                unsafe { $(ptr::drop_in_place(slices.$indices);)* }
            }
        }

        unsafe impl<$($types,)*> RawSoa for ($($types,)*) {
            type Context = ();
            type Fields = ($($types,)*);
        }

        unsafe impl<$($types,)*> CloneToUninitSoaContext<($($types,)*)> for ()
        where
            $($types: Clone,)*
        {
            #[inline]
            unsafe fn clone_to_uninit(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>) {
                let src = unsafe { ($(&*src.$indices,)*) };
                unsafe { $(ptr::write(dst.$indices, src.$indices.clone());)* }
            }
        }

        unsafe impl<'a, $($types,)*> ReadSoaContext<'a, ($($types,)*), ($($types,)*)> for () {
            #[inline]
            unsafe fn read(&'a self, ptrs: Self::Ptrs<'a>) -> ($($types,)*) {
                unsafe { ($(ptr::read(ptrs.$indices),)*) }
            }
        }

        unsafe impl<$($types,)*> WriteSoaContext<($($types,)*), ($($types,)*)> for () {
            #[inline]
            unsafe fn write(&self, dst: Self::MutPtrs<'_>, value: ($($types,)*)) {
                unsafe { $(ptr::write(dst.$indices, value.$indices);)* }
            }
        }

        impl<'a, $($types,)*> FieldDescriptors<'a, ($($types,)*)> for () {
            type Output = [FieldDescriptor; count_idents!($($types,)*)];

            #[inline]
            fn field_descriptors(&'a self) -> Self::Output {
                TupleHelper::<($($types,)*)>::FIELD_DESCRIPTORS
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
        }

        unsafe impl<$($types,)*> AllocSoaTrusted for ($($types,)*) {}

        unsafe impl<'data, $($types,)*> SoaContext<'data, ($($types,)*)> for ()
        where
            $($types: 'data,)*
        {
            type Refs<'a> = ($(&'data $types,)*);

            #[inline]
            fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
                from
            }

            #[inline]
            unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
                let refs = unsafe { ($(&*ptrs.$indices,)*) };
                refs
            }

            #[inline]
            fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
                let ptrs = ($(ptr::from_ref(refs.$indices),)*);
                ptrs
            }

            type RefsMut<'a> = ($(&'data mut $types,)*);

            #[inline]
            fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
                from
            }

            #[inline]
            unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
                let refs = unsafe { ($(&mut *ptrs.$indices,)*) };
                refs
            }

            #[inline]
            fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
                let ptrs = ($(ptr::from_mut(refs.$indices),)*);
                ptrs
            }

            #[inline]
            fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
                let refs = ($(&*refs.$indices,)*);
                refs
            }

            type Slices<'a> = ($(&'data [$types],)*);

            #[inline]
            fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
                from
            }

            #[inline]
            unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
                let data = RawSoaContext::<($($types,)*)>::slice_ptrs_as_ptrs(self, slices);
                let len = RawSoaContext::<($($types,)*)>::slice_ptrs_len(self, &slices);
                let slices = unsafe { ($(slice::from_raw_parts(data.$indices, len),)*) };
                slices
            }

            #[inline]
            fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
                let slices = ($(ptr::from_ref(slices.$indices),)*);
                slices
            }

            #[inline]
            fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            type SlicesMut<'a> = ($(&'data mut [$types],)*);

            #[inline]
            fn upcast_mut_slices<'short, 'long: 'short>(
                from: Self::SlicesMut<'long>,
            ) -> Self::SlicesMut<'short> {
                from
            }

            #[inline]
            unsafe fn mut_slice_ptrs_to_mut_slices<'a>(
                &'a self,
                slices: Self::SliceMutPtrs<'a>,
            ) -> Self::SlicesMut<'a> {
                let data = RawSoaContext::<($($types,)*)>::mut_slice_ptrs_as_ptrs(self, slices);
                let len = RawSoaContext::<($($types,)*)>::mut_slice_ptrs_len(self, &slices);
                let slices = unsafe { ($(slice::from_raw_parts_mut(data.$indices, len),)*) };
                slices
            }

            #[inline]
            fn mut_slices_as_mut_slice_ptrs<'a>(
                &'a self,
                slices: Self::SlicesMut<'a>,
            ) -> Self::SliceMutPtrs<'a> {
                let slices = ($(ptr::from_mut(slices.$indices),)*);
                slices
            }

            #[inline]
            fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
                let slices = ($(&*slices.$indices,)*);
                slices
            }
        }
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
