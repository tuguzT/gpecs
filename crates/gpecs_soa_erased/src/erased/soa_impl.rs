use core::{
    fmt::Debug,
    ptr::{self, NonNull},
};

use crate::{
    soa::{
        field::FieldDescriptor,
        traits::{
            AllocSoaContext, MutPtrs, Ptrs, RawSoa, RawSoaContext, Refs, RefsMut, SoaAsMutRefs,
            SoaAsRefs, SoaContext, SoaRead, SoaWrite,
        },
    },
    storage::{AddressableUnit, AlignedStorage, AlignedStorageFromLayout},
};

use super::{
    ErasedSoa, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs, ErasedSoaNonNullPtrs,
    ErasedSoaPtrs, ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSliceMutPtrs, ErasedSoaSlicePtrs,
    ErasedSoaSlices, ErasedSoaSlicesMut, assert::debug_assert_eq_descriptors, slice_from_raw_parts,
    slice_from_raw_parts_mut,
};

unsafe impl<D, A> RawSoaContext for ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Ptrs<'a> = ErasedSoaPtrs<&'a [FieldDescriptor], A>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        let descriptors = self.field_descriptors();
        ErasedSoaPtrs::dangling(descriptors).expect("descriptors should have sufficient alignment")
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());
        debug_assert_eq_descriptors(descriptors, origin.field_descriptors());

        unsafe { ptrs.offset_from(&origin) }
    }

    type MutPtrs<'a> = ErasedSoaMutPtrs<&'a [FieldDescriptor], A>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        let descriptors = self.field_descriptors();
        ErasedSoaMutPtrs::dangling(descriptors)
            .expect("descriptors should have sufficient alignment")
    }

    #[inline]
    unsafe fn ptrs_add_mut<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        offset: usize,
    ) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        &self,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());
        debug_assert_eq_descriptors(descriptors, origin.field_descriptors());

        unsafe { ptrs.offset_from(&origin) }
    }

    #[inline]
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_swap(&self, mut a: Self::MutPtrs<'_>, mut b: Self::MutPtrs<'_>) {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, a.field_descriptors());
        debug_assert_eq_descriptors(descriptors, b.field_descriptors());

        unsafe { a.swap(&mut b) }
    }

    #[inline]
    unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, mut dst: Self::MutPtrs<'_>, len: usize) {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, src.field_descriptors());
        debug_assert_eq_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, mut dst: Self::MutPtrs<'_>, len: usize) {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, src.field_descriptors());
        debug_assert_eq_descriptors(descriptors, dst.field_descriptors());

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
        debug_assert_eq_descriptors(descriptors, src.field_descriptors());
        debug_assert_eq_descriptors(descriptors, dst.field_descriptors());

        unsafe { dst.copy_from_nonoverlapping(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, _: Self::MutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }

    type NonNullPtrs<'a> = ErasedSoaNonNullPtrs<&'a [FieldDescriptor], A>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        unsafe { ErasedSoaNonNullPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        let (descriptors, ptr, capacity, offset) = ptrs.into_parts();
        let ptr = ptr.as_ptr();
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    type SlicePtrs<'a> = ErasedSoaSlicePtrs<&'a [FieldDescriptor], A>;

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
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { slice_from_raw_parts(ptrs, len) }
    }

    #[inline]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.into_ptrs()
    }

    type SliceMutPtrs<'a> = ErasedSoaSliceMutPtrs<&'a [FieldDescriptor], A>;

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
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { slice_from_raw_parts_mut(ptrs, len) }
    }

    #[inline]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.into_mut_ptrs()
    }

    #[inline]
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

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
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Context = ErasedSoaContext<D, A>;
    type Fields = ErasedSoaFields;
}

unsafe impl<T, D, A> SoaRead for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug,
    D: AsRef<[FieldDescriptor]> + Clone,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Ptrs<'_, Self>) -> Self {
        let descriptors = context.field_descriptors();
        debug_assert_eq_descriptors(descriptors, src.field_descriptors());

        let fields = src
            .into_iter()
            .map(|src| unsafe { src.deref().into_buffer() });
        let descriptors = context.clone().into_field_descriptors();
        Self::try_from_fields_descriptors(fields, descriptors)
            .expect("length of fields should be equal to the length of descriptors")
    }
}

unsafe impl<T, D, A> SoaWrite for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    unsafe fn write(context: &Self::Context, dst: MutPtrs<'_, Self>, value: Self) {
        let descriptors = context.field_descriptors();
        debug_assert_eq_descriptors(descriptors, dst.field_descriptors());
        debug_assert_eq_descriptors(descriptors, value.field_descriptors());

        dst.into_iter()
            .zip(value.as_fields())
            .for_each(|(dst, src)| unsafe { dst.copy_from_nonoverlapping(src.as_field_ptr(), 1) });
    }
}

unsafe impl<D> AllocSoaContext for ErasedSoaContext<D, u8>
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

    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let descriptors = self.field_descriptors();
        let layout = unsafe { self.buffer_layout(capacity).unwrap_unchecked() };
        let buffer = ptr::slice_from_raw_parts(buffer, layout.size());
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, buffer, capacity, 0) }
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let descriptors = self.field_descriptors();
        let layout = unsafe { self.buffer_layout(capacity).unwrap_unchecked() };
        let buffer = ptr::slice_from_raw_parts_mut(buffer, layout.size());
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, 0) }
    }
}

unsafe impl<'data, D, A> SoaContext<'data> for ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Refs<'a> = ErasedSoaRefs<'data, &'a [FieldDescriptor], A>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.deref() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, refs.field_descriptors());

        refs.into_ptrs()
    }

    type RefsMut<'a> = ErasedSoaRefsMut<'data, &'a [FieldDescriptor], A>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
    }

    #[inline]
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, ptrs.field_descriptors());

        unsafe { ptrs.deref_mut() }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, refs.field_descriptors());

        refs.into_mut_ptrs()
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, refs.field_descriptors());

        let (descriptors, buffer, capacity, offset) = refs.into_parts();
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    type Slices<'a> = ErasedSoaSlices<'data, &'a [FieldDescriptor], A>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        unsafe { slices.deref() }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.into_ptrs()
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    type SlicesMut<'a> = ErasedSoaSlicesMut<'data, &'a [FieldDescriptor], A>;

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
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        unsafe { slices.deref_mut() }
    }

    #[inline]
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.into_mut_ptrs()
    }

    #[inline]
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        slices.len()
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        let descriptors = self.field_descriptors();
        debug_assert_eq_descriptors(descriptors, slices.field_descriptors());

        let (descriptors, buffer, capacity, offset, len) = slices.into_parts();
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }
}

impl<'a, T, D, A> SoaAsRefs<'a> for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    fn as_refs(&'a self, context: &'a Self::Context) -> Refs<'a, 'a, Self> {
        let descriptors = context.field_descriptors();
        debug_assert_eq_descriptors(descriptors, self.field_descriptors());

        self.as_fields()
    }
}

impl<'a, T, D, A> SoaAsMutRefs<'a> for ErasedSoa<T, D, A>
where
    A: AddressableUnit,
    T: AlignedStorage<A>,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    fn as_mut_refs(&'a mut self, context: &'a Self::Context) -> RefsMut<'a, 'a, Self> {
        let descriptors = context.field_descriptors();
        debug_assert_eq_descriptors(descriptors, self.field_descriptors());

        self.as_mut_fields()
    }
}
