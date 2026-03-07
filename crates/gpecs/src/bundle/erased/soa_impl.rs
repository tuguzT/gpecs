use gpecs_soa_erased::{ErasedSoaContext, ErasedSoaFields, ptr::slice::CoreSliceItemPtrs};
use itertools::zip_eq;

use crate::{
    archetype::erased::ErasedArchetype,
    bundle::erased::{
        ErasedArchetypeKind, ErasedBorrowedBundle, ErasedBundle, ErasedBundleKind,
        ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundleMutSlicePtrs, ErasedBundleMutSlices,
        ErasedBundleNonNullPtrs, ErasedBundlePtrs, ErasedBundleRefs, ErasedBundleSlicePtrs,
        ErasedBundleSlices,
    },
    component::registry::DropFn,
    soa::{
        field::FieldDescriptor,
        traits::{
            AllocSoaContext, RawSoa, RawSoaContext, ReadSoaContext, SoaContext, WriteSoaContext,
        },
    },
};

unsafe impl<Meta> RawSoaContext for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    type Ptrs<'a> = ErasedBundlePtrs<'a, Meta>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        ErasedBundlePtrs::dangling(self)
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(&origin) }
    }

    type MutPtrs<'a> = ErasedBundleMutPtrs<'a, Meta>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        ErasedBundleMutPtrs::dangling(self)
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
        for (component, ptr) in zip_eq(self, ptrs) {
            let Some(drop_fn) = component.meta.as_ref() else {
                continue;
            };

            let ptr = unsafe { ptr.as_mut_ptr().cast() };
            unsafe { drop_fn(ptr) }
        }
    }

    type NonNullPtrs<'a> = ErasedBundleNonNullPtrs<'a, Meta>;

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

    type SlicePtrs<'a> = ErasedBundleSlicePtrs<'a, Meta>;

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

    type SliceMutPtrs<'a> = ErasedBundleMutSlicePtrs<'a, Meta>;

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
        for (component, slice) in zip_eq(self, slices) {
            let Some(drop_fn) = component.meta.as_ref() else {
                continue;
            };

            for i in 0..slice.len() {
                let ptr = unsafe { slice.component_ptr().add(i) };
                let ptr = unsafe { ptr.as_mut_ptr().cast() };
                unsafe { drop_fn(ptr) }
            }
        }
    }
}

unsafe impl<Meta> RawSoa for ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    type Context = ErasedArchetype<Meta>;
    type Fields = ErasedSoaFields<u8>;
}

unsafe impl<'a, Meta> ReadSoaContext<'a, ErasedBorrowedBundle<'a, Meta>> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    #[inline]
    unsafe fn read(&'a self, src: Self::Ptrs<'a>) -> ErasedBorrowedBundle<'a, Meta> {
        let context = unsafe { ErasedSoaContext::from_inner_ref(self) };
        let inner = unsafe { context.read(src.into_inner()) };
        unsafe { ErasedBorrowedBundle::from_inner(inner) }
    }
}

unsafe impl<'a, Meta> ReadSoaContext<'a, ErasedBundle<Meta>> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + Clone + 'static,
{
    #[inline]
    unsafe fn read(&'a self, src: Self::Ptrs<'a>) -> ErasedBundle<Meta> {
        let bundle = unsafe { ReadSoaContext::<ErasedBorrowedBundle<_>>::read(self, src) };
        bundle.into()
    }
}

unsafe impl<Meta, T> WriteSoaContext<ErasedBundleKind<T>> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
    T: ErasedArchetypeKind<Meta = Meta>,
{
    #[inline]
    unsafe fn write(&self, dst: Self::MutPtrs<'_>, bundle: ErasedBundleKind<T>) {
        let src = bundle.as_ptrs();
        unsafe { self.write(dst, src) };
        let _ = bundle.into_inner(); // avoid dropping moved bundle's components
    }
}

unsafe impl<Meta> WriteSoaContext<ErasedBundlePtrs<'_, Meta>> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    #[inline]
    unsafe fn write(&self, dst: Self::MutPtrs<'_>, src: ErasedBundlePtrs<'_, Meta>) {
        let context = unsafe { ErasedSoaContext::<_, CoreSliceItemPtrs<_>>::from_inner_ref(self) };
        let dst = dst.into_inner();
        let src = &src.into_inner();
        unsafe { context.write(dst, src) }
    }
}

unsafe impl<Meta> AllocSoaContext for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let context = unsafe { ErasedSoaContext::<_, CoreSliceItemPtrs<_>>::from_inner_ref(self) };
        let inner = unsafe { context.ptrs_from_buffer(buffer, capacity) };
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let context = unsafe { ErasedSoaContext::<_, CoreSliceItemPtrs<_>>::from_inner_ref(self) };
        let inner = unsafe { context.ptrs_from_buffer_mut(buffer, capacity) };
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

unsafe impl<'data, Meta> SoaContext<'data> for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    type Refs<'a> = ErasedBundleRefs<'data, 'a, Meta>;

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

    type RefsMut<'a> = ErasedBundleMutRefs<'data, 'a, Meta>;

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

    type Slices<'a> = ErasedBundleSlices<'data, 'a, Meta>;

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

    type SlicesMut<'a> = ErasedBundleMutSlices<'data, 'a, Meta>;

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
