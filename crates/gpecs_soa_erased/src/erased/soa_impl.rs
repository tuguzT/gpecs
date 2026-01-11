use core::{fmt::Debug, ptr::NonNull};

use crate::{
    aligned_bytes::{AlignedBytes, AlignedBytesFromLayout},
    soa::{
        field::FieldDescriptor,
        traits::{
            MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaRead, SoaWrite,
        },
    },
};

use super::{
    ErasedSoa, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs, ErasedSoaNonNullPtrs,
    ErasedSoaPtrs, ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSliceMutPtrs, ErasedSoaSlicePtrs,
    ErasedSoaSlices, ErasedSoaSlicesMut, assert::debug_assert_descriptors, slice_from_raw_parts,
    slice_from_raw_parts_mut,
};

unsafe impl<D> RawSoaContext for ErasedSoaContext<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    type FieldDescriptors<'a> = &'a [FieldDescriptor];

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        from
    }

    #[inline]
    fn field_descriptors(&self) -> Self::FieldDescriptors<'_> {
        Self::field_descriptors(self)
    }

    type Ptrs<'a> = ErasedSoaPtrs<&'a [FieldDescriptor]>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        let descriptors = self.field_descriptors();
        ErasedSoaPtrs::dangling(descriptors)
    }

    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let descriptors = self.field_descriptors();
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, buffer, capacity, 0) }
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());
        debug_assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { ptrs.offset_from(&origin) }
    }

    type MutPtrs<'a> = ErasedSoaMutPtrs<&'a [FieldDescriptor]>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        let descriptors = self.field_descriptors();
        ErasedSoaMutPtrs::dangling(descriptors)
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let descriptors = self.field_descriptors();
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, 0) }
    }

    #[inline]
    unsafe fn ptrs_add_mut<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        offset: usize,
    ) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        &self,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());
        debug_assert_descriptors(descriptors, origin.field_descriptors());

        unsafe { ptrs.offset_from(&origin) }
    }

    #[inline]
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_swap(&self, mut a: Self::MutPtrs<'_>, mut b: Self::MutPtrs<'_>) {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, a.field_descriptors());
        debug_assert_descriptors(descriptors, b.field_descriptors());

        unsafe { a.swap(&mut b) }
    }

    #[inline]
    unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, mut dst: Self::MutPtrs<'_>, len: usize) {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, src.field_descriptors());
        debug_assert_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, mut dst: Self::MutPtrs<'_>, len: usize) {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, src.field_descriptors());
        debug_assert_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from_rev(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        mut dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, src.field_descriptors());
        debug_assert_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from_nonoverlapping(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, _: Self::MutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }

    type NonNullPtrs<'a> = ErasedSoaNonNullPtrs<&'a [FieldDescriptor]>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        unsafe { ErasedSoaNonNullPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        let ptr = ptr.as_ptr();
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    type SlicePtrs<'a> = ErasedSoaSlicePtrs<&'a [FieldDescriptor]>;

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
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { slice_from_raw_parts(ptrs, len) }
    }

    #[inline]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_ptrs()
    }

    type SliceMutPtrs<'a> = ErasedSoaSliceMutPtrs<&'a [FieldDescriptor]>;

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
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { slice_from_raw_parts_mut(ptrs, len) }
    }

    #[inline]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_mut_ptrs()
    }

    #[inline]
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.cast_mut()
    }

    #[inline]
    unsafe fn slices_drop_in_place(&self, _: Self::SliceMutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }
}

unsafe impl<B, D> RawSoa for ErasedSoa<B, D>
where
    B: ?Sized,
    D: AsRef<[FieldDescriptor]>,
{
    type Context = ErasedSoaContext<D>;
    type Fields = ErasedSoaFields;
}

unsafe impl<'a, B, D> Soa<'a> for ErasedSoa<B, D>
where
    B: AlignedBytes + ?Sized + 'a,
    D: AsRef<[FieldDescriptor]>,
{
    type Refs<'ctx> = ErasedSoaRefs<'a, &'ctx [FieldDescriptor]>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    type RefsMut<'ctx> = ErasedSoaRefsMut<'a, &'ctx [FieldDescriptor]>;

    #[inline]
    fn upcast_refs_mut<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'ctx>(
        context: &'ctx Self::Context,
        ptrs: Ptrs<'ctx, Self>,
    ) -> Self::Refs<'ctx> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.deref() }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'ctx>(
        context: &'ctx Self::Context,
        ptrs: MutPtrs<'ctx, Self>,
    ) -> Self::RefsMut<'ctx> {
        let descriptors: &[FieldDescriptor] = context.field_descriptors();
        debug_assert_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.deref_mut() }
    }

    #[inline]
    fn refs_as_ptrs<'ctx>(
        context: &'ctx Self::Context,
        refs: Self::Refs<'ctx>,
    ) -> Ptrs<'ctx, Self> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, refs.field_descriptors());

        refs.into_ptrs()
    }

    #[inline]
    fn refs_mut_as_ptrs<'ctx>(
        context: &'ctx Self::Context,
        refs: Self::RefsMut<'ctx>,
    ) -> MutPtrs<'ctx, Self> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, refs.field_descriptors());

        refs.into_mut_ptrs()
    }

    #[inline]
    fn refs_mut_as_refs<'ctx>(
        context: &'ctx Self::Context,
        refs: Self::RefsMut<'ctx>,
    ) -> Self::Refs<'ctx> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, refs.field_descriptors());

        let (descriptors, ptr, capacity, offset) = refs.into_parts();
        let ptr = ptr.cast_const();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    fn value_as_refs(context: &'a Self::Context, value: &'a Self) -> Self::Refs<'a> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, value.field_descriptors());

        value.as_fields()
    }

    #[inline]
    fn mut_value_as_refs(context: &'a Self::Context, value: &'a mut Self) -> Self::RefsMut<'a> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, value.field_descriptors());

        value.as_mut_fields()
    }

    type Slices<'ctx> = ErasedSoaSlices<'a, &'ctx [FieldDescriptor]>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
    }

    type SlicesMut<'ctx> = ErasedSoaSlicesMut<'a, &'ctx [FieldDescriptor]>;

    #[inline]
    fn upcast_mut_slices<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'ctx>(
        context: &'ctx Self::Context,
        slices: SlicePtrs<'ctx, Self>,
    ) -> Self::Slices<'ctx> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        unsafe { slices.deref() }
    }

    #[inline]
    unsafe fn mut_slice_ptrs_to_mut_slices<'ctx>(
        context: &'ctx Self::Context,
        slices: SliceMutPtrs<'ctx, Self>,
    ) -> Self::SlicesMut<'ctx> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        unsafe { slices.deref_mut() }
    }

    #[inline]
    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn mut_slices_len(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slices_as_slice_ptrs<'ctx>(
        context: &'ctx Self::Context,
        slices: Self::Slices<'ctx>,
    ) -> SlicePtrs<'ctx, Self> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_ptrs()
    }

    #[inline]
    fn mut_slices_as_slice_ptrs<'ctx>(
        context: &'ctx Self::Context,
        slices: Self::SlicesMut<'ctx>,
    ) -> SliceMutPtrs<'ctx, Self> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_mut_ptrs()
    }

    fn mut_slices_as_slices<'ctx>(
        context: &'ctx Self::Context,
        slices: Self::SlicesMut<'ctx>,
    ) -> Self::Slices<'ctx> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        let (descriptors, ptr, capacity, offset, len) = slices.into_parts();
        let ptr = ptr.cast_const();
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, ptr, capacity, offset, len) }
    }

    #[inline]
    fn slices_as_ptrs<'ctx>(
        context: &'ctx Self::Context,
        slices: Self::Slices<'ctx>,
    ) -> Ptrs<'ctx, Self> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_ptrs().into_ptrs()
    }

    #[inline]
    fn mut_slices_as_ptrs<'ctx>(
        context: &'ctx Self::Context,
        slices: Self::SlicesMut<'ctx>,
    ) -> MutPtrs<'ctx, Self> {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, slices.field_descriptors());

        slices.into_mut_ptrs().into_mut_ptrs()
    }
}

unsafe impl<B, D> SoaRead for ErasedSoa<B, D>
where
    B: AlignedBytesFromLayout<Error: Debug>,
    D: AsRef<[FieldDescriptor]> + Clone,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Ptrs<'_, Self>) -> Self {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, src.field_descriptors());

        let fields = src
            .into_iter()
            .map(|src| unsafe { src.deref().into_buffer() });
        let descriptors = context.clone().into_field_descriptors();
        Self::try_from_fields_descriptors(fields, descriptors)
            .expect("length of fields should be equal to the length of descriptors")
    }
}

unsafe impl<B, D> SoaWrite for ErasedSoa<B, D>
where
    B: AlignedBytes,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    unsafe fn write(context: &Self::Context, dst: MutPtrs<'_, Self>, value: Self) {
        let descriptors = context.field_descriptors();
        debug_assert_descriptors(descriptors, dst.field_descriptors());
        debug_assert_descriptors(descriptors, value.field_descriptors());

        dst.into_iter()
            .zip(value.as_fields())
            .for_each(|(dst, src)| unsafe { dst.copy_from_nonoverlapping(src.as_field_ptr(), 1) });
    }
}
