use core::{fmt::Debug, ptr::NonNull};

use crate::{
    aligned_bytes::{AlignedBytes, AlignedBytesFromLayout},
    soa::{
        field::FieldDescriptor,
        traits::{Soa, SoaRead, SoaWrite},
    },
};

use super::{
    ErasedSoa, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs, ErasedSoaNonNullPtrs,
    ErasedSoaPtrs, ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSliceMutPtrs, ErasedSoaSlicePtrs,
    ErasedSoaSlices, ErasedSoaSlicesMut, assert::debug_assert_descriptors,
    soa_slice_from_raw_parts, soa_slice_from_raw_parts_mut,
};

unsafe impl<B, D> Soa for ErasedSoa<B, D>
where
    B: AlignedBytes + ?Sized,
    D: AsRef<[FieldDescriptor]>,
{
    type Context = ErasedSoaContext<D>;
    type Fields = ErasedSoaFields;

    type FieldDescriptors<'context> = &'context [FieldDescriptor];

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        from
    }

    #[inline]
    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_> {
        context.field_descriptors()
    }

    type Ptrs<'context> = ErasedSoaPtrs<&'context [FieldDescriptor]>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(context: &Self::Context) -> Self::Ptrs<'_> {
        let descriptors = context.field_descriptors();
        ErasedSoaPtrs::dangling(descriptors)
    }

    #[inline]
    unsafe fn ptrs_from_buffer(
        context: &Self::Context,
        buffer: *const u8,
        capacity: usize,
    ) -> Self::Ptrs<'_> {
        let descriptors = context.field_descriptors();
        unsafe { ErasedSoaPtrs::new(descriptors, buffer, capacity, 0) }
    }

    type MutPtrs<'context> = ErasedSoaMutPtrs<&'context [FieldDescriptor]>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(context: &Self::Context) -> Self::MutPtrs<'_> {
        let descriptors = context.field_descriptors();
        ErasedSoaMutPtrs::dangling(descriptors)
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(
        context: &Self::Context,
        buffer: *mut u8,
        capacity: usize,
    ) -> Self::MutPtrs<'_> {
        let descriptors = context.field_descriptors();
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, 0) }
    }

    #[inline]
    fn ptrs_cast_const<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_add<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        offset: usize,
    ) -> Self::Ptrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_add_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        offset: usize,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());
        debug_assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { ptrs.offset_from(&origin) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());
        debug_assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { ptrs.offset_from(&origin) }
    }

    #[inline]
    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, a.field_descriptors());
        debug_assert_descriptors(descriptors, b.field_descriptors());

        unsafe { a.swap(&b) }
    }

    #[inline]
    unsafe fn ptrs_copy(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, src.field_descriptors());
        debug_assert_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, src.field_descriptors());
        debug_assert_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from_rev(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, src.field_descriptors());
        debug_assert_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from_nonoverlapping(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(_: &Self::Context, _: Self::MutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }

    type NonNullPtrs<'context> = ErasedSoaNonNullPtrs<&'context [FieldDescriptor]>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::NonNullPtrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

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
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        let buffer = buffer.as_ptr();
        unsafe { ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset) }
    }

    type Refs<'context, 'a>
        = ErasedSoaRefs<'a, &'context [FieldDescriptor]>
    where
        Self: 'a;

    #[inline]
    fn upcast_refs<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Refs<'long, 'a_long>,
    ) -> Self::Refs<'short, 'a_short>
    where
        Self: 'a_long,
    {
        from
    }

    type RefsMut<'context, 'a>
        = ErasedSoaRefsMut<'a, &'context [FieldDescriptor]>
    where
        Self: 'a;

    #[inline]
    fn upcast_refs_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::RefsMut<'long, 'a_long>,
    ) -> Self::RefsMut<'short, 'a_short>
    where
        Self: 'a_long,
    {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::Refs<'context, 'a>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.deref() }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::RefsMut<'context, 'a>
    where
        Self: 'a,
    {
        let descriptors: &[FieldDescriptor] = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.deref_mut() }
    }

    #[inline]
    fn refs_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::Refs<'context, 'a>,
    ) -> Self::Ptrs<'context>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, refs.field_descriptors());

        refs.into_ptrs()
    }

    #[inline]
    fn refs_mut_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::MutPtrs<'context>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, refs.field_descriptors());

        refs.into_mut_ptrs()
    }

    #[inline]
    fn refs_mut_as_refs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, refs.field_descriptors());

        let (descriptors, buffer, capacity, offset) = refs.into_parts();
        let buffer = buffer.cast_const();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    fn value_as_refs<'context, 'a>(
        context: &'context Self::Context,
        value: &'a Self,
    ) -> Self::Refs<'context, 'a>
    where
        Self: 'a,
        'a: 'context,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, value.field_descriptors());

        value.as_refs()
    }

    #[inline]
    fn mut_value_as_refs<'context, 'a>(
        context: &'context Self::Context,
        value: &'a mut Self,
    ) -> Self::RefsMut<'context, 'a>
    where
        Self: 'a,
        'a: 'context,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, value.field_descriptors());

        value.as_refs_mut()
    }

    type SlicePtrs<'context> = ErasedSoaSlicePtrs<&'context [FieldDescriptor]>;

    #[inline]
    fn upcast_slice_ptrs<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        from
    }

    type SliceMutPtrs<'context> = ErasedSoaSliceMutPtrs<&'context [FieldDescriptor]>;

    #[inline]
    fn upcast_slice_mut_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        from
    }

    #[inline]
    fn slices_from_raw_parts<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        len: usize,
    ) -> Self::SlicePtrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        soa_slice_from_raw_parts(ptrs, len)
    }

    #[inline]
    fn slices_from_raw_parts_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        len: usize,
    ) -> Self::SliceMutPtrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        soa_slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn slice_ptrs_cast_const<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicePtrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::SliceMutPtrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.cast_mut()
    }

    #[inline]
    fn slice_ptrs_len(context: &Self::Context, slices: &Self::SlicePtrs<'_>) -> usize {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slice_mut_ptrs_len(context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_ptrs()
    }

    #[inline]
    fn slice_mut_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_mut_ptrs()
    }

    type Slices<'context, 'a>
        = ErasedSoaSlices<'a, &'context [FieldDescriptor]>
    where
        Self: 'a;

    #[inline]
    fn upcast_slices<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Slices<'long, 'a_long>,
    ) -> Self::Slices<'short, 'a_short>
    where
        Self: 'a_long,
    {
        from
    }

    type SlicesMut<'context, 'a>
        = ErasedSoaSlicesMut<'a, &'context [FieldDescriptor]>
    where
        Self: 'a;

    #[inline]
    fn upcast_slices_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::SlicesMut<'long, 'a_long>,
    ) -> Self::SlicesMut<'short, 'a_short>
    where
        Self: 'a_long,
    {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Slices<'context, 'a>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        unsafe { slices.deref() }
    }

    #[inline]
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicesMut<'context, 'a>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        unsafe { slices.deref_mut() }
    }

    #[inline]
    fn slices_len<'a>(context: &Self::Context, slices: &Self::Slices<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slices_mut_len<'a>(context: &Self::Context, slices: &Self::SlicesMut<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slices_as_slice_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> Self::SlicePtrs<'context>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_ptrs()
    }

    #[inline]
    fn slices_mut_as_slice_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::SliceMutPtrs<'context>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_mut_ptrs()
    }

    fn slices_mut_as_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        let (descriptors, buffer, capacity, range) = slices.into_parts();
        let buffer = buffer.cast_const();
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, buffer, capacity, range) }
    }

    #[inline]
    fn slices_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> Self::Ptrs<'context>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_ptrs().into_ptrs()
    }

    #[inline]
    fn slices_mut_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::MutPtrs<'context>
    where
        Self: 'a,
    {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_mut_ptrs().into_mut_ptrs()
    }

    #[inline]
    unsafe fn slices_drop_in_place(_: &Self::Context, _: Self::SliceMutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }
}

unsafe impl<B, D> SoaRead for ErasedSoa<B, D>
where
    B: AlignedBytesFromLayout<Error: Debug>,
    D: AsRef<[FieldDescriptor]> + Clone,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Self::Ptrs<'_>) -> Self {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, src.field_descriptors());

        let fields = src
            .into_iter()
            .map(|src| unsafe { src.deref().into_buffer() });
        let descriptors = context.clone().into_field_descriptors();
        Self::from_fields_descriptors(fields, descriptors)
            .expect("length of fields should be equal to the length of descriptors")
    }
}

unsafe impl<B, D> SoaWrite for ErasedSoa<B, D>
where
    B: AlignedBytes,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    unsafe fn write(context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self) {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, dst.field_descriptors());
        debug_assert_descriptors(descriptors, value.field_descriptors());

        dst.into_iter()
            .zip(value.as_refs())
            .for_each(|(dst, src)| unsafe { dst.copy_from_nonoverlapping(src.as_field_ptr(), 1) });
    }
}
