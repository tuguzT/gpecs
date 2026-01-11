use core::{
    alloc::{Layout, LayoutError},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::{
    field::FieldDescriptor,
    ptr::assert_ptr_is_aligned,
    traits::{
        MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaCloneToUninit,
        SoaRead, SoaTrustedFields, SoaWrite,
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

/// Type of SoA [context](RawSoaContext) for [tuples](prim@tuple).
pub struct TupleContext<T>(PhantomData<fn() -> T>);

impl<T> TupleContext<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Debug for TupleContext<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TupleContext").finish_non_exhaustive()
    }
}

impl<T> Default for TupleContext<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for TupleContext<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TupleContext<T> {}

impl<T> PartialEq for TupleContext<T> {
    fn eq(&self, other: &Self) -> bool {
        let Self(this) = self;
        let Self(other) = other;
        this == other
    }
}

impl<T> Eq for TupleContext<T> {}

impl<T> PartialOrd for TupleContext<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for TupleContext<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self(this) = self;
        let Self(other) = other;
        this.cmp(other)
    }
}

impl<T> Hash for TupleContext<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self(this) = self;
        this.hash(state);
    }
}

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
        impl<$($types,)*> TupleContext<($($types,)*)> {
            #[doc(hidden)]
            pub const PERMUTATION: [usize; count_idents!($($types,)*)] = {
                let layouts = [$(Layout::new::<$types>(),)*];
                layout_permutation(layouts)
            };
            #[doc(hidden)]
            pub const FIELD_DESCRIPTORS: [FieldDescriptor; count_idents!($($types,)*)] = {
                let permutation = Self::PERMUTATION;
                let descriptors = [$(FieldDescriptor::of::<$types>(),)*];
                [$(descriptors[permutation[$indices]],)*]
            };
        }

        unsafe impl<$($types,)*> super::RawSoaContext for TupleContext<($($types,)*)> {
            type FieldDescriptors<'a> = [FieldDescriptor; count_idents!($($types,)*)];

            #[inline]
            fn upcast_field_descriptors<'short, 'long: 'short>(
                from: Self::FieldDescriptors<'long>,
            ) -> Self::FieldDescriptors<'short> {
                from
            }

            #[inline]
            fn field_descriptors(&self) -> Self::FieldDescriptors<'_> {
                Self::FIELD_DESCRIPTORS
            }

            #[inline]
            fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
                let permutation = Self::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let regions = [$(Layout::array::<$types>(capacity)?,)*];
                $((layout, _) = layout.extend(regions[permutation[$indices]])?;)*

                Ok(layout)
            }

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
            unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
                let permutation = Self::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let mut offsets = [0; count_idents!($($types,)*)];

                let regions = unsafe { [$(Layout::array::<$types>(capacity).unwrap_unchecked(),)*] };
                $((layout, offsets[permutation[$indices]]) = unsafe { layout.extend(regions[permutation[$indices]]).unwrap_unchecked() };)*
                let _ = layout;

                let ptrs = unsafe { ($(buffer.add(offsets[$indices]).cast(),)*) };
                $(assert_ptr_is_aligned(ptrs.$indices);)*
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
            unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
                let permutation = Self::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let mut offsets = [0; count_idents!($($types,)*)];

                let regions = unsafe { [$(Layout::array::<$types>(capacity).unwrap_unchecked(),)*] };
                $((layout, offsets[permutation[$indices]]) = unsafe { layout.extend(regions[permutation[$indices]]).unwrap_unchecked() };)*
                let _ = layout;

                let ptrs = unsafe { ($(buffer.add(offsets[$indices]).cast(),)*) };
                $(assert_ptr_is_aligned(ptrs.$indices);)*
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
                let permutation = Self::PERMUTATION;

                let closures = ($(|| unsafe { ptr::swap(a.$indices, b.$indices) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
                let permutation = Self::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
                let permutation = Self::PERMUTATION;

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
            type Context = TupleContext<($($types,)*)>;
            type Fields = ($($types,)*);
        }

        unsafe impl<'a, $($types,)*> Soa<'a> for ($($types,)*)
        where
            $($types: 'a,)*
        {
            type Refs<'ctx> = ($(&'a $types,)*);

            #[inline]
            fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
                from
            }

            type RefsMut<'ctx> = ($(&'a mut $types,)*);

            #[inline]
            fn upcast_refs_mut<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
                from
            }

            #[inline]
            unsafe fn ptrs_to_refs<'ctx>(
                _context: &'ctx Self::Context,
                ptrs: Ptrs<'ctx, Self>,
            ) -> Self::Refs<'ctx> {
                let refs = unsafe { ($(&*ptrs.$indices,)*) };
                refs
            }

            #[inline]
            unsafe fn ptrs_to_refs_mut<'ctx>(
                _context: &'ctx Self::Context,
                ptrs: MutPtrs<'ctx, Self>,
            ) -> Self::RefsMut<'ctx> {
                let refs = unsafe { ($(&mut *ptrs.$indices,)*) };
                refs
            }

            #[inline]
            fn refs_as_ptrs<'ctx>(
                _context: &'ctx Self::Context,
                refs: Self::Refs<'ctx>,
            ) -> Ptrs<'ctx, Self> {
                let ptrs = ($(ptr::from_ref(refs.$indices),)*);
                ptrs
            }

            #[inline]
            fn refs_mut_as_ptrs<'ctx>(
                _context: &'ctx Self::Context,
                refs: Self::RefsMut<'ctx>,
            ) -> MutPtrs<'ctx, Self> {
                let ptrs = ($(ptr::from_mut(refs.$indices),)*);
                ptrs
            }

            #[inline]
            fn refs_mut_as_refs<'ctx>(
                _context: &'ctx Self::Context,
                refs: Self::RefsMut<'ctx>,
            ) -> Self::Refs<'ctx> {
                let refs = ($(&*refs.$indices,)*);
                refs
            }

            #[inline]
            fn value_as_refs(_context: &'a Self::Context, value: &'a Self) -> Self::Refs<'a> {
                let refs = ($(&value.$indices,)*);
                refs
            }

            #[inline]
            fn mut_value_as_refs(_context: &'a Self::Context, value: &'a mut Self) -> Self::RefsMut<'a> {
                let refs = ($(&mut value.$indices,)*);
                refs
            }

            type Slices<'ctx> = ($(&'a [$types],)*);

            #[inline]
            fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
                from
            }

            type SlicesMut<'ctx> = ($(&'a mut [$types],)*);

            #[inline]
            fn upcast_mut_slices<'short, 'long: 'short>(from: Self::SlicesMut<'long>) -> Self::SlicesMut<'short> {
                from
            }

            #[inline]
            unsafe fn slice_ptrs_to_slices<'ctx>(
                context: &'ctx Self::Context,
                slices: SlicePtrs<'ctx, Self>,
            ) -> Self::Slices<'ctx> {
                let data = context.slice_ptrs_as_ptrs(slices);
                let len = context.slice_ptrs_len(&slices);
                let slices = unsafe { ($(slice::from_raw_parts(data.$indices, len),)*) };
                slices
            }

            #[inline]
            unsafe fn mut_slice_ptrs_to_mut_slices<'ctx>(
                context: &'ctx Self::Context,
                slices: SliceMutPtrs<'ctx, Self>,
            ) -> Self::SlicesMut<'ctx> {
                let data = context.mut_slice_ptrs_as_ptrs(slices);
                let len = context.mut_slice_ptrs_len(&slices);
                let slices = unsafe { ($(slice::from_raw_parts_mut(data.$indices, len),)*) };
                slices
            }

            #[inline]
            fn slices_len(_context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn mut_slices_len(_context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slices_as_slice_ptrs<'ctx>(
                _context: &'ctx Self::Context,
                slices: Self::Slices<'ctx>,
            ) -> SlicePtrs<'ctx, Self> {
                let slices = ($(ptr::from_ref(slices.$indices),)*);
                slices
            }

            #[inline]
            fn mut_slices_as_slice_ptrs<'ctx>(
                _context: &'ctx Self::Context,
                slices: Self::SlicesMut<'ctx>,
            ) -> SliceMutPtrs<'ctx, Self> {
                let slices = ($(ptr::from_mut(slices.$indices),)*);
                slices
            }

            #[inline]
            fn mut_slices_as_slices<'ctx>(
                _context: &'ctx Self::Context,
                slices: Self::SlicesMut<'ctx>,
            ) -> Self::Slices<'ctx> {
                let slices = ($(&*slices.$indices,)*);
                slices
            }

            #[inline]
            fn slices_as_ptrs<'ctx>(
                _context: &'ctx Self::Context,
                slices: Self::Slices<'ctx>,
            ) -> Ptrs<'ctx, Self> {
                let slices = ($(slices.$indices.as_ptr(),)*);
                slices
            }

            #[inline]
            fn mut_slices_as_ptrs<'ctx>(
                _context: &'ctx Self::Context,
                slices: Self::SlicesMut<'ctx>,
            ) -> MutPtrs<'ctx, Self> {
                let slices = ($(slices.$indices.as_mut_ptr(),)*);
                slices
            }
        }

        unsafe impl<$($types,)*> SoaRead for ($($types,)*) {
            #[inline]
            unsafe fn read(_context: &Self::Context, ptrs: Ptrs<'_, Self>) -> Self {
                unsafe { ($(ptr::read(ptrs.$indices),)*) }
            }
        }

        unsafe impl<$($types,)*> SoaWrite for ($($types,)*) {
            #[inline]
            unsafe fn write(_context: &Self::Context, dst: MutPtrs<'_ , Self>, value: Self) {
                unsafe { $(ptr::write(dst.$indices, value.$indices);)* }
            }
        }

        unsafe impl<$($types,)*> SoaCloneToUninit for ($($types,)*)
        where
            $($types: Clone,)*
        {
            #[inline]
            unsafe fn clone_to_uninit(
                _context: &Self::Context,
                src: Ptrs<'_, Self>,
                dst: MutPtrs<'_, Self>,
            ) {
                let src = unsafe { ($(&*src.$indices,)*) };
                unsafe { $(ptr::write(dst.$indices, src.$indices.clone());)* }
            }
        }

        unsafe impl<$($types,)*> SoaTrustedFields for ($($types,)*) {}
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
