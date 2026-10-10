use core::{fmt::Debug, ptr::NonNull};

use gpecs_component::registry::ComponentId;
use gpecs_soa_erased::{
    CovariantFieldLayouts, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs,
    ErasedSoaNonNullPtrs, ErasedSoaPtrs,
    ptr::slice::SliceItemPtrs,
    soa::{
        field::{FieldLayouts, FieldLayoutsOutput},
        traits::{
            SoaAllocContext, SoaContext, SoaRaw, SoaRawContext, SoaReadContext, SoaWriteContext,
        },
    },
    storage::{AlignedStorage, AlignedStorageFromLayout},
};
use itertools::zip_eq;

use crate::{
    bundle::erased::{
        ErasedBorrowedViewBundle, ErasedBundleKind, ErasedBundleMutPtrs, ErasedBundleMutRefs,
        ErasedBundleMutSlicePtrs, ErasedBundleMutSlices, ErasedBundleNonNullPtrs,
        ErasedBundleNonNullSlicePtrs, ErasedBundlePtrs, ErasedBundleRefs, ErasedBundleSlicePtrs,
        ErasedBundleSlices,
        traits::{ErasedArchetypeKind, ErasedArchetypeMeta, ErasedBundleDrop},
    },
    erased::{ErasedArchetypeView, Iter},
};

unsafe impl<'view, T, D, S, P> SoaRawContext<ErasedBundleKind<T, D, S, P>>
    for ErasedSoaContext<ErasedArchetypeView<'view, T::Meta>, P>
where
    T: ErasedArchetypeKind + ?Sized,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Ptrs<'a> = ErasedBundlePtrs<ErasedArchetypeView<'view, T::Meta>, P::Const>;

    #[inline]
    fn ptrs_upcast<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
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
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, count: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(count) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(&origin) }
    }

    type MutPtrs<'a> = ErasedBundleMutPtrs<ErasedArchetypeView<'view, T::Meta>, P::Mut>;

    #[inline]
    fn mut_ptrs_upcast<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn mut_ptrs_dangling(&self) -> Self::MutPtrs<'_> {
        let archetype = *self.as_inner();
        let inner = ErasedSoaMutPtrs::dangling(archetype)
            .expect("archetype components should have sufficient alignment");
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn mut_ptrs_add<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        count: usize,
    ) -> Self::MutPtrs<'a> {
        unsafe { ptrs.add(count) }
    }

    #[inline]
    unsafe fn mut_ptrs_offset_from(
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
    unsafe fn ptrs_swap_nonoverlapping(
        &self,
        mut x: Self::MutPtrs<'_>,
        mut y: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { x.swap_nonoverlapping(&mut y, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        mut dst: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_nonoverlapping(&src, count) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, to_drop: Self::MutPtrs<'_>) {
        let archetype = self.as_inner();
        for ((_, meta), to_drop) in zip_eq(archetype, to_drop) {
            unsafe { D::drop_in_place_with(to_drop, meta) }
        }
    }

    type NonNullPtrs<'a> = ErasedBundleNonNullPtrs<ErasedArchetypeView<'view, T::Meta>, P::NonNull>;

    #[inline]
    fn nonnull_ptrs_upcast<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    fn nonnull_ptrs_dangling(&self) -> Self::NonNullPtrs<'_> {
        let archetype = *self.as_inner();
        let inner = ErasedSoaNonNullPtrs::dangling(archetype)
            .expect("archetype components should have sufficient alignment");
        unsafe { ErasedBundleNonNullPtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_ptrs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::NonNullPtrs<'a> {
        unsafe { ErasedBundleNonNullPtrs::new_unchecked(ptrs.cast_mut()) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_mut_ptrs<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
    ) -> Self::NonNullPtrs<'a> {
        unsafe { ErasedBundleNonNullPtrs::new_unchecked(ptrs) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_add<'a>(
        &'a self,
        ptrs: Self::NonNullPtrs<'a>,
        count: usize,
    ) -> Self::NonNullPtrs<'a> {
        unsafe { ptrs.add(count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_offset_from(
        &self,
        ptrs: Self::NonNullPtrs<'_>,
        origin: Self::NonNullPtrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(&origin) }
    }

    #[inline]
    fn nonnull_ptrs_as_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::Ptrs<'a> {
        ptrs.into_ptrs()
    }

    #[inline]
    fn nonnull_ptrs_as_mut_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.into_mut_ptrs()
    }

    #[inline]
    unsafe fn nonnull_ptrs_swap_nonoverlapping(
        &self,
        mut x: Self::NonNullPtrs<'_>,
        mut y: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { x.swap_nonoverlapping(&mut y, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_nonoverlapping(
        &self,
        src: Self::NonNullPtrs<'_>,
        mut dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_nonoverlapping(&src, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_drop_in_place(&self, to_drop: Self::NonNullPtrs<'_>) {
        let archetype = self.as_inner();
        for ((_, meta), to_drop) in zip_eq(archetype, to_drop) {
            unsafe { D::drop_in_place_with(to_drop.into_mut_ptr(), meta) }
        }
    }

    type SlicePtrs<'a> = ErasedBundleSlicePtrs<ErasedArchetypeView<'view, T::Meta>, P::Const>;

    #[inline]
    fn slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        from
    }

    #[inline]
    fn slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::SlicePtrs<'a> {
        unsafe { ErasedBundleSlicePtrs::from_ptrs(data, len) }
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
    fn mut_slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        from
    }

    #[inline]
    fn mut_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a> {
        unsafe { ErasedBundleMutSlicePtrs::from_ptrs(data, len) }
    }

    #[inline]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs().cast_const()
    }

    #[inline]
    fn mut_slice_ptrs_as_mut_ptrs<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::MutPtrs<'a> {
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
    unsafe fn slices_drop_in_place(&self, slices_to_drop: Self::SliceMutPtrs<'_>) {
        let archetype = self.as_inner();
        for ((_, meta), to_drop) in zip_eq(archetype, slices_to_drop) {
            unsafe { D::drop_in_place_slice_with(to_drop, meta) }
        }
    }

    type SliceNonNullPtrs<'a> =
        ErasedBundleNonNullSlicePtrs<ErasedArchetypeView<'view, T::Meta>, P::NonNull>;

    #[inline]
    fn nonnull_slice_ptrs_upcast<'short, 'long: 'short>(
        from: Self::SliceNonNullPtrs<'long>,
    ) -> Self::SliceNonNullPtrs<'short> {
        from
    }

    #[inline]
    fn nonnull_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        data: Self::NonNullPtrs<'a>,
        len: usize,
    ) -> Self::SliceNonNullPtrs<'a> {
        unsafe { ErasedBundleNonNullSlicePtrs::from_ptrs(data, len) }
    }

    #[inline]
    fn nonnull_slice_ptrs_len(&self, slices: &Self::SliceNonNullPtrs<'_>) -> usize {
        slices.len()
    }
}

unsafe impl<'a, Meta, D, S, P> SoaRaw for ErasedBorrowedViewBundle<'a, Meta, D, S, P>
where
    Meta: ErasedArchetypeMeta,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Context = ErasedSoaContext<ErasedArchetypeView<'a, Meta>, P>;
    type Fields = ErasedSoaFields<P::Item>;
}

unsafe impl<'me, 'a, T, D, S, P>
    SoaReadContext<
        'me,
        ErasedBundleKind<T, D, S, P>,
        ErasedBorrowedViewBundle<'a, T::Meta, D, S, P>,
    > for ErasedSoaContext<ErasedArchetypeView<'a, T::Meta>, P>
where
    T: ErasedArchetypeKind,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Clone, Error: Debug>,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    unsafe fn ptrs_read(
        &'me self,
        src: Self::Ptrs<'me>,
    ) -> ErasedBorrowedViewBundle<'a, T::Meta, D, S, P> {
        let bundle = unsafe { src.read() };
        bundle.expect("erased bundle should be created successfully")
    }

    #[inline]
    unsafe fn mut_ptrs_read(
        &'me self,
        src: Self::MutPtrs<'me>,
    ) -> ErasedBorrowedViewBundle<'a, T::Meta, D, S, P> {
        let bundle = unsafe { src.read() };
        bundle.expect("erased bundle should be created successfully")
    }

    #[inline]
    unsafe fn nonnull_ptrs_read(
        &'me self,
        src: Self::NonNullPtrs<'me>,
    ) -> ErasedBorrowedViewBundle<'a, T::Meta, D, S, P> {
        let bundle = unsafe { src.read() };
        bundle.expect("erased bundle should be created successfully")
    }
}

unsafe impl<T, W, D, N, S, U, P>
    SoaWriteContext<ErasedBundleKind<T, D, S, P>, ErasedBundleKind<W, N, U, P>>
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
    unsafe fn ptrs_write(&self, mut dst: Self::MutPtrs<'_>, bundle: ErasedBundleKind<W, N, U, P>) {
        unsafe { dst.write(bundle) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_write(
        &self,
        dst: Self::NonNullPtrs<'_>,
        bundle: ErasedBundleKind<W, N, U, P>,
    ) {
        unsafe { dst.into_mut_ptrs().write(bundle) }
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
    type OutputIter = Iter<'a, T::Meta>;
    type OutputItem = (ComponentId, &'a T::Meta);

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

unsafe impl<T, D, S, P> SoaAllocContext<ErasedBundleKind<T, D, S, P>>
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
        let inner = unsafe { inner.map_layouts(|_| *self.as_inner()) };
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn mut_ptrs_from_buffer(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let inner = unsafe { self.mut_ptrs_from_buffer(buffer, capacity) };
        let inner = unsafe { inner.map_layouts(|_| *self.as_inner()) };
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_buffer(
        &self,
        buffer: NonNull<u8>,
        capacity: usize,
    ) -> Self::NonNullPtrs<'_> {
        let inner = unsafe { self.nonnull_ptrs_from_buffer(buffer, capacity) };
        let inner = unsafe { inner.map_layouts(|_| *self.as_inner()) };
        unsafe { ErasedBundleNonNullPtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn ptrs_copy_forward(
        &self,
        src: Self::Ptrs<'_>,
        mut dst: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_forward(&src, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_forward(
        &self,
        src: Self::NonNullPtrs<'_>,
        mut dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_forward(&src, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_backward(
        &self,
        src: Self::Ptrs<'_>,
        mut dst: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_backward(&src, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_backward(
        &self,
        src: Self::NonNullPtrs<'_>,
        mut dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_backward(&src, count) }
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
    fn refs_upcast<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    #[inline]
    unsafe fn refs_from_ptrs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.as_ref_unchecked() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs()
    }

    type RefsMut<'a> = ErasedBundleMutRefs<'data, ErasedArchetypeView<'view, T::Meta>, P::Mut>;

    #[inline]
    fn mut_refs_upcast<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
    }

    #[inline]
    unsafe fn mut_refs_from_mut_ptrs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.as_mut_unchecked() }
    }

    #[inline]
    fn mut_refs_as_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs()
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        refs.into_mut_ptrs()
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs.into_refs()
    }

    type Slices<'a> = ErasedBundleSlices<'data, ErasedArchetypeView<'view, T::Meta>, P::Const>;

    #[inline]
    fn slices_upcast<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
    }

    #[inline]
    unsafe fn slices_from_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicePtrs<'a>,
    ) -> Self::Slices<'a> {
        unsafe { slices.as_ref_unchecked() }
    }

    #[inline]
    unsafe fn slices_from_raw_parts<'a>(
        &'a self,
        data: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::Slices<'a> {
        unsafe { ErasedBundleSlices::from_ptrs(data, len) }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        slices.into_slice_ptrs()
    }

    #[inline]
    fn slices_as_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs()
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    type SlicesMut<'a> = ErasedBundleMutSlices<'data, ErasedArchetypeView<'view, T::Meta>, P::Mut>;

    #[inline]
    fn mut_slices_upcast<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        from
    }

    #[inline]
    unsafe fn mut_slices_from_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a> {
        unsafe { slices.as_mut_unchecked() }
    }

    #[inline]
    unsafe fn mut_slices_from_raw_parts<'a>(
        &'a self,
        data: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SlicesMut<'a> {
        unsafe { ErasedBundleMutSlices::from_ptrs(data, len) }
    }

    #[inline]
    fn mut_slices_as_slice_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::SlicePtrs<'a> {
        slices.into_slice_ptrs()
    }

    #[inline]
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a> {
        slices.into_mut_slice_ptrs()
    }

    #[inline]
    fn mut_slices_as_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs()
    }

    #[inline]
    fn mut_slices_as_mut_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::MutPtrs<'a> {
        slices.into_mut_ptrs()
    }

    #[inline]
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        slices.into_slices()
    }
}
