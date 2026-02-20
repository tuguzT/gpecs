use core::{fmt::Debug, mem::MaybeUninit, ptr};

use itertools::zip_eq;

use crate::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs,
    ErasedSoaMutRefs, ErasedSoaMutSlicePtrs, ErasedSoaMutSlices, ErasedSoaNonNullPtrs,
    ErasedSoaPtrs, ErasedSoaRefs, ErasedSoaSlicePtrs, ErasedSoaSlices,
    ptr::slice::SliceItemPtrs,
    soa::{
        field::{FieldDescriptors, FieldDescriptorsOutput},
        traits::{
            AllocSoaContext, MutPtrs, Ptrs, RawSoa, RawSoaContext, Refs, RefsMut, SoaAsMutRefs,
            SoaAsRefs, SoaContext, SoaRead, SoaWrite,
        },
    },
    storage::{AlignedStorage, AlignedStorageFromLayout},
};

unsafe impl<D, P, U> RawSoaContext for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
    P: SliceItemPtrs<Item = MaybeUninit<U>>,
{
    type Ptrs<'a> = ErasedSoaPtrs<FieldDescriptorsOutput<'a, D>, P::Const>;

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
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(&origin) }
    }

    type MutPtrs<'a> = ErasedSoaMutPtrs<FieldDescriptorsOutput<'a, D>, P::Mut>;

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
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        &self,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(&origin) }
    }

    #[inline]
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_swap(&self, mut a: Self::MutPtrs<'_>, mut b: Self::MutPtrs<'_>) {
        unsafe { a.swap(&mut b) }
    }

    #[inline]
    unsafe fn ptrs_copy_forward(
        &self,
        src: Self::Ptrs<'_>,
        mut dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { dst.copy_from_forward(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_backward(
        &self,
        src: Self::Ptrs<'_>,
        mut dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { dst.copy_from_backward(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        mut dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { dst.copy_from_nonoverlapping(&src, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, _: Self::MutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }

    type NonNullPtrs<'a> = ErasedSoaNonNullPtrs<FieldDescriptorsOutput<'a, D>, P::NonNull>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        let (descriptors, ptr, capacity, offset) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaNonNullPtrs::from_parts(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        unsafe { ErasedSoaNonNullPtrs::new_unchecked(ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.into()
    }

    type SlicePtrs<'a> = ErasedSoaSlicePtrs<FieldDescriptorsOutput<'a, D>, P::Const>;

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
        unsafe { ErasedSoaSlicePtrs::from_ptrs(ptrs, len) }
    }

    #[inline]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs()
    }

    type SliceMutPtrs<'a> = ErasedSoaMutSlicePtrs<FieldDescriptorsOutput<'a, D>, P::Mut>;

    #[inline]
    fn upcast_mut_slice_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        let (descriptors, buffer, capacity, offset, len) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaMutSlicePtrs::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }

    #[inline]
    fn mut_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a> {
        unsafe { ErasedSoaMutSlicePtrs::from_ptrs(ptrs, len) }
    }

    #[inline]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
        slices.into_mut_ptrs()
    }

    #[inline]
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
        slices.cast_mut()
    }

    #[inline]
    unsafe fn slices_drop_in_place(&self, _: Self::SliceMutPtrs<'_>) {
        // do nothing; it's safe to not drop anything
    }
}

unsafe impl<T, D, P, U> RawSoa for ErasedSoa<T, D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
    P: SliceItemPtrs<Item = MaybeUninit<U>>,
{
    type Context = ErasedSoaContext<D, P>;
    type Fields = ErasedSoaFields<U>;
}

unsafe impl<T, D, P, U> SoaRead for ErasedSoa<T, D, P>
where
    T: AlignedStorageFromLayout<Item = U, Error: Debug>,
    D: CovariantFieldDescriptors + Clone,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
    P: SliceItemPtrs<Item = MaybeUninit<U>>,
    U: Copy,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Ptrs<'_, Self>) -> Self {
        let fields = src
            .into_iter()
            .map(|src| unsafe { src.deref().into_buffer() });
        let descriptors = context.clone().into_inner();
        Self::try_from_fields_descriptors(fields, descriptors)
            .expect("length of fields should be equal to the length of descriptors")
    }
}

unsafe impl<T, D, P, U> SoaWrite for ErasedSoa<T, D, P>
where
    T: AlignedStorage<Item = U>,
    D: CovariantFieldDescriptors,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
    P: SliceItemPtrs<Item = MaybeUninit<U>>,
{
    #[inline]
    unsafe fn write(_: &Self::Context, dst: MutPtrs<'_, Self>, value: Self) {
        zip_eq(dst, value.as_fields())
            .for_each(|(dst, src)| unsafe { dst.copy_from_nonoverlapping(src.as_field_ptr(), 1) });
    }
}

unsafe impl<D, P> AllocSoaContext for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
    P: SliceItemPtrs<Item = MaybeUninit<u8>>,
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

unsafe impl<'data, D, P, U> SoaContext<'data> for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
    P: SliceItemPtrs<Item = MaybeUninit<U>>,
    U: 'data,
{
    type Refs<'a> = ErasedSoaRefs<'data, FieldDescriptorsOutput<'a, D>, P::Const>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        let (descriptors, buffer, capacity, offset) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.deref() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs()
    }

    type RefsMut<'a> = ErasedSoaMutRefs<'data, FieldDescriptorsOutput<'a, D>, P::Mut>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        let (descriptors, buffer, capacity, offset) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaMutRefs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.deref_mut() }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        refs.into_mut_ptrs()
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs.into()
    }

    type Slices<'a> = ErasedSoaSlices<'data, FieldDescriptorsOutput<'a, D>, P::Const>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        let (descriptors, buffer, capacity, offset, len) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaSlices::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
        unsafe { slices.deref() }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        slices.into_ptrs()
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    type SlicesMut<'a> = ErasedSoaMutSlices<'data, FieldDescriptorsOutput<'a, D>, P::Mut>;

    #[inline]
    fn upcast_mut_slices<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        let (descriptors, buffer, capacity, offset, len) = from.into_parts();
        let descriptors = D::upcast_field_descriptors(descriptors);
        unsafe { ErasedSoaMutSlices::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }

    #[inline]
    unsafe fn mut_slice_ptrs_to_mut_slices<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a> {
        unsafe { slices.deref_mut() }
    }

    #[inline]
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a> {
        slices.into_mut_ptrs()
    }

    #[inline]
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        slices.into()
    }
}

impl<'me, T, D, P, U> SoaAsRefs<'me> for ErasedSoa<T, D, P>
where
    T: AlignedStorage<Item = U>,
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
    P: SliceItemPtrs<Item = MaybeUninit<U>>,
    U: 'me,
{
    #[inline]
    fn as_refs(&'me self, _: &'me Self::Context) -> Refs<'me, 'me, Self> {
        self.as_fields()
    }
}

impl<'me, T, D, P, U> SoaAsMutRefs<'me> for ErasedSoa<T, D, P>
where
    T: AlignedStorage<Item = U>,
    D: CovariantFieldDescriptors + ?Sized,
    for<'a, 'b> FieldDescriptorsOutput<'a, D>: FieldDescriptors<'b> + Clone,
    P: SliceItemPtrs<Item = MaybeUninit<U>>,
    U: 'me,
{
    #[inline]
    fn as_mut_refs(&'me mut self, _: &'me Self::Context) -> RefsMut<'me, 'me, Self> {
        self.as_mut_fields()
    }
}
