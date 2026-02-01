use core::{
    fmt::Debug,
    ptr::{self, NonNull},
};

use crate::{
    erased::{
        CovariantFieldDescriptors, ErasedSoa, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs,
        ErasedSoaNonNullPtrs, ErasedSoaPtrs, ErasedSoaRefs, ErasedSoaRefsMut,
        ErasedSoaSliceMutPtrs, ErasedSoaSlicePtrs, ErasedSoaSlices, ErasedSoaSlicesMut,
        assert::assert_descriptors, slice_from_raw_parts, slice_from_raw_parts_mut,
    },
    soa::{
        field::{FieldDescriptors, FieldDescriptorsOutput},
        traits::{
            AllocSoaContext, MutPtrs, Ptrs, RawSoa, RawSoaContext, Refs, RefsMut, SoaAsMutRefs,
            SoaAsRefs, SoaContext, SoaRead, SoaWrite,
        },
    },
    storage::{AddressableUnit, AlignedStorage, AlignedStorageFromLayout},
};

unsafe impl<D, A> RawSoaContext for ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
{
    type Ptrs<'a> = ErasedSoaPtrs<FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        let (descriptors, ptr, capacity, offset) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        ErasedSoaPtrs::dangling(self.field_descriptors())
            .expect("descriptors should have sufficient alignment")
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());
        assert_descriptors(self.field_descriptors(), origin.field_descriptors());

        unsafe { ptrs.offset_from(&origin) }
    }

    type MutPtrs<'a> = ErasedSoaMutPtrs<FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        let (descriptors, ptr, capacity, offset) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        ErasedSoaMutPtrs::dangling(self.field_descriptors())
            .expect("descriptors should have sufficient alignment")
    }

    #[inline]
    unsafe fn ptrs_add_mut<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        offset: usize,
    ) -> Self::MutPtrs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        &self,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());
        assert_descriptors(self.field_descriptors(), origin.field_descriptors());

        unsafe { ptrs.offset_from(&origin) }
    }

    #[inline]
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_swap(&self, mut a: Self::MutPtrs<'_>, mut b: Self::MutPtrs<'_>) {
        assert_descriptors(self.field_descriptors(), a.field_descriptors());
        assert_descriptors(self.field_descriptors(), b.field_descriptors());

        unsafe { a.swap(&mut b) }
    }

    #[inline]
    unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, mut dst: Self::MutPtrs<'_>, len: usize) {
        assert_descriptors(self.field_descriptors(), src.field_descriptors());
        assert_descriptors(self.field_descriptors(), dst.field_descriptors());

        unsafe { dst.copy_from(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, mut dst: Self::MutPtrs<'_>, len: usize) {
        assert_descriptors(self.field_descriptors(), src.field_descriptors());
        assert_descriptors(self.field_descriptors(), dst.field_descriptors());

        unsafe { dst.copy_from_rev(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        mut dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        assert_descriptors(self.field_descriptors(), src.field_descriptors());
        assert_descriptors(self.field_descriptors(), dst.field_descriptors());

        unsafe { dst.copy_from_nonoverlapping(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, _: Self::MutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }

    type NonNullPtrs<'a> = ErasedSoaNonNullPtrs<FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        let (descriptors, ptr, capacity, offset) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaNonNullPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        unsafe { ErasedSoaNonNullPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        let ptr = ptr.as_ptr();
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    type SlicePtrs<'a> = ErasedSoaSlicePtrs<FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_slice_ptrs<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        let (descriptors, buffer, capacity, offset, len) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaSlicePtrs::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }

    #[inline]
    fn slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::SlicePtrs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        unsafe { slice_from_raw_parts(ptrs, len) }
    }

    #[inline]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.into_ptrs()
    }

    type SliceMutPtrs<'a> = ErasedSoaSliceMutPtrs<FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_mut_slice_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        let (descriptors, buffer, capacity, offset, len) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaSliceMutPtrs::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }

    #[inline]
    fn mut_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        unsafe { slice_from_raw_parts_mut(ptrs, len) }
    }

    #[inline]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.into_mut_ptrs()
    }

    #[inline]
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.cast_mut()
    }

    #[inline]
    unsafe fn slices_drop_in_place(&self, _: Self::SliceMutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }
}

unsafe impl<T, D, A> RawSoa for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
{
    type Context = ErasedSoaContext<D, A>;
    type Fields = ErasedSoaFields<A>;
}

unsafe impl<T, D, A> SoaRead for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A, Error: Debug>,
    D: CovariantFieldDescriptors + Clone,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Ptrs<'_, Self>) -> Self {
        assert_descriptors(context.field_descriptors(), src.field_descriptors());

        let fields = src
            .into_iter()
            .map(|src| unsafe { src.deref().into_buffer() });
        let descriptors = context.clone().into_inner();
        Self::try_from_fields_descriptors(fields, descriptors)
            .expect("length of fields should be equal to the length of descriptors")
    }
}

unsafe impl<T, D, A> SoaWrite for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: CovariantFieldDescriptors,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
{
    #[inline]
    unsafe fn write(context: &Self::Context, dst: MutPtrs<'_, Self>, value: Self) {
        assert_descriptors(context.field_descriptors(), dst.field_descriptors());
        assert_descriptors(context.field_descriptors(), value.field_descriptors());

        dst.into_iter()
            .zip(value.as_fields())
            .for_each(|(dst, src)| unsafe { dst.copy_from_nonoverlapping(src.as_field_ptr(), 1) });
    }
}

unsafe impl<D> AllocSoaContext for ErasedSoaContext<D, u8>
where
    D: CovariantFieldDescriptors,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
{
    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let descriptors = self.field_descriptors();
        let layout = unsafe { self.buffer_layout(capacity).unwrap_unchecked() };
        let buffer = ptr::slice_from_raw_parts(buffer.cast(), layout.size());
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, buffer, capacity, 0) }
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let descriptors = self.field_descriptors();
        let layout = unsafe { self.buffer_layout(capacity).unwrap_unchecked() };
        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), layout.size());
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, 0) }
    }
}

unsafe impl<'data, D, A> SoaContext<'data> for ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
{
    type Refs<'a> = ErasedSoaRefs<'data, FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        let (descriptors, buffer, capacity, offset) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        unsafe { ptrs.deref() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        assert_descriptors(self.field_descriptors(), refs.field_descriptors());

        refs.into_ptrs()
    }

    type RefsMut<'a> = ErasedSoaRefsMut<'data, FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        let (descriptors, buffer, capacity, offset) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaRefsMut::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        assert_descriptors(self.field_descriptors(), ptrs.field_descriptors());

        unsafe { ptrs.deref_mut() }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        assert_descriptors(self.field_descriptors(), refs.field_descriptors());

        refs.into_mut_ptrs()
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        assert_descriptors(self.field_descriptors(), refs.field_descriptors());

        let (descriptors, buffer, capacity, offset) = refs.into_parts();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    type Slices<'a> = ErasedSoaSlices<'data, FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        let (descriptors, buffer, capacity, offset, len) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        unsafe { slices.deref() }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.into_ptrs()
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.len()
    }

    type SlicesMut<'a> = ErasedSoaSlicesMut<'data, FieldDescriptorsOutput<'a, D>, A>;

    #[inline]
    fn upcast_mut_slices<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        let (descriptors, buffer, capacity, offset, len) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaSlicesMut::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }

    #[inline]
    unsafe fn mut_slice_ptrs_to_mut_slices<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        unsafe { slices.deref_mut() }
    }

    #[inline]
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.into_mut_ptrs()
    }

    #[inline]
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        assert_descriptors(self.field_descriptors(), slices.field_descriptors());

        let (descriptors, buffer, capacity, offset, len) = slices.into_parts();
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }
}

impl<'me, T, D, A> SoaAsRefs<'me> for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
{
    #[inline]
    fn as_refs(&'me self, context: &'me Self::Context) -> Refs<'me, 'me, Self> {
        assert_descriptors(context.field_descriptors(), self.field_descriptors());

        self.as_fields()
    }
}

impl<'me, T, D, A> SoaAsMutRefs<'me> for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
{
    #[inline]
    fn as_mut_refs(&'me mut self, context: &'me Self::Context) -> RefsMut<'me, 'me, Self> {
        assert_descriptors(context.field_descriptors(), self.field_descriptors());

        self.as_mut_fields()
    }
}
