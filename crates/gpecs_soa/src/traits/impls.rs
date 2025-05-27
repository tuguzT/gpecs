use core::{
    alloc::{Layout, LayoutError},
    any::type_name,
    array,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};
#[cfg(feature = "alloc")]
use core_alloc::vec::Vec;

#[cfg(feature = "alloc")]
use super::SoaVecs;
use super::{DefaultContext, FieldDescriptor, Soa, SoaToOwned, SoaTrustedFields};

#[inline]
#[track_caller]
pub fn collect_array<T, const N: usize>(iter: impl IntoIterator<Item = T>) -> [T; N] {
    #[cold]
    #[inline(never)]
    #[track_caller]
    fn collect_fail(actual_len: usize, required_len: usize) -> ! {
        panic!("iterator should have {required_len} items, but got {actual_len}")
    }

    let mut iter = iter.into_iter();
    let array = array::from_fn(|index| {
        let Some(offset) = iter.next() else {
            collect_fail(index, N);
        };
        offset
    });
    match iter.count() {
        0 => array,
        len => collect_fail(len + N, N),
    }
}

#[inline]
#[track_caller]
pub fn debug_assert_ptr_is_aligned<T>(ptr: *const T) {
    debug_assert!(
        ptr.is_aligned(),
        "pointer {:p} of {} is not aligned to {} [its current align offset (in bytes) is {}]",
        ptr,
        type_name::<T>(),
        align_of::<T>(),
        ptr.cast::<u8>().align_offset(align_of::<T>()),
    )
}

unsafe impl Soa for () {
    type Context = DefaultContext;
    type Fields = Self;
    type FieldDescriptors<'context> = [FieldDescriptor; 1];

    #[inline]
    fn field_descriptors(_context: &Self::Context) -> Self::FieldDescriptors<'_> {
        [FieldDescriptor::of::<Self>()]
    }

    #[inline]
    fn buffer_layout(_context: &Self::Context, capacity: usize) -> Result<Layout, LayoutError> {
        Layout::array::<Self>(capacity)
    }

    #[inline]
    fn capacity_from(_context: &Self::Context, _buffer_layout: Layout) -> usize {
        usize::MAX
    }

    type Ptrs<'context> = *const Self;
    type MutPtrs<'context> = *mut Self;

    type ErasedPtrs<'context> = [*const u8; 1];
    type ErasedMutPtrs<'context> = [*mut u8; 1];

    #[inline]
    fn ptrs_erase<'context>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::ErasedPtrs<'context> {
        [ptrs.cast()]
    }

    #[inline]
    fn ptrs_erase_mut<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::ErasedMutPtrs<'context> {
        [ptrs.cast()]
    }

    #[track_caller]
    #[inline]
    fn ptrs_restore(
        _context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs<'_> {
        let ptrs = collect_array::<_, 1>(ptrs);
        ptrs[0].cast()
    }

    #[track_caller]
    #[inline]
    fn ptrs_restore_mut(
        _context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs<'_> {
        let ptrs = collect_array::<_, 1>(ptrs);
        ptrs[0].cast()
    }

    #[inline]
    fn ptrs_dangling(_context: &Self::Context) -> Self::MutPtrs<'_> {
        ptr::dangling_mut()
    }

    #[inline]
    unsafe fn ptrs_from_buffer<'context>(
        _context: &'context Self::Context,
        buffer: *mut u8,
        _capacity: usize,
    ) -> Self::MutPtrs<'context> {
        buffer.cast()
    }

    #[inline]
    fn ptrs_cast_const<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::Ptrs<'context> {
        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'context>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::MutPtrs<'context> {
        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_add<'context>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        offset: usize,
    ) -> Self::Ptrs<'context> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_add_mut<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        offset: usize,
    ) -> Self::MutPtrs<'context> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(
        _context: &Self::Context,
        ptrs: Self::Ptrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        _context: &Self::Context,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_swap(_context: &Self::Context, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        unsafe { ptr::swap(a, b) }
    }

    #[inline]
    unsafe fn ptrs_copy(
        _context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(
        _context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        _context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_read(_context: &Self::Context, ptrs: Self::Ptrs<'_>) -> Self {
        unsafe { ptr::read(ptrs) }
    }

    #[inline]
    unsafe fn ptrs_write(_context: &Self::Context, ptrs: Self::MutPtrs<'_>, value: Self) {
        unsafe { ptr::write(ptrs, value) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(_context: &Self::Context, ptrs: Self::MutPtrs<'_>) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs<'context> = NonNull<Self>;

    #[inline]
    unsafe fn ptrs_to_nonnull<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::NonNullPtrs<'context> {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs<'context>(
        _context: &'context Self::Context,
        ptrs: Self::NonNullPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        ptrs.as_ptr()
    }

    type Refs<'context, 'a>
        = &'a Self
    where
        Self: 'a;

    type RefsMut<'context, 'a>
        = &'a mut Self
    where
        Self: 'a;

    #[inline]
    unsafe fn ptrs_to_refs<'context, 'a>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::Refs<'context, 'a> {
        unsafe { &*ptrs }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::RefsMut<'context, 'a> {
        unsafe { &mut *ptrs }
    }

    #[inline]
    fn refs_as_ptrs<'context>(
        _context: &'context Self::Context,
        refs: Self::Refs<'context, '_>,
    ) -> Self::Ptrs<'context> {
        ptr::from_ref(refs)
    }

    #[inline]
    fn refs_mut_as_ptrs<'context>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        ptr::from_mut(refs)
    }

    #[inline]
    fn refs_mut_as_refs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        &*refs
    }

    type SlicePtrs<'context> = *const [Self];
    type SliceMutPtrs<'context> = *mut [Self];

    #[inline]
    fn slices_from_raw_parts<'context>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        len: usize,
    ) -> Self::SlicePtrs<'context> {
        ptr::slice_from_raw_parts(ptrs, len)
    }

    #[inline]
    fn slices_from_raw_parts_mut<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        len: usize,
    ) -> Self::SliceMutPtrs<'context> {
        ptr::slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn slice_ptrs_cast_const<'context>(
        _context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicePtrs<'context> {
        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'context>(
        _context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::SliceMutPtrs<'context> {
        slices.cast_mut()
    }

    #[inline]
    fn slice_ptrs_len(_context: &Self::Context, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_mut_ptrs_len(_context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        slices.cast() // should be `slices.as_ptr()` but it's unstable
    }

    #[inline]
    fn slice_mut_ptrs_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        slices.cast() // should be `slices.as_mut_ptr()` but it's unstable
    }

    type Slices<'context, 'a>
        = &'a [Self]
    where
        Self: 'a;

    type SlicesMut<'context, 'a>
        = &'a mut [Self]
    where
        Self: 'a;

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Slices<'context, 'a> {
        let data = Self::slice_ptrs_as_ptrs(context, slices);
        let len = Self::slice_ptrs_len(context, &slices);
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicesMut<'context, 'a> {
        let data = Self::slice_mut_ptrs_as_ptrs(context, slices);
        let len = Self::slice_mut_ptrs_len(context, &slices);
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    fn slices_len(_context: &Self::Context, slices: &Self::Slices<'_, '_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slices_mut_len(_context: &Self::Context, slices: &Self::SlicesMut<'_, '_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slices_as_slice_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::SlicePtrs<'context> {
        ptr::from_ref(slices)
    }

    #[inline]
    fn slices_mut_as_slice_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::SliceMutPtrs<'context> {
        ptr::from_mut(slices)
    }

    #[inline]
    fn slices_mut_as_slices<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a> {
        &*slices
    }

    #[inline]
    fn slices_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::Ptrs<'context> {
        slices.as_ptr()
    }

    #[inline]
    fn slices_mut_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        slices.as_mut_ptr()
    }

    #[inline]
    unsafe fn slices_drop_in_place(_context: &Self::Context, slices: Self::SliceMutPtrs<'_>) {
        unsafe { ptr::drop_in_place(slices) }
    }
}

impl<'a> SoaToOwned<'_, 'a> for &'a () {
    type Owned = ();

    #[inline]
    fn to_owned(&self) -> Self::Owned {}

    #[inline]
    fn clone_into(&self, _context: &mut Self::Owned) {}

    #[inline]
    unsafe fn clone_into_ptrs(
        &self,
        _context: &<Self::Owned as Soa>::Context,
        _target: <Self::Owned as Soa>::MutPtrs<'_>,
    ) {
    }

    #[inline]
    fn clone_into_refs<'context>(
        &self,
        _context: &'context <Self::Owned as Soa>::Context,
        _target: <Self::Owned as Soa>::RefsMut<'context, '_>,
    ) {
    }
}

#[cfg(feature = "alloc")]
unsafe impl SoaVecs for () {
    type Vecs = Vec<Self>;

    #[inline]
    fn vecs_with_capacity(_context: &Self::Context, capacity: usize) -> Self::Vecs {
        Vec::with_capacity(capacity)
    }

    #[inline]
    fn vecs_as_ptrs<'context>(
        _context: &'context Self::Context,
        vecs: &Self::Vecs,
    ) -> Self::Ptrs<'context> {
        vecs.as_ptr()
    }

    #[inline]
    fn vecs_as_ptrs_mut<'context>(
        _context: &'context Self::Context,
        vecs: &mut Self::Vecs,
    ) -> Self::MutPtrs<'context> {
        vecs.as_mut_ptr()
    }

    #[inline]
    fn vecs_len(_context: &Self::Context, vecs: &Self::Vecs) -> usize {
        vecs.len()
    }

    #[inline]
    unsafe fn vecs_set_len(_context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        unsafe { vecs.set_len(len) }
    }
}

// https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#enum-counting
#[macro_export]
#[doc(hidden)]
macro_rules! count_idents {
    ($($idents:ident),* $(,)*) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            #[repr(usize)]
            enum Idents { $($idents,)* __CountIdentsLast }

            const COUNT: usize = Idents::__CountIdentsLast as usize;
            COUNT
        }
    };
}

#[doc(hidden)]
pub use count_idents;

#[doc(hidden)]
pub struct SoaTupleImplHelper<T>(PhantomData<T>);

#[inline]
const fn permutation<const N: usize>() -> [usize; N] {
    let mut permutation = [0; N];
    let mut i = 0;
    while i < N {
        permutation[i] = i;
        i += 1;
    }
    permutation
}

#[inline]
const fn layout_permutation<const N: usize>(layouts: [Layout; N]) -> [usize; N] {
    let mut permutation = permutation::<N>();
    let mut i = 1;
    while i < N {
        let mut j = i;
        while j > 0 && layouts[permutation[j - 1]].align() > layouts[permutation[j]].align() {
            let tmp = permutation[j - 1];
            permutation[j - 1] = permutation[j];
            permutation[j] = tmp;
            j -= 1;
        }
        i += 1;
    }
    permutation
}

macro_rules! soa_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        impl<$($types,)*> SoaTupleImplHelper<($($types,)*)> {
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

        unsafe impl<$($types,)*> Soa for ($($types,)*) {
            type Context = DefaultContext;
            type Fields = Self;
            type FieldDescriptors<'context> = [FieldDescriptor; count_idents!($($types,)*)];

            #[inline]
            fn field_descriptors(_context: &Self::Context) -> Self::FieldDescriptors<'_> {
                SoaTupleImplHelper::<($($types,)*)>::FIELD_DESCRIPTORS
            }

            #[inline]
            fn buffer_layout(_context: &Self::Context, capacity: usize) -> Result<Layout, LayoutError> {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let regions = [$(Layout::array::<$types>(capacity)?,)*];
                $((layout, _) = layout.extend(regions[permutation[$indices]])?;)*

                Ok(layout)
            }

            type Ptrs<'context> = ($(*const $types,)*);
            type MutPtrs<'context> = ($(*mut $types,)*);

            type ErasedPtrs<'context> = [*const u8; count_idents!($($types,)*)];
            type ErasedMutPtrs<'context> = [*mut u8; count_idents!($($types,)*)];

            #[inline]
            fn ptrs_erase<'context>(
                _context: &'context Self::Context,
                ptrs: Self::Ptrs<'context>,
            ) -> Self::ErasedPtrs<'context> {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let ptrs = [$(ptrs.$indices.cast(),)*];
                let ptrs = [$(ptrs[permutation[$indices]],)*];
                ptrs
            }

            #[inline]
            fn ptrs_erase_mut<'context>(
                _context: &'context Self::Context,
                ptrs: Self::MutPtrs<'context>,
            ) -> Self::ErasedMutPtrs<'context> {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let ptrs = [$(ptrs.$indices.cast(),)*];
                let ptrs = [$(ptrs[permutation[$indices]],)*];
                ptrs
            }

            #[inline]
            fn ptrs_restore(_context: &Self::Context, ptrs: impl IntoIterator<Item = *const u8>) -> Self::Ptrs<'_> {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let ptrs = collect_array::<_, { count_idents!($($types,)*) }>(ptrs);
                let ptrs = {
                    let mut result = [ptr::null(); count_idents!($($types,)*)];
                    $(result[permutation[$indices]] = ptrs[$indices];)*
                    result
                };

                let ptrs: Self::Ptrs<'_> = ($(ptrs[$indices].cast(),)*);
                $(debug_assert_ptr_is_aligned(ptrs.$indices);)*
                ptrs
            }

            #[inline]
            fn ptrs_restore_mut(_context: &Self::Context, ptrs: impl IntoIterator<Item = *mut u8>) -> Self::MutPtrs<'_> {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let ptrs = collect_array::<_, { count_idents!($($types,)*) }>(ptrs);
                let ptrs = {
                    let mut result = [ptr::null_mut(); count_idents!($($types,)*)];
                    $(result[permutation[$indices]] = ptrs[$indices];)*
                    result
                };

                let ptrs: Self::MutPtrs<'_> = ($(ptrs[$indices].cast(),)*);
                $(debug_assert_ptr_is_aligned(ptrs.$indices);)*
                ptrs
            }

            #[inline]
            fn ptrs_dangling(_context: &Self::Context) -> Self::MutPtrs<'_> {
                let ptrs = ($(ptr::dangling_mut::<$types>(),)*);
                ptrs
            }

            #[inline]
            unsafe fn ptrs_from_buffer<'context>(
                _context: &'context Self::Context,
                buffer: *mut u8,
                capacity: usize,
            ) -> Self::MutPtrs<'context> {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let mut offsets = [0; count_idents!($($types,)*)];

                let regions = unsafe { [$(Layout::array::<$types>(capacity).unwrap_unchecked(),)*] };
                $((layout, offsets[permutation[$indices]]) = unsafe { layout.extend(regions[permutation[$indices]]).unwrap_unchecked() };)*
                let _ = layout;

                let ptrs = unsafe { ($(buffer.add(offsets[$indices]).cast(),)*) };
                $(debug_assert_ptr_is_aligned(ptrs.$indices);)*
                ptrs
            }

            #[inline]
            fn ptrs_cast_const<'context>(
                _context: &'context Self::Context,
                ptrs: Self::MutPtrs<'context>,
            ) -> Self::Ptrs<'context> {
                let ptrs = ($(ptrs.$indices.cast_const(),)*);
                ptrs
            }

            #[inline]
            fn ptrs_cast_mut<'context>(
                _context: &'context Self::Context,
                ptrs: Self::Ptrs<'context>,
            ) -> Self::MutPtrs<'context> {
                let ptrs = ($(ptrs.$indices.cast_mut(),)*);
                ptrs
            }

            #[inline]
            unsafe fn ptrs_add<'context>(
                _context: &'context Self::Context,
                ptrs: Self::Ptrs<'context>,
                offset: usize,
            ) -> Self::Ptrs<'context> {
                let ptrs = unsafe { ($(ptrs.$indices.add(offset),)*) };
                ptrs
            }

            #[inline]
            unsafe fn ptrs_add_mut<'context>(
                _context: &'context Self::Context,
                ptrs: Self::MutPtrs<'context>,
                offset: usize,
            ) -> Self::MutPtrs<'context> {
                let ptrs = unsafe { ($(ptrs.$indices.add(offset),)*) };
                ptrs
            }

            #[inline]
            unsafe fn ptrs_offset_from(
                _context: &Self::Context,
                ptrs: Self::Ptrs<'_>,
                origin: Self::Ptrs<'_>,
            ) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline]
            unsafe fn ptrs_offset_from_mut(
                _context: &Self::Context,
                ptrs: Self::MutPtrs<'_>,
                origin: Self::Ptrs<'_>,
            ) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline]
            unsafe fn ptrs_swap(
                _context: &Self::Context,
                a: Self::MutPtrs<'_>,
                b: Self::MutPtrs<'_>,
            ) {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::swap(a.$indices, b.$indices) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy(
                _context: &Self::Context,
                src: Self::Ptrs<'_>,
                dst: Self::MutPtrs<'_>,
                len: usize,
            ) {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_rev(
                _context: &Self::Context,
                src: Self::Ptrs<'_>,
                dst: Self::MutPtrs<'_>,
                len: usize,
            ) {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in (0..count_idents!($($types,)*)).rev() {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_nonoverlapping(
                _context: &Self::Context,
                src: Self::Ptrs<'_>,
                dst: Self::MutPtrs<'_>,
                len: usize,
            ) {
                // because source and destination are non-overlapping, we can copy them in any order
                unsafe { $(ptr::copy_nonoverlapping(src.$indices, dst.$indices, len);)* }
            }

            #[inline]
            unsafe fn ptrs_read(_context: &Self::Context, ptrs: Self::Ptrs<'_>) -> Self {
                unsafe { ($(ptr::read(ptrs.$indices),)*) }
            }

            #[inline]
            unsafe fn ptrs_write(_context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self) {
                unsafe { $(ptr::write(dst.$indices, value.$indices);)* }
            }

            #[inline]
            unsafe fn ptrs_drop_in_place(_context: &Self::Context, ptrs: Self::MutPtrs<'_>) {
                unsafe { $(ptr::drop_in_place(ptrs.$indices);)* }
            }

            type NonNullPtrs<'context> = ($(NonNull<$types>,)*);

            #[inline]
            unsafe fn ptrs_to_nonnull<'context>(
                _context: &'context Self::Context,
                ptrs: Self::MutPtrs<'context>,
            ) -> Self::NonNullPtrs<'context> {
                let ptrs = unsafe { ($(NonNull::new_unchecked(ptrs.$indices),)*) };
                ptrs
            }

            #[inline]
            fn nonnull_to_ptrs<'context>(
                _context: &'context Self::Context,
                ptrs: Self::NonNullPtrs<'context>,
            ) -> Self::MutPtrs<'context> {
                let ptrs = ($(ptrs.$indices.as_ptr(),)*);
                ptrs
            }

            type Refs<'context, 'a>
                = ($(&'a $types,)*)
            where
                Self: 'a;

            type RefsMut<'context, 'a>
                = ($(&'a mut $types,)*)
            where
                Self: 'a;

            #[inline]
            unsafe fn ptrs_to_refs<'context, 'a>(
                _context: &'context Self::Context,
                ptrs: Self::Ptrs<'context>,
            ) -> Self::Refs<'context, 'a> {
                let refs = unsafe { ($(&*ptrs.$indices,)*) };
                refs
            }

            #[inline]
            unsafe fn ptrs_to_refs_mut<'context, 'a>(
                _context: &'context Self::Context,
                ptrs: Self::MutPtrs<'context>,
            ) -> Self::RefsMut<'context, 'a> {
                let refs = unsafe { ($(&mut *ptrs.$indices,)*) };
                refs
            }

            #[inline]
            fn refs_as_ptrs<'context>(
                _context: &'context Self::Context,
                refs: Self::Refs<'context, '_>,
            ) -> Self::Ptrs<'context> {
                let ptrs = ($(ptr::from_ref(refs.$indices),)*);
                ptrs
            }

            #[inline]
            fn refs_mut_as_ptrs<'context>(
                _context: &'context Self::Context,
                refs: Self::RefsMut<'context, '_>,
            ) -> Self::MutPtrs<'context> {
                let ptrs = ($(ptr::from_mut(refs.$indices),)*);
                ptrs
            }

            #[inline]
            fn refs_mut_as_refs<'context, 'a>(
                _context: &'context Self::Context,
                refs: Self::RefsMut<'context, 'a>,
            ) -> Self::Refs<'context, 'a> {
                let refs = ($(&*refs.$indices,)*);
                refs
            }

            type SlicePtrs<'context> = ($(*const [$types],)*);
            type SliceMutPtrs<'context> = ($(*mut [$types],)*);

            #[inline]
            fn slices_from_raw_parts<'context>(
                _context: &'context Self::Context,
                ptrs: Self::Ptrs<'context>,
                len: usize,
            ) -> Self::SlicePtrs<'context> {
                let slices = ($(ptr::slice_from_raw_parts(ptrs.$indices, len),)*);
                slices
            }

            #[inline]
            fn slices_from_raw_parts_mut<'context>(
                _context: &'context Self::Context,
                ptrs: Self::MutPtrs<'context>,
                len: usize,
            ) -> Self::SliceMutPtrs<'context> {
                let slices = ($(ptr::slice_from_raw_parts_mut(ptrs.$indices, len),)*);
                slices
            }

            #[inline]
            fn slice_ptrs_cast_const<'context>(
                _context: &'context Self::Context,
                slices: Self::SliceMutPtrs<'context>,
            ) -> Self::SlicePtrs<'context> {
                let slices = ($(slices.$indices.cast_const(),)*);
                slices
            }

            #[inline]
            fn slice_ptrs_cast_mut<'context>(
                _context: &'context Self::Context,
                slices: Self::SlicePtrs<'context>,
            ) -> Self::SliceMutPtrs<'context> {
                let slices = ($(slices.$indices.cast_mut(),)*);
                slices
            }

            #[inline]
            fn slice_ptrs_len(_context: &Self::Context, slices: &Self::SlicePtrs<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slice_mut_ptrs_len(_context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slice_ptrs_as_ptrs<'context>(
                _context: &'context Self::Context,
                slices: Self::SlicePtrs<'context>,
            ) -> Self::Ptrs<'context> {
                let slices = ($(slices.$indices.cast(),)*); // should be `slices.$indices.as_ptr()` but it's unstable
                slices
            }

            #[inline]
            fn slice_mut_ptrs_as_ptrs<'context>(
                _context: &'context Self::Context,
                slices: Self::SliceMutPtrs<'context>,
            ) -> Self::MutPtrs<'context> {
                let slices = ($(slices.$indices.cast(),)*); // should be `slices.$indices.as_mut_ptr()` but it's unstable
                slices
            }

            type Slices<'context, 'a>
                = ($(&'a [$types],)*)
            where
                Self: 'a;

            type SlicesMut<'context, 'a>
                = ($(&'a mut [$types],)*)
            where
                Self: 'a;

            #[inline]
            unsafe fn slice_ptrs_to_slices<'context, 'a>(
                context: &'context Self::Context,
                slices: Self::SlicePtrs<'context>,
            ) -> Self::Slices<'context, 'a> {
                let data = Self::slice_ptrs_as_ptrs(context, slices);
                let len = Self::slice_ptrs_len(context, &slices);
                let slices = unsafe { ($(slice::from_raw_parts(data.$indices, len),)*) };
                slices
            }

            #[inline]
            unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
                context: &'context Self::Context,
                slices: Self::SliceMutPtrs<'context>,
            ) -> Self::SlicesMut<'context, 'a> {
                let data = Self::slice_mut_ptrs_as_ptrs(context, slices);
                let len = Self::slice_mut_ptrs_len(context, &slices);
                let slices = unsafe { ($(slice::from_raw_parts_mut(data.$indices, len),)*) };
                slices
            }

            #[inline]
            fn slices_len(_context: &Self::Context, slices: &Self::Slices<'_, '_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slices_mut_len(_context: &Self::Context, slices: &Self::SlicesMut<'_, '_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slices_as_slice_ptrs<'context>(
                _context: &'context Self::Context,
                slices: Self::Slices<'context, '_>,
            ) -> Self::SlicePtrs<'context> {
                let slices = ($(ptr::from_ref(slices.$indices),)*);
                slices
            }

            #[inline]
            fn slices_mut_as_slice_ptrs<'context>(
                _context: &'context Self::Context,
                slices: Self::SlicesMut<'context, '_>,
            ) -> Self::SliceMutPtrs<'context> {
                let slices = ($(ptr::from_mut(slices.$indices),)*);
                slices
            }

            #[inline]
            fn slices_mut_as_slices<'context, 'a>(
                _context: &'context Self::Context,
                slices: Self::SlicesMut<'context, 'a>,
            ) -> Self::Slices<'context, 'a> {
                let slices = ($(&*slices.$indices,)*);
                slices
            }

            #[inline]
            fn slices_as_ptrs<'context>(
                _context: &'context Self::Context,
                slices: Self::Slices<'context, '_>,
            ) -> Self::Ptrs<'context> {
                let slices = ($(slices.$indices.as_ptr(),)*);
                slices
            }

            #[inline]
            fn slices_mut_as_ptrs<'context>(
                _context: &'context Self::Context,
                slices: Self::SlicesMut<'context, '_>,
            ) -> Self::MutPtrs<'context> {
                let slices = ($(slices.$indices.as_mut_ptr(),)*);
                slices
            }

            #[inline]
            unsafe fn slices_drop_in_place(_context: &Self::Context, slices: Self::SliceMutPtrs<'_>) {
                unsafe { $(ptr::drop_in_place(slices.$indices);)* }
            }
        }

        impl<'a, $($types,)*> SoaToOwned<'_, 'a> for ($(&'a $types,)*)
        where
            $($types: Clone,)*
        {
            type Owned = ($($types,)*);

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                let owned = ($(self.$indices.clone(),)*);
                owned
            }

            #[inline]
            fn clone_into(&self, target: &mut Self::Owned) {
                $(target.$indices.clone_from(self.$indices);)*
            }

            #[inline]
            fn clone_into_refs<'context>(
                &self,
                _context: &'context <Self::Owned as Soa>::Context,
                target: <Self::Owned as Soa>::RefsMut<'context, '_>,
            ) {
                $(target.$indices.clone_from(self.$indices);)*
            }
        }

        #[cfg(feature = "alloc")]
        unsafe impl<$($types,)*> SoaVecs for ($($types,)*) {
            type Vecs = ($(Vec<$types>,)*);

            #[inline]
            fn vecs_with_capacity(_context: &Self::Context, capacity: usize) -> Self::Vecs {
                let vecs = ($(Vec::<$types>::with_capacity(capacity),)*);
                vecs
            }

            #[inline]
            fn vecs_as_ptrs<'context>(
                _context: &'context Self::Context,
                vecs: &Self::Vecs,
            ) -> Self::Ptrs<'context> {
                let ptrs = ($(vecs.$indices.as_ptr(),)*);
                ptrs
            }

            #[inline]
            fn vecs_as_ptrs_mut<'context>(
                _context: &'context Self::Context,
                vecs: &mut Self::Vecs,
            ) -> Self::MutPtrs<'context> {
                let ptrs = ($(vecs.$indices.as_mut_ptr(),)*);
                ptrs
            }

            #[inline]
            fn vecs_len(_context: &Self::Context, vecs: &Self::Vecs) -> usize {
                let lens = [$(vecs.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            unsafe fn vecs_set_len(_context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
                unsafe { $(vecs.$indices.set_len(len);)* }
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
