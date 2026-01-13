use core::alloc::{Layout, LayoutError};

use crate::{
    item::{
        DenseContext, DenseFieldDescriptors, DenseItem, DenseMutPtrs, DenseNonNullPtrs, DensePtrs,
        DenseRefs, DenseRefsMut, DenseSliceMutPtrs, DenseSlicePtrs, DenseSlices, DenseSlicesMut,
    },
    soa::traits::{
        MutPtrs, Ptrs, RawSoa, RawSoaContext, Refs, RefsMut, Soa, SoaAsMutRefs, SoaAsRefs,
        SoaCloneToUninit, SoaContext, SoaRead, SoaTrustedFields, SoaWrite,
    },
};

unsafe impl<K, V> RawSoaContext for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
{
    type FieldDescriptors<'a> = DenseFieldDescriptors<'a, K, V>;

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        from
    }

    #[inline]
    fn field_descriptors(&self) -> Self::FieldDescriptors<'_> {
        let context = self.as_inner();
        DenseFieldDescriptors::new(context)
    }

    #[inline]
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        let keys = Layout::array::<K>(capacity)?;
        let values = self.as_inner().buffer_layout(capacity)?;
        let (buffer_layout, _) = keys.extend(values)?;
        Ok(buffer_layout)
    }

    type Ptrs<'a> = DensePtrs<'a, K, V>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        let context = self.as_inner();
        DensePtrs::dangling(context)
    }

    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let context = self.as_inner();

        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = buffer.cast();
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { context.ptrs_from_buffer(buffer, capacity) };
        DensePtrs::new(key, value)
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(self, offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(self, origin) }
    }

    type MutPtrs<'a> = DenseMutPtrs<'a, K, V>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        let context = self.as_inner();
        DenseMutPtrs::dangling(context)
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let context = self.as_inner();

        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = buffer.cast();
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { context.ptrs_from_buffer_mut(buffer, capacity) };
        DenseMutPtrs::new(key, value)
    }

    #[inline]
    unsafe fn ptrs_add_mut<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        offset: usize,
    ) -> Self::MutPtrs<'a> {
        unsafe { ptrs.add(self, offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        &self,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(self, origin) }
    }

    #[inline]
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
        ptrs.cast_const(self)
    }

    #[inline]
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.cast_mut(self)
    }

    #[inline]
    unsafe fn ptrs_swap(&self, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        unsafe { a.swap(self, b) }
    }

    #[inline]
    unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
        unsafe { dst.copy_from(self, src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
        unsafe { dst.copy_from_rev(self, src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { dst.copy_from_nonoverlapping(self, src, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, ptrs: Self::MutPtrs<'_>) {
        unsafe { ptrs.drop_in_place(self) }
    }

    type NonNullPtrs<'a> = DenseNonNullPtrs<'a, K, V>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        let context = self.as_inner();
        let (key, value) = ptrs.into_parts();
        unsafe { DenseNonNullPtrs::new_unchecked(context, key, value) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.into_mut_ptrs(self)
    }

    type SlicePtrs<'a> = DenseSlicePtrs<'a, K, V>;

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
        let context = self.as_inner();
        DenseSlicePtrs::from_raw_parts(context, ptrs, len)
    }

    #[inline]
    #[track_caller]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len(self)
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs(self)
    }

    type SliceMutPtrs<'a> = DenseSliceMutPtrs<'a, K, V>;

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
        let context = self.as_inner();
        DenseSliceMutPtrs::from_raw_parts(context, ptrs, len)
    }

    #[inline]
    #[track_caller]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len(self)
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
        slices.into_mut_ptrs(self)
    }

    #[inline]
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
        slices.cast_const(self)
    }

    #[inline]
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
        slices.cast_mut(self)
    }

    #[inline]
    unsafe fn slices_drop_in_place(&self, slices: Self::SliceMutPtrs<'_>) {
        unsafe { slices.drop_in_place(self) }
    }
}

unsafe impl<K, V> RawSoa for DenseItem<K, V>
where
    V: RawSoa + ?Sized,
{
    type Context = DenseContext<K, V>;
    type Fields = (K, V::Fields);
}

unsafe impl<K, V> SoaTrustedFields for DenseItem<K, V> where V: SoaTrustedFields {}

unsafe impl<K, V> SoaCloneToUninit for DenseItem<K, V>
where
    K: Clone,
    V: SoaCloneToUninit + ?Sized,
{
    #[inline]
    unsafe fn clone_to_uninit(
        context: &Self::Context,
        src: Ptrs<'_, Self>,
        dst: MutPtrs<'_, Self>,
    ) {
        unsafe { src.clone_to_uninit(context, dst) }
    }
}

unsafe impl<K, V> SoaRead for DenseItem<K, V>
where
    V: SoaRead,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Ptrs<'_, Self>) -> Self {
        unsafe { src.read(context) }
    }
}

unsafe impl<K, V> SoaWrite for DenseItem<K, V>
where
    V: SoaWrite,
{
    #[inline]
    unsafe fn write(context: &Self::Context, dst: MutPtrs<'_, Self>, value: Self) {
        unsafe { dst.write(context, value) }
    }
}

unsafe impl<'data, K, V> SoaContext<'data> for DenseContext<K, V>
where
    K: 'data,
    V: Soa<'data> + ?Sized,
{
    type Refs<'a> = DenseRefs<'a, 'data, K, V>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        let DenseRefs { key, value } = from;
        let value = V::Context::upcast_refs(value.into_inner());
        DenseRefs::new(key, value)
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.deref(self) }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs(self)
    }

    type RefsMut<'a> = DenseRefsMut<'a, 'data, K, V>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        let DenseRefsMut { key, value } = from;
        let value = V::Context::upcast_mut_refs(value.into_inner());
        DenseRefsMut::new(key, value)
    }

    #[inline]
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.deref_mut(self) }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        refs.into_ptrs(self)
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs.into_refs(self)
    }

    type Slices<'a> = DenseSlices<'a, 'data, K, V>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        let (keys, values) = from.into_parts();
        let values = V::Context::upcast_slices(values);
        unsafe { DenseSlices::new_unchecked(keys, values) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
        unsafe { slices.deref(self) }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        slices.into_slice_ptrs(self)
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        slices.len(self)
    }

    type SlicesMut<'a> = DenseSlicesMut<'a, 'data, K, V>;

    #[inline]
    fn upcast_mut_slices<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        let (keys, values) = from.into_parts();
        let values = V::Context::upcast_mut_slices(values);
        unsafe { DenseSlicesMut::new_unchecked(keys, values) }
    }

    #[inline]
    unsafe fn mut_slice_ptrs_to_mut_slices<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a> {
        unsafe { slices.deref_mut(self) }
    }

    #[inline]
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a> {
        slices.into_mut_slice_ptrs(self)
    }

    #[inline]
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len(self)
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        slices.into_slices(self)
    }
}

impl<'a, K, V> SoaAsRefs<'a> for DenseItem<K, V>
where
    K: 'a,
    V: SoaAsRefs<'a> + ?Sized,
{
    #[inline]
    fn as_refs(&'a self, context: &'a Self::Context) -> Refs<'a, 'a, Self> {
        Self::as_refs(self, context)
    }
}

impl<'a, K, V> SoaAsMutRefs<'a> for DenseItem<K, V>
where
    K: 'a,
    V: SoaAsMutRefs<'a> + ?Sized,
{
    #[inline]
    fn as_mut_refs(&'a mut self, context: &'a Self::Context) -> RefsMut<'a, 'a, Self> {
        Self::as_mut_refs(self, context)
    }
}
