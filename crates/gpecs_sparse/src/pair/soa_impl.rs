use core::alloc::{Layout, LayoutError};

use crate::{
    pair::{
        KeyValueFieldDescriptors, KeyValueMutPtrs, KeyValueNonNullPtrs, KeyValuePair,
        KeyValuePairContext, KeyValuePtrs, KeyValueRefs, KeyValueRefsMut, KeyValueSliceMutPtrs,
        KeyValueSlicePtrs, KeyValueSlices, KeyValueSlicesMut,
    },
    soa::traits::{
        MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaCloneToUninit,
        SoaRead, SoaTrustedFields, SoaWrite,
    },
};

unsafe impl<K, V> RawSoaContext for KeyValuePairContext<K, V>
where
    V: RawSoa + ?Sized,
{
    type FieldDescriptors<'a> = KeyValueFieldDescriptors<'a, K, V>;

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        from
    }

    #[inline]
    fn field_descriptors(&self) -> Self::FieldDescriptors<'_> {
        let context = self.as_inner();
        KeyValueFieldDescriptors::new(context)
    }

    #[inline]
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        let keys = Layout::array::<K>(capacity)?;
        let values = self.as_inner().buffer_layout(capacity)?;
        let (buffer_layout, _) = keys.extend(values)?;
        Ok(buffer_layout)
    }

    type Ptrs<'a> = KeyValuePtrs<'a, K, V>;

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
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_> {
        let context = self.as_inner();

        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = buffer.cast();
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { context.ptrs_from_buffer(buffer, capacity) };
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(self, offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(self, origin) }
    }

    type MutPtrs<'a> = KeyValueMutPtrs<'a, K, V>;

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
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let context = self.as_inner();

        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { context.buffer_layout(capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = buffer.cast();
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { context.ptrs_from_buffer_mut(buffer, capacity) };
        KeyValueMutPtrs::new(key, value)
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

    type NonNullPtrs<'a> = KeyValueNonNullPtrs<'a, K, V>;

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

    type SlicePtrs<'a> = KeyValueSlicePtrs<'a, K, V>;

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
        KeyValueSlicePtrs::from_raw_parts(context, ptrs, len)
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

    type SliceMutPtrs<'a> = KeyValueSliceMutPtrs<'a, K, V>;

    #[inline]
    fn upcast_slice_mut_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        from
    }

    #[inline]
    fn slice_mut_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a> {
        let context = self.as_inner();
        KeyValueSliceMutPtrs::from_raw_parts(context, ptrs, len)
    }

    #[inline]
    #[track_caller]
    fn slice_mut_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len(self)
    }

    #[inline]
    fn slice_mut_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
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

unsafe impl<K, V> RawSoa for KeyValuePair<K, V>
where
    V: RawSoa + ?Sized,
{
    type Context = KeyValuePairContext<K, V>;
    type Fields = (K, V::Fields);
}

unsafe impl<K, V> Soa for KeyValuePair<K, V>
where
    V: Soa + ?Sized,
{
    type Refs<'context, 'a>
        = KeyValueRefs<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_refs<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Refs<'long, 'a_long>,
    ) -> Self::Refs<'short, 'a_short> {
        let KeyValueRefs { key, value } = from;
        let value = V::upcast_refs(value.into_inner());
        KeyValueRefs::new(key, value)
    }

    type RefsMut<'context, 'a>
        = KeyValueRefsMut<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_refs_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::RefsMut<'long, 'a_long>,
    ) -> Self::RefsMut<'short, 'a_short> {
        let KeyValueRefsMut { key, value } = from;
        let value = V::upcast_refs_mut(value.into_inner());
        KeyValueRefsMut::new(key, value)
    }

    #[inline]
    unsafe fn ptrs_to_refs<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Ptrs<'context, Self>,
    ) -> Self::Refs<'context, 'a> {
        unsafe { ptrs.deref(context) }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        context: &'context Self::Context,
        ptrs: MutPtrs<'context, Self>,
    ) -> Self::RefsMut<'context, 'a> {
        unsafe { ptrs.deref_mut(context) }
    }

    #[inline]
    fn refs_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::Refs<'context, 'a>,
    ) -> Ptrs<'context, Self>
    where
        Self: 'a,
    {
        refs.into_ptrs(context)
    }

    #[inline]
    fn refs_mut_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> MutPtrs<'context, Self>
    where
        Self: 'a,
    {
        refs.into_ptrs(context)
    }

    #[inline]
    fn refs_mut_as_refs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        refs.into_refs(context)
    }

    #[inline]
    fn value_as_refs<'a>(context: &'a Self::Context, value: &'a Self) -> Self::Refs<'a, 'a>
    where
        Self: 'a,
    {
        value.as_refs(context)
    }

    #[inline]
    fn mut_value_as_refs<'a>(
        context: &'a Self::Context,
        value: &'a mut Self,
    ) -> Self::RefsMut<'a, 'a>
    where
        Self: 'a,
    {
        value.as_refs_mut(context)
    }

    type Slices<'context, 'a>
        = KeyValueSlices<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_slices<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Slices<'long, 'a_long>,
    ) -> Self::Slices<'short, 'a_short> {
        let (keys, values) = from.into_parts();
        let values = V::upcast_slices(values);
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }

    type SlicesMut<'context, 'a>
        = KeyValueSlicesMut<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_slices_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::SlicesMut<'long, 'a_long>,
    ) -> Self::SlicesMut<'short, 'a_short> {
        let (keys, values) = from.into_parts();
        let values = V::upcast_slices_mut(values);
        unsafe { KeyValueSlicesMut::new_unchecked(keys, values) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: SlicePtrs<'context, Self>,
    ) -> Self::Slices<'context, 'a> {
        unsafe { slices.deref(context) }
    }

    #[inline]
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: SliceMutPtrs<'context, Self>,
    ) -> Self::SlicesMut<'context, 'a> {
        unsafe { slices.deref_mut(context) }
    }

    #[inline]
    #[track_caller]
    fn slices_len<'a>(context: &Self::Context, slices: &Self::Slices<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        slices.len(context)
    }

    #[inline]
    #[track_caller]
    fn slices_mut_len<'a>(context: &Self::Context, slices: &Self::SlicesMut<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        slices.len(context)
    }

    #[inline]
    fn slices_as_slice_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> SlicePtrs<'context, Self>
    where
        Self: 'a,
    {
        slices.into_slice_ptrs(context)
    }

    #[inline]
    fn slices_mut_as_slice_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> SliceMutPtrs<'context, Self>
    where
        Self: 'a,
    {
        slices.into_slice_mut_ptrs(context)
    }

    #[inline]
    fn slices_mut_as_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a> {
        slices.into_slices(context)
    }

    #[inline]
    fn slices_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> Ptrs<'context, Self>
    where
        Self: 'a,
    {
        slices.into_ptrs(context)
    }

    #[inline]
    fn slices_mut_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> MutPtrs<'context, Self>
    where
        Self: 'a,
    {
        slices.into_mut_ptrs(context)
    }
}

unsafe impl<K, V> SoaRead for KeyValuePair<K, V>
where
    V: SoaRead,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Ptrs<'_, Self>) -> Self {
        unsafe { src.read(context) }
    }
}

unsafe impl<K, V> SoaWrite for KeyValuePair<K, V>
where
    V: SoaWrite,
{
    #[inline]
    unsafe fn write(context: &Self::Context, dst: MutPtrs<'_, Self>, value: Self) {
        unsafe { dst.write(context, value) }
    }
}

unsafe impl<K, V> SoaTrustedFields for KeyValuePair<K, V> where V: SoaTrustedFields {}

unsafe impl<K, V> SoaCloneToUninit for KeyValuePair<K, V>
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
