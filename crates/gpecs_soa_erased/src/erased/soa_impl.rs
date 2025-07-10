use alloc::vec::Vec;
use core::{iter, ptr::NonNull};

use crate::{
    erased::{
        ErasedSoa, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs, ErasedSoaNonNullPtrs,
        ErasedSoaPtrs, ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSliceMutPtrs, ErasedSoaSlicePtrs,
        ErasedSoaSlices, ErasedSoaSlicesMut, assert::assert_descriptors, soa_slice_from_raw_parts,
        soa_slice_from_raw_parts_mut,
    },
    soa::traits::{FieldDescriptor, Soa},
};

unsafe impl Soa for ErasedSoa {
    type Context = ErasedSoaContext;
    type Fields = ErasedSoaFields;

    type FieldDescriptors<'context> = &'context [FieldDescriptor];

    #[inline]
    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_> {
        context.field_descriptors()
    }

    type Ptrs<'context> = ErasedSoaPtrs<'context>;
    type MutPtrs<'context> = ErasedSoaMutPtrs<'context>;

    #[inline]
    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs<'_> {
        let descriptors = context.field_descriptors();
        ErasedSoaMutPtrs::dangling(descriptors)
    }

    #[inline]
    unsafe fn ptrs_from_buffer<'context>(
        context: &'context Self::Context,
        buffer: *mut u8,
        capacity: usize,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, 0) }
    }

    #[inline]
    fn ptrs_cast_const<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_add<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        offset: usize,
    ) -> Self::Ptrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_add_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        offset: usize,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());
        assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());
        assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, a.field_descriptors());
        assert_descriptors(descriptors, b.field_descriptors());

        let count = descriptors
            .iter()
            .map(|desc| desc.layout().size())
            .max()
            .unwrap_or(0);
        let mut temp = iter::repeat_n(0, count).collect::<Vec<_>>();
        unsafe { a.swap(b, &mut temp) }
    }

    #[inline]
    unsafe fn ptrs_copy(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, src.field_descriptors());
        assert_descriptors(descriptors, dst.field_descriptors());

        let count = descriptors
            .iter()
            .map(|desc| desc.layout().size() * len)
            .max()
            .unwrap_or(0);
        let mut temp = iter::repeat_n(0, count).collect::<Vec<_>>();
        unsafe { dst.copy_from(src, len, &mut temp) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, src.field_descriptors());
        assert_descriptors(descriptors, dst.field_descriptors());

        let count = descriptors
            .iter()
            .map(|desc| desc.layout().size() * len)
            .max()
            .unwrap_or(0);
        let mut temp = iter::repeat_n(0, count).collect::<Vec<_>>();
        unsafe { dst.copy_from_rev(src, len, &mut temp) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, src.field_descriptors());
        assert_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from_nonoverlapping(src, len) }
    }

    #[inline]
    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs<'_>) -> Self {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, src.field_descriptors());

        let fields = descriptors
            .iter()
            .zip(src)
            .map(|(desc, src)| (*desc, unsafe { src.deref().into_buffer() }));
        unsafe { Self::new_unchecked(fields) }
    }

    #[inline]
    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self) {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, dst.field_descriptors());
        assert_descriptors(descriptors, value.field_descriptors());

        dst.into_iter()
            .zip(value.as_refs())
            .for_each(|(dst, src)| unsafe { dst.copy_from_nonoverlapping(src.as_field_ptr(), 1) })
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(_: &Self::Context, _: Self::MutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }

    type NonNullPtrs<'context> = ErasedSoaNonNullPtrs<'context>;

    #[inline]
    unsafe fn ptrs_to_nonnull<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::NonNullPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        unsafe { ErasedSoaNonNullPtrs::new(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    fn nonnull_to_ptrs<'context>(
        context: &'context Self::Context,
        ptrs: Self::NonNullPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = buffer.as_ptr();
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset) }
    }

    type Refs<'context, 'a>
        = ErasedSoaRefs<'context, 'a>
    where
        Self: 'a;

    type RefsMut<'context, 'a>
        = ErasedSoaRefsMut<'context, 'a>
    where
        Self: 'a;

    #[inline]
    unsafe fn ptrs_to_refs<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::Refs<'context, 'a> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.deref() }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::RefsMut<'context, 'a> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.deref_mut() }
    }

    #[inline]
    fn refs_as_ptrs<'context>(
        context: &'context Self::Context,
        refs: Self::Refs<'context, '_>,
    ) -> Self::Ptrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, refs.field_descriptors());

        refs.as_ptr()
    }

    #[inline]
    fn refs_mut_as_ptrs<'context>(
        context: &'context Self::Context,
        mut refs: Self::RefsMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, refs.field_descriptors());

        refs.as_mut_ptr()
    }

    #[inline]
    fn refs_mut_as_refs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, refs.field_descriptors());

        let (descriptors, buffer, capacity, offset) = refs.into_parts();
        let buffer = buffer.cast_const();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    type SlicePtrs<'context> = ErasedSoaSlicePtrs<'context>;
    type SliceMutPtrs<'context> = ErasedSoaSliceMutPtrs<'context>;

    #[inline]
    fn slices_from_raw_parts<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        len: usize,
    ) -> Self::SlicePtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        soa_slice_from_raw_parts(ptrs, len)
    }

    #[inline]
    fn slices_from_raw_parts_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        len: usize,
    ) -> Self::SliceMutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, ptrs.field_descriptors());

        soa_slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn slice_ptrs_cast_const<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicePtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::SliceMutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.cast_mut()
    }

    #[inline]
    fn slice_ptrs_len(context: &Self::Context, slices: &Self::SlicePtrs<'_>) -> usize {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slice_mut_ptrs_len(context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.as_ptrs()
    }

    #[inline]
    fn slice_mut_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        mut slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.as_mut_ptrs()
    }

    type Slices<'context, 'a>
        = ErasedSoaSlices<'context, 'a>
    where
        Self: 'a;

    type SlicesMut<'context, 'a>
        = ErasedSoaSlicesMut<'context, 'a>
    where
        Self: 'a;

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Slices<'context, 'a> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        unsafe { slices.deref() }
    }

    #[inline]
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicesMut<'context, 'a> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        unsafe { slices.deref_mut() }
    }

    #[inline]
    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_, '_>) -> usize {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slices_mut_len(context: &Self::Context, slices: &Self::SlicesMut<'_, '_>) -> usize {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slices_as_slice_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::SlicePtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.as_ptrs()
    }

    #[inline]
    fn slices_mut_as_slice_ptrs<'context>(
        context: &'context Self::Context,
        mut slices: Self::SlicesMut<'context, '_>,
    ) -> Self::SliceMutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.as_mut_ptrs()
    }

    fn slices_mut_as_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        let (descriptors, buffer, capacity, range) = slices.into_parts();
        let buffer = buffer.cast_const();
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, buffer, capacity, range) }
    }

    #[inline]
    fn slices_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::Ptrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.as_ptrs().as_ptrs()
    }

    #[inline]
    fn slices_mut_as_ptrs<'context>(
        context: &'context Self::Context,
        mut slices: Self::SlicesMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        assert_descriptors(descriptors, slices.field_descriptors());

        slices.as_mut_ptrs().as_mut_ptrs()
    }

    #[inline]
    unsafe fn slices_drop_in_place(_: &Self::Context, _: Self::SliceMutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }
}
