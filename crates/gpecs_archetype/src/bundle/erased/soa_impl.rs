use core::fmt::Debug;

use gpecs_soa_erased::{
    CovariantFieldLayouts, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs, ErasedSoaPtrs,
    ptr::slice::SliceItemPtrs,
    soa::{
        field::{FieldLayouts, FieldLayoutsOutput},
        layout::WithLayout,
        traits::{
            AllocSoaContext, RawSoa, RawSoaContext, ReadSoaContext, SoaContext, WriteSoaContext,
        },
    },
    storage::{AlignedStorage, AlignedStorageFromLayout},
};
use itertools::zip_eq;

use crate::{
    bundle::erased::{
        ErasedBorrowedViewBundle, ErasedBundleKind, ErasedBundleMutPtrs, ErasedBundleMutRefs,
        ErasedBundleMutSlicePtrs, ErasedBundleMutSlices, ErasedBundleNonNullPtrs, ErasedBundlePtrs,
        ErasedBundleRefs, ErasedBundleSlicePtrs, ErasedBundleSlices,
        traits::{ErasedArchetypeKind, ErasedBundleDrop},
    },
    erased::ErasedArchetypeView,
};

unsafe impl<'view, T, D, S, P> RawSoaContext<ErasedBundleKind<T, D, S, P>>
    for ErasedSoaContext<ErasedArchetypeView<'view, T::Meta>, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Ptrs<'a> = ErasedBundlePtrs<ErasedArchetypeView<'view, T::Meta>, P::Const>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        let archetype = *self.as_inner();
        let inner = ErasedSoaPtrs::dangling(archetype)
            .expect("archetype components should have sufficient alignment");
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(&origin) }
    }

    type MutPtrs<'a> = ErasedBundleMutPtrs<ErasedArchetypeView<'view, T::Meta>, P::Mut>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        let archetype = *self.as_inner();
        let inner = ErasedSoaMutPtrs::dangling(archetype)
            .expect("archetype components should have sufficient alignment");
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
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
    unsafe fn ptrs_drop_in_place(&self, ptrs: Self::MutPtrs<'_>) {
        let archetype = self.as_inner();
        for (component_desc, to_drop) in zip_eq(archetype, ptrs) {
            unsafe { D::drop_in_place_with(to_drop, component_desc.as_meta()) }
        }
    }

    type NonNullPtrs<'a> = ErasedBundleNonNullPtrs<ErasedArchetypeView<'view, T::Meta>, P::NonNull>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        unsafe { ErasedBundleNonNullPtrs::new_unchecked(ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.into()
    }

    type SlicePtrs<'a> = ErasedBundleSlicePtrs<ErasedArchetypeView<'view, T::Meta>, P::Const>;

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
        unsafe { ErasedBundleSlicePtrs::from_ptrs(ptrs, len) }
    }

    #[inline]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs()
    }

    type SliceMutPtrs<'a> = ErasedBundleMutSlicePtrs<ErasedArchetypeView<'view, T::Meta>, P::Mut>;

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
        unsafe { ErasedBundleMutSlicePtrs::from_ptrs(ptrs, len) }
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
    unsafe fn slices_drop_in_place(&self, slices: Self::SliceMutPtrs<'_>) {
        let archetype = self.as_inner();
        for (component_desc, to_drop) in zip_eq(archetype, slices) {
            unsafe { D::drop_in_place_slice_with(to_drop, component_desc.as_meta()) }
        }
    }
}

unsafe impl<'a, Meta, D, S, P> RawSoa for ErasedBorrowedViewBundle<'a, Meta, D, S, P>
where
    Meta: WithLayout + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Context = ErasedSoaContext<ErasedArchetypeView<'a, Meta>, P>;
    type Fields = ErasedSoaFields<P::Item>;
}

unsafe impl<'me, 'a, T, D, S, P>
    ReadSoaContext<
        'me,
        ErasedBorrowedViewBundle<'a, T::Meta, D, S, P>,
        ErasedBundleKind<T, D, S, P>,
    > for ErasedSoaContext<ErasedArchetypeView<'a, T::Meta>, P>
where
    T: ErasedArchetypeKind,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Clone, Error: Debug>,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    unsafe fn read(
        &'me self,
        src: Self::Ptrs<'me>,
    ) -> ErasedBorrowedViewBundle<'a, T::Meta, D, S, P> {
        unsafe { src.read() }.expect("erased bundle should be created successfully")
    }
}

unsafe impl<T, W, D, N, S, U, P>
    WriteSoaContext<ErasedBundleKind<W, N, U, P>, ErasedBundleKind<T, D, S, P>>
    for ErasedSoaContext<ErasedArchetypeView<'_, T::Meta>, P>
where
    T: ErasedArchetypeKind + ?Sized,
    W: ErasedArchetypeKind,
    D: ErasedBundleDrop<T::Meta>,
    N: ErasedBundleDrop<W::Meta>,
    S: AlignedStorage,
    U: AlignedStorage<Item = S::Item>,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    unsafe fn write(&self, mut dst: Self::MutPtrs<'_>, bundle: ErasedBundleKind<W, N, U, P>) {
        unsafe { dst.write(bundle) }
    }
}

impl<'a, T, D, S, P> FieldLayouts<'a, ErasedBundleKind<T, D, S, P>>
    for ErasedSoaContext<ErasedArchetypeView<'_, T::Meta>, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Output = ErasedArchetypeView<'a, T::Meta>;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        *self.as_inner()
    }
}

impl<T, D, S, P> CovariantFieldLayouts<ErasedBundleKind<T, D, S, P>>
    for ErasedSoaContext<ErasedArchetypeView<'_, T::Meta>, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self, ErasedBundleKind<T, D, S, P>>,
    ) -> FieldLayoutsOutput<'short, Self, ErasedBundleKind<T, D, S, P>> {
        from
    }
}

unsafe impl<T, D, S, P> AllocSoaContext<ErasedBundleKind<T, D, S, P>>
    for ErasedSoaContext<ErasedArchetypeView<'_, T::Meta>, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let inner = unsafe { self.ptrs_from_buffer(buffer, capacity) };

        let archetype = *self.as_inner();
        let (_, buffer, capacity, offset) = inner.into_parts();
        let inner = unsafe { ErasedSoaPtrs::new_unchecked(archetype, buffer, capacity, offset) };
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let inner = unsafe { self.ptrs_from_buffer_mut(buffer, capacity) };

        let archetype = *self.as_inner();
        let (_, buffer, capacity, offset) = inner.into_parts();
        let inner = unsafe { ErasedSoaMutPtrs::new_unchecked(archetype, buffer, capacity, offset) };
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

unsafe impl<'data, 'view, T, D, S, P> SoaContext<'data, ErasedBundleKind<T, D, S, P>>
    for ErasedSoaContext<ErasedArchetypeView<'view, T::Meta>, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage<Item: 'data>,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Refs<'a> = ErasedBundleRefs<'data, ErasedArchetypeView<'view, T::Meta>, P::Const>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.as_ref_unchecked() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs()
    }

    type RefsMut<'a> = ErasedBundleMutRefs<'data, ErasedArchetypeView<'view, T::Meta>, P::Mut>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
    }

    #[inline]
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.as_mut_unchecked() }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        refs.into_ptrs()
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs.into()
    }

    type Slices<'a> = ErasedBundleSlices<'data, ErasedArchetypeView<'view, T::Meta>, P::Const>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
        unsafe { slices.as_ref_unchecked() }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        slices.into_ptrs()
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    type SlicesMut<'a> = ErasedBundleMutSlices<'data, ErasedArchetypeView<'view, T::Meta>, P::Mut>;

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
        unsafe { slices.as_mut_unchecked() }
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
