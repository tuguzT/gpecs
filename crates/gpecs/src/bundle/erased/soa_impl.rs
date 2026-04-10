use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoaContext, ErasedSoaFields, ErasedSoaMutPtrs, ErasedSoaPtrs,
    ptr::slice::CoreSliceItemPtrs,
};
use itertools::zip_eq;

use crate::{
    archetype::erased::{ErasedArchetype, ErasedArchetypeView},
    bundle::erased::{
        ErasedBorrowedBundle, ErasedBorrowedViewBundle, ErasedBundle, ErasedBundleKind,
        ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundleMutSlicePtrs, ErasedBundleMutSlices,
        ErasedBundleNonNullPtrs, ErasedBundlePtrs, ErasedBundleRefs, ErasedBundleSlicePtrs,
        ErasedBundleSlices, traits::ErasedArchetypeKind,
    },
    component::erased::WithErasedDrop,
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
        traits::{
            AllocSoaContext, RawSoa, RawSoaContext, ReadSoaContext, SoaContext, WriteSoaContext,
        },
    },
};

unsafe impl<T> RawSoaContext<ErasedBundleKind<T>> for ErasedArchetypeView<'_, T::Meta>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop> + ?Sized,
{
    type Ptrs<'a> = ErasedBundlePtrs<Self>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        let inner = ErasedSoaPtrs::dangling(*self)
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

    type MutPtrs<'a> = ErasedBundleMutPtrs<Self>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        let inner = ErasedSoaMutPtrs::dangling(*self)
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
        for (component_info, to_drop) in zip_eq(self, ptrs) {
            let Some(erased_drop) = component_info.erased_drop() else {
                continue;
            };
            unsafe { erased_drop.drop_in_place(to_drop) }
        }
    }

    type NonNullPtrs<'a> = ErasedBundleNonNullPtrs<Self>;

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

    type SlicePtrs<'a> = ErasedBundleSlicePtrs<Self>;

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

    type SliceMutPtrs<'a> = ErasedBundleMutSlicePtrs<Self>;

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
        for (component_info, to_drop) in zip_eq(self, slices) {
            let Some(erased_drop) = component_info.erased_drop() else {
                continue;
            };
            unsafe { erased_drop.drop_in_place_slice(to_drop) }
        }
    }
}

unsafe impl<'a, Meta> RawSoa for ErasedBorrowedViewBundle<'a, Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    type Context = ErasedArchetypeView<'a, Meta>;
    type Fields = ErasedSoaFields<u8>;
}

unsafe impl<'me, 'a, T>
    ReadSoaContext<'me, ErasedBorrowedViewBundle<'a, T::Meta>, ErasedBundleKind<T>>
    for ErasedArchetypeView<'a, T::Meta>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop>,
{
    #[inline]
    unsafe fn read(&'me self, src: Self::Ptrs<'me>) -> ErasedBorrowedViewBundle<'a, T::Meta> {
        let bundle = unsafe { src.read() };
        bundle.expect("erased bundle should be created successfully")
    }
}

unsafe impl<T, W> WriteSoaContext<ErasedBundleKind<W>, ErasedBundleKind<T>>
    for ErasedArchetypeView<'_, T::Meta>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop> + ?Sized,
    W: ErasedArchetypeKind<Meta: WithErasedDrop>,
{
    #[inline]
    unsafe fn write(&self, mut dst: Self::MutPtrs<'_>, bundle: ErasedBundleKind<W>) {
        unsafe { dst.write(bundle) }
    }
}

impl<'a, T> FieldDescriptors<'a, ErasedBundleKind<T>> for ErasedArchetypeView<'_, T::Meta>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop> + ?Sized,
{
    type Output = ErasedArchetypeView<'a, T::Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        *self
    }
}

impl<T> CovariantFieldDescriptors<ErasedBundleKind<T>> for ErasedArchetypeView<'_, T::Meta>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop> + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self, ErasedBundleKind<T>>,
    ) -> FieldDescriptorsOutput<'short, Self, ErasedBundleKind<T>> {
        from
    }
}

unsafe impl<T> AllocSoaContext<ErasedBundleKind<T>> for ErasedArchetypeView<'_, T::Meta>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop> + ?Sized,
{
    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let context = unsafe { ErasedSoaContext::<_, CoreSliceItemPtrs<_>>::from_inner_ref(self) };
        let inner = unsafe { context.ptrs_from_buffer(buffer, capacity) };

        let (_, buffer, capacity, offset) = inner.into_parts();
        let inner = unsafe { ErasedSoaPtrs::new_unchecked(*self, buffer, capacity, offset) };
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let context = unsafe { ErasedSoaContext::<_, CoreSliceItemPtrs<_>>::from_inner_ref(self) };
        let inner = unsafe { context.ptrs_from_buffer_mut(buffer, capacity) };

        let (_, buffer, capacity, offset) = inner.into_parts();
        let inner = unsafe { ErasedSoaMutPtrs::new_unchecked(*self, buffer, capacity, offset) };
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

unsafe impl<'data, T> SoaContext<'data, ErasedBundleKind<T>> for ErasedArchetypeView<'_, T::Meta>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop> + ?Sized,
{
    type Refs<'a> = ErasedBundleRefs<'data, Self>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.deref() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs()
    }

    type RefsMut<'a> = ErasedBundleMutRefs<'data, Self>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
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

    type Slices<'a> = ErasedBundleSlices<'data, Self>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
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

    type SlicesMut<'a> = ErasedBundleMutSlices<'data, Self>;

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

unsafe impl<T> RawSoaContext<ErasedBundleKind<T>> for ErasedArchetype<T::Meta>
where
    T: ErasedArchetypeKind<Meta: WithErasedDrop> + ?Sized,
{
    type Ptrs<'a> = ErasedBundlePtrs<&'a Self>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        let inner = ErasedSoaPtrs::dangling(self)
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

    type MutPtrs<'a> = ErasedBundleMutPtrs<&'a Self>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        let inner = ErasedSoaMutPtrs::dangling(self)
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
        for (component_info, to_drop) in zip_eq(self, ptrs) {
            let Some(erased_drop) = component_info.erased_drop() else {
                continue;
            };
            unsafe { erased_drop.drop_in_place(to_drop) }
        }
    }

    type NonNullPtrs<'a> = ErasedBundleNonNullPtrs<&'a Self>;

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

    type SlicePtrs<'a> = ErasedBundleSlicePtrs<&'a Self>;

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

    type SliceMutPtrs<'a> = ErasedBundleMutSlicePtrs<&'a Self>;

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
        for (component_info, to_drop) in zip_eq(self, slices) {
            let Some(erased_drop) = component_info.erased_drop() else {
                continue;
            };
            unsafe { erased_drop.drop_in_place_slice(to_drop) }
        }
    }
}

unsafe impl<Meta> RawSoa for ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    type Context = ErasedArchetype<Meta>;
    type Fields = ErasedSoaFields<u8>;
}

unsafe impl<'a, Meta> ReadSoaContext<'a, ErasedBorrowedBundle<'a, Meta>, ErasedBundle<Meta>>
    for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    #[inline]
    unsafe fn read(&'a self, src: Self::Ptrs<'a>) -> ErasedBorrowedBundle<'a, Meta> {
        let bundle = unsafe { src.read() };
        bundle.expect("erased bundle should be created successfully")
    }
}

unsafe impl<'a, Meta> ReadSoaContext<'a, ErasedBundle<Meta>, ErasedBundle<Meta>>
    for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + Clone + 'static,
{
    #[inline]
    unsafe fn read(&'a self, src: Self::Ptrs<'a>) -> ErasedBundle<Meta> {
        let bundle: ErasedBorrowedBundle<_> = unsafe { self.read(src) };
        bundle.into()
    }
}

unsafe impl<Meta, W> WriteSoaContext<ErasedBundleKind<W>, ErasedBundle<Meta>>
    for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + Clone + 'static,
    W: ErasedArchetypeKind<Meta: WithErasedDrop>,
{
    #[inline]
    unsafe fn write(&self, mut dst: Self::MutPtrs<'_>, bundle: ErasedBundleKind<W>) {
        unsafe { dst.write(bundle) }
    }
}

impl<'a, Meta> FieldDescriptors<'a, ErasedBundle<Meta>> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    type Output = &'a Self;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self
    }
}

impl<Meta> CovariantFieldDescriptors<ErasedBundle<Meta>> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self, ErasedBundle<Meta>>,
    ) -> FieldDescriptorsOutput<'short, Self, ErasedBundle<Meta>> {
        from
    }
}

unsafe impl<Meta> AllocSoaContext<ErasedBundle<Meta>> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let context = unsafe { ErasedSoaContext::<_, CoreSliceItemPtrs<_>>::from_inner_ref(self) };
        let inner = unsafe { context.ptrs_from_buffer(buffer, capacity) };

        let (_, buffer, capacity, offset) = inner.into_parts();
        let inner = unsafe { ErasedSoaPtrs::new_unchecked(self, buffer, capacity, offset) };
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let context = unsafe { ErasedSoaContext::<_, CoreSliceItemPtrs<_>>::from_inner_ref(self) };
        let inner = unsafe { context.ptrs_from_buffer_mut(buffer, capacity) };

        let (_, buffer, capacity, offset) = inner.into_parts();
        let inner = unsafe { ErasedSoaMutPtrs::new_unchecked(self, buffer, capacity, offset) };
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

unsafe impl<'data, Meta> SoaContext<'data, ErasedBundle<Meta>> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    type Refs<'a> = ErasedBundleRefs<'data, &'a Self>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.deref() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs()
    }

    type RefsMut<'a> = ErasedBundleMutRefs<'data, &'a Self>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
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

    type Slices<'a> = ErasedBundleSlices<'data, &'a Self>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
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

    type SlicesMut<'a> = ErasedBundleMutSlices<'data, &'a Self>;

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
