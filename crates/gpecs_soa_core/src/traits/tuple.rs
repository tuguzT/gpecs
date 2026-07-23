use core::{
    ptr::{self, NonNull},
    slice,
};

use crate::traits::{
    CloneToUninitSoaContext, RawSoa, RawSoaContext, ReadSoaContext, SoaContext, WriteSoaContext,
};

macro_rules! tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
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
            unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, count: usize) -> Self::Ptrs<'a> {
                let ptrs = unsafe { ($(ptrs.$indices.add(count),)*) };
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
                count: usize,
            ) -> Self::MutPtrs<'a> {
                let ptrs = unsafe { ($(ptrs.$indices.add(count),)*) };
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
            unsafe fn ptrs_swap_nonoverlapping(
                &self,
                x: Self::MutPtrs<'_>,
                y: Self::MutPtrs<'_>,
                count: usize,
            ) {
                unsafe { $(ptr::swap_nonoverlapping(x.$indices, y.$indices, count);)* }
            }

            #[inline]
            unsafe fn ptrs_copy_nonoverlapping(
                &self,
                src: Self::Ptrs<'_>,
                dst: Self::MutPtrs<'_>,
                count: usize,
            ) {
                // because source and destination are non-overlapping, we can copy them in any order
                unsafe { $(ptr::copy_nonoverlapping(src.$indices, dst.$indices, count);)* }
            }

            #[inline]
            unsafe fn ptrs_drop_in_place(&self, to_drop: Self::MutPtrs<'_>) {
                unsafe { $(ptr::drop_in_place(to_drop.$indices);)* }
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
                data: Self::Ptrs<'a>,
                len: usize,
            ) -> Self::SlicePtrs<'a> {
                let slices = ($(ptr::slice_from_raw_parts(data.$indices, len),)*);
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
                data: Self::MutPtrs<'a>,
                len: usize,
            ) -> Self::SliceMutPtrs<'a> {
                let slices = ($(ptr::slice_from_raw_parts_mut(data.$indices, len),)*);
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
            unsafe fn slices_drop_in_place(&self, slices_to_drop: Self::SliceMutPtrs<'_>) {
                unsafe { $(ptr::drop_in_place(slices_to_drop.$indices);)* }
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
                let src = unsafe { ($(src.$indices.as_ref_unchecked(),)*) };
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
                let refs = unsafe { ($(ptrs.$indices.as_ref_unchecked(),)*) };
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
                let refs = unsafe { ($(ptrs.$indices.as_mut_unchecked(),)*) };
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

tuple_impl!(
    A index 0,
);

tuple_impl!(
    A index 0,
    B index 1,
);

tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
);

tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
);

tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
);

tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
);

tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
);

tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
);

tuple_impl!(
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

tuple_impl!(
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

tuple_impl!(
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

tuple_impl!(
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
