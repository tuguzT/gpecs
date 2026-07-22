use core::{
    alloc::{Layout, LayoutError},
    iter::{Chain, Once},
    ptr,
};

use gpecs_ptr::slice::{ConstSliceItemPtr, MutSliceItemPtr, SliceItemPtrs};
use gpecs_soa::{
    field::{FieldLayouts, IntoFieldLayouts},
    identity::Identity,
    traits::{
        AllocSoaContext, AllocSoaTrusted, CloneToUninitSoaContext, RawSoa, RawSoaContext,
        ReadSoaContext, SoaCloneToUninit, SoaContext, SoaRead, SoaWrite, WriteSoaContext,
    },
};

use crate::{
    KeyValueFieldLayouts, KeyValueMutPtrs, KeyValueMutRefs, KeyValueMutSlicePtrs,
    KeyValueMutSlices, KeyValueNonNullPtrs, KeyValuePair, KeyValuePtrs, KeyValueRefs,
    KeyValueSlicePtrs, KeyValueSlices,
};

unsafe impl<K, V, P> RawSoaContext<KeyValuePair<K, V, P>> for Identity<V::Context>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Ptrs<'a> = KeyValuePtrs<'a, K, V, P::Const>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
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
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        let context = self.as_inner();
        KeyValueMutPtrs::dangling(context)
    }

    #[inline]
    unsafe fn ptrs_add_mut<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        count: usize,
    ) -> Self::MutPtrs<'a> {
        unsafe { ptrs.add(self, count) }
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
    unsafe fn ptrs_swap_nonoverlapping(
        &self,
        x: Self::MutPtrs<'_>,
        y: Self::MutPtrs<'_>,
        count: usize,
    ) {
        unsafe { x.swap_nonoverlapping(self, y, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
        unsafe { dst.copy_from_forward(self, src, count) }
    }

    #[inline]
    unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize) {
        unsafe { dst.copy_from_backward(self, src, count) }
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
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        let context = self.as_inner();
        let (key, value) = ptrs.into_parts();
        unsafe { KeyValueNonNullPtrs::new_unchecked(context, key, value) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.into_mut_ptrs(self)
    }

    type SlicePtrs<'a> = KeyValueSlicePtrs<'a, K, V, P::Const>;

    #[inline]
    fn upcast_slice_ptrs<'short, 'long: 'short>(
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
    fn upcast_mut_slice_ptrs<'short, 'long: 'short>(
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
    unsafe fn slices_drop_in_place(&self, slices_to_drop: Self::SliceMutPtrs<'_>) {
        unsafe { slices_to_drop.drop_in_place(self) }
    }
}

unsafe impl<K, V, P> RawSoa for KeyValuePair<K, V, P>
where
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Context = Identity<V::Context>;
    type Fields = (K, V::Fields);
}

unsafe impl<K, V, P> CloneToUninitSoaContext<KeyValuePair<K, V, P>> for Identity<V::Context>
where
    K: Clone,
    V: SoaCloneToUninit + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    unsafe fn clone_to_uninit(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>) {
        unsafe { src.clone_to_uninit(self, dst) }
    }
}

unsafe impl<'a, K, V, P, R> ReadSoaContext<'a, KeyValuePair<K, R, P>, KeyValuePair<K, V, P>>
    for Identity<V::Context>
where
    V: SoaRead<'a, R> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    unsafe fn read(&'a self, src: Self::Ptrs<'a>) -> KeyValuePair<K, R, P> {
        unsafe { src.read(self) }
    }
}

unsafe impl<K, V, P, W> WriteSoaContext<KeyValuePair<K, W, P>, KeyValuePair<K, V, P>>
    for Identity<V::Context>
where
    V: SoaWrite<W>,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    unsafe fn write(&self, dst: Self::MutPtrs<'_>, value: KeyValuePair<K, W, P>) {
        unsafe { dst.write(self, value) }
    }
}

impl<'a, K, V, P, C> FieldLayouts<'a, KeyValuePair<K, V, P>> for Identity<C>
where
    V: RawSoa<Context = C> + ?Sized,
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

unsafe impl<K, V, P> AllocSoaContext<KeyValuePair<K, V, P>> for Identity<V::Context>
where
    V: RawSoa<Context: AllocSoaContext<V>> + ?Sized,
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
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let context = self.as_inner();

        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = unsafe {
            let slice = ptr::slice_from_raw_parts_mut(buffer.cast(), capacity);
            P::Mut::from_slice(slice, 0)
        };
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { context.ptrs_from_buffer_mut(buffer, capacity) };
        KeyValueMutPtrs::new(key, value)
    }
}

unsafe impl<K, V, P> AllocSoaTrusted for KeyValuePair<K, V, P>
where
    V: AllocSoaTrusted,
    P: SliceItemPtrs<Item = K>,
{
}

unsafe impl<'data, K, V, P> SoaContext<'data, KeyValuePair<K, V, P>> for Identity<V::Context>
where
    K: 'data,
    V: RawSoa<Context: SoaContext<'data, V>> + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    type Refs<'a> = KeyValueRefs<'a, 'data, K, V, P::Const>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        let (key, value) = from.into_parts();
        let value = V::Context::upcast_refs(value);
        KeyValueRefs::new(key, value)
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.as_ref_unchecked(self) }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        refs.into_ptrs(self)
    }

    type RefsMut<'a> = KeyValueMutRefs<'a, 'data, K, V, P::Mut>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        let (key, value) = from.into_parts();
        let value = V::Context::upcast_mut_refs(value);
        KeyValueMutRefs::new(key, value)
    }

    #[inline]
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.as_mut_unchecked(self) }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        refs.into_ptrs(self)
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs.into_refs(self)
    }

    type Slices<'a> = KeyValueSlices<'a, 'data, K, V, P::Const>;

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        let (keys, values) = from.into_parts();
        let values = V::Context::upcast_slices(values);
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
        unsafe { slices.as_ref_unchecked(self) }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        slices.into_slice_ptrs(self)
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    type SlicesMut<'a> = KeyValueMutSlices<'a, 'data, K, V, P::Mut>;

    #[inline]
    fn upcast_mut_slices<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        let (keys, values) = from.into_parts();
        let values = V::Context::upcast_mut_slices(values);
        unsafe { KeyValueMutSlices::new_unchecked(keys, values) }
    }

    #[inline]
    unsafe fn mut_slice_ptrs_to_mut_slices<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a> {
        unsafe { slices.as_mut_unchecked(self) }
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
        slices.len()
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        slices.into_slices(self)
    }
}
