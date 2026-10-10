use core::{
    alloc::{Layout, LayoutError},
    iter::{Chain, Once},
    ptr::{self, NonNull},
};

use gpecs_ptr::slice::{ConstSliceItemPtr, MutSliceItemPtr, NonNullSliceItemPtr, SliceItemPtrs};
use gpecs_soa::{
    field::{FieldLayouts, IntoFieldLayouts},
    identity::Identity,
    traits::{
        SoaAllocContext, SoaAllocTrusted, SoaCloneToUninit, SoaCloneToUninitContext, SoaContext,
        SoaRaw, SoaRawContext, SoaRead, SoaReadContext, SoaWrite, SoaWriteContext,
    },
};

use crate::{
    KeyValueFieldLayouts, KeyValueMutPtrs, KeyValueMutRefs, KeyValueMutSlicePtrs,
    KeyValueMutSlices, KeyValueNonNullPtrs, KeyValueNonNullSlicePtrs, KeyValuePair, KeyValuePtrs,
    KeyValueRefs, KeyValueSlicePtrs, KeyValueSlices,
};

unsafe impl<K, V, P> SoaRawContext<KeyValuePair<K, V, P>> for Identity<V::Context>
where
    V: SoaRaw + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Ptrs<'a> = KeyValuePtrs<'a, K, V, P::Const>;

    #[inline]
    fn ptrs_upcast<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        let context = self.as_inner();
        KeyValuePtrs::dangling(context)
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, count: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(self, count) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(self, origin) }
    }

    type MutPtrs<'a> = KeyValueMutPtrs<'a, K, V, P::Mut>;

    #[inline]
    fn mut_ptrs_upcast<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn mut_ptrs_dangling(&self) -> Self::MutPtrs<'_> {
        let context = self.as_inner();
        KeyValueMutPtrs::dangling(context)
    }

    #[inline]
    unsafe fn mut_ptrs_add<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        count: usize,
    ) -> Self::MutPtrs<'a> {
        unsafe { ptrs.add(self, count) }
    }

    #[inline]
    unsafe fn mut_ptrs_offset_from(
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
    unsafe fn ptrs_swap_nonoverlapping(
        &self,
        x: Self::MutPtrs<'_>,
        y: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { x.swap_nonoverlapping(self, y, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_nonoverlapping(self, src, count) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, to_drop: Self::MutPtrs<'_>) {
        unsafe { to_drop.drop_in_place(self) }
    }

    type NonNullPtrs<'a> = KeyValueNonNullPtrs<'a, K, V, P::NonNull>;

    #[inline]
    fn nonnull_ptrs_upcast<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    fn nonnull_ptrs_dangling(&self) -> Self::NonNullPtrs<'_> {
        let context = self.as_inner();
        KeyValueNonNullPtrs::dangling(context)
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_ptrs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::NonNullPtrs<'a> {
        let context = self.as_inner();
        unsafe { KeyValueNonNullPtrs::from_ptrs(context, ptrs) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_mut_ptrs<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
    ) -> Self::NonNullPtrs<'a> {
        let context = self.as_inner();
        unsafe { KeyValueNonNullPtrs::from_mut_ptrs(context, ptrs) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_add<'a>(
        &'a self,
        ptrs: Self::NonNullPtrs<'a>,
        count: usize,
    ) -> Self::NonNullPtrs<'a> {
        unsafe { ptrs.add(self, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_offset_from(
        &self,
        ptrs: Self::NonNullPtrs<'_>,
        origin: Self::NonNullPtrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(self, origin) }
    }

    #[inline]
    fn nonnull_ptrs_as_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::Ptrs<'a> {
        ptrs.into_ptrs(self)
    }

    #[inline]
    fn nonnull_ptrs_as_mut_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.into_mut_ptrs(self)
    }

    #[inline]
    unsafe fn nonnull_ptrs_swap_nonoverlapping(
        &self,
        x: Self::NonNullPtrs<'_>,
        y: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { x.swap_nonoverlapping(self, y, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_nonoverlapping(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_nonoverlapping(self, src, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_drop_in_place(&self, to_drop: Self::NonNullPtrs<'_>) {
        unsafe { to_drop.drop_in_place(self) }
    }

    type SlicePtrs<'a> = KeyValueSlicePtrs<'a, K, V, P::Const>;

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
        let context = self.as_inner();
        KeyValueSlicePtrs::from_ptrs(context, data, len)
    }

    #[inline]
    #[track_caller]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs(self)
    }

    type SliceMutPtrs<'a> = KeyValueMutSlicePtrs<'a, K, V, P::Mut>;

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
        let context = self.as_inner();
        KeyValueMutSlicePtrs::from_ptrs(context, data, len)
    }

    #[inline]
    #[track_caller]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs(self)
    }

    #[inline]
    fn mut_slice_ptrs_as_mut_ptrs<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::MutPtrs<'a> {
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
    unsafe fn slices_drop_in_place(&self, slices_to_drop: Self::SliceMutPtrs<'_>) {
        unsafe { slices_to_drop.drop_in_place(self) }
    }

    type SliceNonNullPtrs<'a> = KeyValueNonNullSlicePtrs<'a, K, V, P::NonNull>;

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
        let context = self.as_inner();
        KeyValueNonNullSlicePtrs::from_ptrs(context, data, len)
    }

    #[inline]
    fn nonnull_slice_ptrs_len(&self, slices: &Self::SliceNonNullPtrs<'_>) -> usize {
        slices.len()
    }
}

unsafe impl<K, V, P> SoaRaw for KeyValuePair<K, V, P>
where
    V: SoaRaw + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Context = Identity<V::Context>;
    type Fields = (K, V::Fields);
}

unsafe impl<K, V, P> SoaCloneToUninitContext<KeyValuePair<K, V, P>> for Identity<V::Context>
where
    K: Clone,
    V: SoaCloneToUninit + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    unsafe fn ptrs_clone_to_uninit(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>) {
        unsafe { src.clone_to_uninit(self, dst) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_clone_to_uninit(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
    ) {
        unsafe { src.clone_to_uninit(self, dst) }
    }
}

unsafe impl<'a, K, V, P, R> SoaReadContext<'a, KeyValuePair<K, V, P>, KeyValuePair<K, R, P>>
    for Identity<V::Context>
where
    V: SoaRead<'a, R> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    unsafe fn ptrs_read(&'a self, src: Self::Ptrs<'a>) -> KeyValuePair<K, R, P> {
        unsafe { src.read(self) }
    }

    #[inline]
    unsafe fn mut_ptrs_read(&'a self, src: Self::MutPtrs<'a>) -> KeyValuePair<K, R, P> {
        unsafe { src.read(self) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_read(&'a self, src: Self::NonNullPtrs<'a>) -> KeyValuePair<K, R, P> {
        unsafe { src.read(self) }
    }
}

unsafe impl<K, V, P, W> SoaWriteContext<KeyValuePair<K, V, P>, KeyValuePair<K, W, P>>
    for Identity<V::Context>
where
    V: SoaWrite<W>,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    unsafe fn ptrs_write(&self, dst: Self::MutPtrs<'_>, value: KeyValuePair<K, W, P>) {
        unsafe { dst.write(self, value) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_write(&self, dst: Self::NonNullPtrs<'_>, value: KeyValuePair<K, W, P>) {
        unsafe { dst.write(self, value) }
    }
}

impl<'a, K, V, P, C> FieldLayouts<'a, KeyValuePair<K, V, P>> for Identity<C>
where
    V: SoaRaw<Context = C> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    C: FieldLayouts<'a, V>,
{
    type Output = KeyValueFieldLayouts<C::Output>;
    type OutputIter = Chain<Once<Layout>, IntoFieldLayouts<C::OutputIter>>;
    type OutputItem = Layout;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let context = self.as_inner();
        KeyValueFieldLayouts::new::<K, V>(context)
    }
}

unsafe impl<K, V, P> SoaAllocContext<KeyValuePair<K, V, P>> for Identity<V::Context>
where
    V: SoaRaw<Context: SoaAllocContext<V>> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        let keys = Layout::array::<K>(capacity)?;
        let values = self.as_inner().buffer_layout(capacity)?;
        let (buffer_layout, _) = keys.extend(values)?;
        Ok(buffer_layout)
    }

    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let context = self.as_inner();

        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = unsafe {
            let slice = ptr::slice_from_raw_parts(buffer.cast(), capacity);
            P::Const::from_slice(slice, 0)
        };
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { context.ptrs_from_buffer(buffer, capacity) };
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    unsafe fn mut_ptrs_from_buffer(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let context = self.as_inner();

        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = unsafe {
            let slice = ptr::slice_from_raw_parts_mut(buffer.cast(), capacity);
            P::Mut::from_slice(slice, 0)
        };
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { context.mut_ptrs_from_buffer(buffer, capacity) };
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    unsafe fn nonnull_ptrs_from_buffer(
        &self,
        buffer: NonNull<u8>,
        capacity: usize,
    ) -> Self::NonNullPtrs<'_> {
        let context = self.as_inner();

        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = unsafe {
            let slice = NonNull::slice_from_raw_parts(buffer.cast(), capacity);
            P::NonNull::from_slice(slice, 0)
        };
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { context.nonnull_ptrs_from_buffer(buffer, capacity) };
        KeyValueNonNullPtrs::new(key, value)
    }

    #[inline]
    unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
        unsafe { dst.copy_from_forward(self, src, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_forward(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_forward(self, src, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
        unsafe { dst.copy_from_backward(self, src, count) }
    }

    #[inline]
    unsafe fn nonnull_ptrs_copy_backward(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        unsafe { dst.copy_from_backward(self, src, count) }
    }
}

unsafe impl<K, V, P> SoaAllocTrusted for KeyValuePair<K, V, P>
where
    V: SoaAllocTrusted,
    P: SliceItemPtrs<Item = K>,
{
}

unsafe impl<'data, K, V, P> SoaContext<'data, KeyValuePair<K, V, P>> for Identity<V::Context>
where
    K: 'data,
    V: SoaRaw<Context: SoaContext<'data, V>> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Refs<'a> = KeyValueRefs<'a, 'data, K, V, P::Const>;

    #[inline]
    fn refs_upcast<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        let (key, value) = from.into_parts();
        let value = V::Context::refs_upcast(value);
        KeyValueRefs::new(key, value)
    }

    #[inline]
    unsafe fn refs_from_ptrs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.as_ref_unchecked(self) }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs(self)
    }

    type RefsMut<'a> = KeyValueMutRefs<'a, 'data, K, V, P::Mut>;

    #[inline]
    fn mut_refs_upcast<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        let (key, value) = from.into_parts();
        let value = V::Context::mut_refs_upcast(value);
        KeyValueMutRefs::new(key, value)
    }

    #[inline]
    unsafe fn mut_refs_from_mut_ptrs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.as_mut_unchecked(self) }
    }

    #[inline]
    fn mut_refs_as_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs(self)
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        refs.into_mut_ptrs(self)
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs.into_refs(self)
    }

    type Slices<'a> = KeyValueSlices<'a, 'data, K, V, P::Const>;

    #[inline]
    fn slices_upcast<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        let (keys, values) = from.into_parts();
        let values = V::Context::slices_upcast(values);
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }

    #[inline]
    unsafe fn slices_from_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicePtrs<'a>,
    ) -> Self::Slices<'a> {
        unsafe { slices.as_ref_unchecked(self) }
    }

    #[inline]
    unsafe fn slices_from_raw_parts<'a>(
        &'a self,
        data: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::Slices<'a> {
        let context = self.as_inner();
        unsafe { KeyValueSlices::from_raw_parts(context, data, len) }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        slices.into_slice_ptrs(self)
    }

    #[inline]
    fn slices_as_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs(self)
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    type SlicesMut<'a> = KeyValueMutSlices<'a, 'data, K, V, P::Mut>;

    #[inline]
    fn mut_slices_upcast<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        let (keys, values) = from.into_parts();
        let values = V::Context::mut_slices_upcast(values);
        unsafe { KeyValueMutSlices::new_unchecked(keys, values) }
    }

    #[inline]
    unsafe fn mut_slices_from_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a> {
        unsafe { slices.as_mut_unchecked(self) }
    }

    #[inline]
    unsafe fn mut_slices_from_raw_parts<'a>(
        &'a self,
        data: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SlicesMut<'a> {
        let context = self.as_inner();
        unsafe { KeyValueMutSlices::from_raw_parts(context, data, len) }
    }

    #[inline]
    fn mut_slices_as_slice_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::SlicePtrs<'a> {
        slices.into_slice_ptrs(self)
    }

    #[inline]
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a> {
        slices.into_mut_slice_ptrs(self)
    }

    #[inline]
    fn mut_slices_as_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Ptrs<'a> {
        slices.into_ptrs(self)
    }

    #[inline]
    fn mut_slices_as_mut_ptrs<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::MutPtrs<'a> {
        slices.into_mut_ptrs(self)
    }

    #[inline]
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        slices.into_slices(self)
    }
}
