use core::{fmt::Debug, mem::MaybeUninit};

use crate::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs,
    ErasedSoaMutRefs, ErasedSoaMutSlicePtrs, ErasedSoaMutSlices, ErasedSoaNonNullPtrs,
    ErasedSoaPtrs, ErasedSoaRefs, ErasedSoaSlicePtrs, ErasedSoaSlices,
    ptr::slice::SliceItemPtrs,
    soa::{
        field::{FieldDescriptors, FieldDescriptorsOutput, FieldDescriptorsOwned},
        traits::{
            AllocSoaContext, RawSoa, RawSoaContext, ReadSoaContext, SoaContext, WriteSoaContext,
        },
    },
    storage::{AlignedStorage, AlignedStorageFromLayout},
};

unsafe impl<T, D, P> RawSoaContext<ErasedSoa<T, D, P>> for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors<Output: FieldDescriptorsOwned + Clone> + ?Sized,
    P: SliceItemPtrs,
{
    type Ptrs<'a> = ErasedSoaPtrs<FieldDescriptorsOutput<'a, D>, P::Const>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
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
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
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
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
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
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
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
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
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
        slices.into_ptrs()
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

unsafe impl<T, D, P> RawSoa for ErasedSoa<T, D, P>
where
    D: CovariantFieldDescriptors<Output: FieldDescriptorsOwned + Clone> + ?Sized,
    P: SliceItemPtrs,
{
    type Context = ErasedSoaContext<D, P>;
    type Fields = ErasedSoaFields<P::Item>;
}

unsafe impl<'a, T, D, P>
    ReadSoaContext<'a, ErasedSoa<T, FieldDescriptorsOutput<'a, D>, P>, ErasedSoa<T, D, P>>
    for ErasedSoaContext<D, P>
where
    T: AlignedStorageFromLayout<Item: Copy, Error: Debug>,
    D: CovariantFieldDescriptors<Output: FieldDescriptorsOwned + Clone>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    unsafe fn read(
        &'a self,
        src: Self::Ptrs<'a>,
    ) -> ErasedSoa<T, FieldDescriptorsOutput<'a, D>, P> {
        let value = unsafe { src.read() };
        value.expect("erased SoA should be created successfully")
    }
}

unsafe impl<T, D, N, P> WriteSoaContext<ErasedSoa<T, N, P>, ErasedSoa<T, D, P>>
    for ErasedSoaContext<D, P>
where
    T: AlignedStorage,
    D: CovariantFieldDescriptors<Output: FieldDescriptorsOwned + Clone>,
    N: FieldDescriptorsOwned<Output: FieldDescriptorsOwned>,
    P: SliceItemPtrs<Item = MaybeUninit<T::Item>>,
{
    #[inline]
    unsafe fn write(&self, mut dst: Self::MutPtrs<'_>, value: ErasedSoa<T, N, P>) {
        unsafe { dst.write(value) }
    }
}

impl<'a, T, D, P> FieldDescriptors<'a, ErasedSoa<T, D, P>> for ErasedSoaContext<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: SliceItemPtrs,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        Self::field_descriptors(self)
    }
}

impl<T, D, P> CovariantFieldDescriptors<ErasedSoa<T, D, P>> for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self, ErasedSoa<T, D, P>>,
    ) -> FieldDescriptorsOutput<'short, Self, ErasedSoa<T, D, P>> {
        D::upcast_field_descriptors(from)
    }
}

unsafe impl<T, D, P> AllocSoaContext<ErasedSoa<T, D, P>> for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors<Output: FieldDescriptorsOwned + Clone>,
    P: SliceItemPtrs,
{
    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        unsafe { Self::ptrs_from_buffer(self, buffer, capacity) }
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        unsafe { Self::ptrs_from_buffer_mut(self, buffer, capacity) }
    }
}

unsafe impl<'data, T, D, P> SoaContext<'data, ErasedSoa<T, D, P>> for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors<Output: FieldDescriptorsOwned + Clone> + ?Sized,
    P: SliceItemPtrs<Item: 'data>,
{
    type Refs<'a> = ErasedSoaRefs<'data, FieldDescriptorsOutput<'a, D>, P::Const>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
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
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
    }

    #[inline]
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.deref_mut() }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        refs.into_ptrs()
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs.into()
    }

    type Slices<'a> = ErasedSoaSlices<'data, FieldDescriptorsOutput<'a, D>, P::Const>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
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
        unsafe { from.map_descriptors(D::upcast_field_descriptors) }
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
        slices.into_ptrs()
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
