use core::alloc::{Layout, LayoutError};

use crate::{
    pair::{
        KeyValueFieldDescriptors, KeyValueMutPtrs, KeyValueNonNullPtrs, KeyValuePair, KeyValuePtrs,
        KeyValueRefs, KeyValueRefsMut, KeyValueSliceMutPtrs, KeyValueSlicePtrs, KeyValueSlices,
        KeyValueSlicesMut,
    },
    soa::traits::{Soa, SoaRead, SoaToOwned, SoaTrustedFields, SoaWrite},
};

unsafe impl<K, V> Soa for KeyValuePair<K, V>
where
    V: Soa + ?Sized,
{
    type Context = V::Context;
    type Fields = (K, V::Fields);

    type FieldDescriptors<'context> = KeyValueFieldDescriptors<'context, K, V>;

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        from
    }

    #[inline]
    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_> {
        KeyValueFieldDescriptors::new(context)
    }

    #[inline]
    fn buffer_layout(context: &Self::Context, capacity: usize) -> Result<Layout, LayoutError> {
        let keys = Layout::array::<K>(capacity)?;
        let values = V::buffer_layout(context, capacity)?;
        let (buffer_layout, _) = keys.extend(values)?;
        Ok(buffer_layout)
    }

    type Ptrs<'context> = KeyValuePtrs<'context, K, V>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(context: &Self::Context) -> Self::Ptrs<'_> {
        KeyValuePtrs::dangling(context)
    }

    #[inline]
    unsafe fn ptrs_from_buffer(
        context: &Self::Context,
        buffer: *const u8,
        capacity: usize,
    ) -> Self::Ptrs<'_> {
        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { V::buffer_layout(context, capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = buffer.cast();
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { V::ptrs_from_buffer(context, buffer, capacity) };
        KeyValuePtrs::new(key, value)
    }

    type MutPtrs<'context> = KeyValueMutPtrs<'context, K, V>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(context: &Self::Context) -> Self::MutPtrs<'_> {
        KeyValueMutPtrs::dangling(context)
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(
        context: &Self::Context,
        buffer: *mut u8,
        capacity: usize,
    ) -> Self::MutPtrs<'_> {
        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { V::buffer_layout(context, capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = buffer.cast();
        let buffer = unsafe { buffer.add(offset) };
        let value = unsafe { V::ptrs_from_buffer_mut(context, buffer, capacity) };
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    fn ptrs_cast_const<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::Ptrs<'context> {
        ptrs.cast_const(context)
    }

    #[inline]
    fn ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::MutPtrs<'context> {
        ptrs.cast_mut(context)
    }

    #[inline]
    unsafe fn ptrs_add<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        offset: usize,
    ) -> Self::Ptrs<'context> {
        unsafe { ptrs.add(context, offset) }
    }

    #[inline]
    unsafe fn ptrs_add_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        offset: usize,
    ) -> Self::MutPtrs<'context> {
        unsafe { ptrs.add(context, offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(context, origin) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(context, origin) }
    }

    #[inline]
    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        unsafe { a.swap(context, b) }
    }

    #[inline]
    unsafe fn ptrs_copy(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { dst.copy_from(context, src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { dst.copy_from_rev(context, src, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { dst.copy_from_nonoverlapping(context, src, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs<'_>) {
        unsafe { ptrs.drop_in_place(context) }
    }

    type NonNullPtrs<'context> = KeyValueNonNullPtrs<'context, K, V>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::NonNullPtrs<'context> {
        unsafe { KeyValueNonNullPtrs::new_unchecked(context, ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs<'context>(
        context: &'context Self::Context,
        ptrs: Self::NonNullPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        ptrs.into_mut_ptrs(context)
    }

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
        ptrs: Self::Ptrs<'context>,
    ) -> Self::Refs<'context, 'a> {
        unsafe { ptrs.deref(context) }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::RefsMut<'context, 'a> {
        unsafe { ptrs.deref_mut(context) }
    }

    #[inline]
    fn refs_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::Refs<'context, 'a>,
    ) -> Self::Ptrs<'context>
    where
        Self: 'a,
    {
        refs.into_ptrs(context)
    }

    #[inline]
    fn refs_mut_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::MutPtrs<'context>
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

    type SlicePtrs<'context> = KeyValueSlicePtrs<'context, K, V>;

    #[inline]
    fn upcast_slice_ptrs<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        from
    }

    type SliceMutPtrs<'context> = KeyValueSliceMutPtrs<'context, K, V>;

    #[inline]
    fn upcast_slice_mut_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        from
    }

    #[inline]
    fn slices_from_raw_parts<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        len: usize,
    ) -> Self::SlicePtrs<'context> {
        KeyValueSlicePtrs::from_raw_parts(context, ptrs, len)
    }

    #[inline]
    fn slices_from_raw_parts_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        len: usize,
    ) -> Self::SliceMutPtrs<'context> {
        KeyValueSliceMutPtrs::from_raw_parts(context, ptrs, len)
    }

    #[inline]
    fn slice_ptrs_cast_const<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicePtrs<'context> {
        slices.cast_const(context)
    }

    #[inline]
    fn slice_ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::SliceMutPtrs<'context> {
        slices.cast_mut(context)
    }

    #[inline]
    #[track_caller]
    fn slice_ptrs_len(context: &Self::Context, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len(context)
    }

    #[inline]
    #[track_caller]
    fn slice_mut_ptrs_len(context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len(context)
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        slices.into_ptrs(context)
    }

    #[inline]
    fn slice_mut_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        slices.into_mut_ptrs(context)
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
        let values = V::upcast_slices(values.into_inner());
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
        let values = V::upcast_slices_mut(values.into_inner());
        unsafe { KeyValueSlicesMut::new_unchecked(keys, values) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Slices<'context, 'a> {
        unsafe { slices.deref(context) }
    }

    #[inline]
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
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
    ) -> Self::SlicePtrs<'context>
    where
        Self: 'a,
    {
        slices.into_slice_ptrs(context)
    }

    #[inline]
    fn slices_mut_as_slice_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::SliceMutPtrs<'context>
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
    ) -> Self::Ptrs<'context>
    where
        Self: 'a,
    {
        slices.into_ptrs(context)
    }

    #[inline]
    fn slices_mut_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::MutPtrs<'context>
    where
        Self: 'a,
    {
        slices.into_mut_ptrs(context)
    }

    #[inline]
    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs<'_>) {
        unsafe { slices.drop_in_place(context) }
    }
}

unsafe impl<K, V> SoaRead for KeyValuePair<K, V>
where
    V: SoaRead,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Self::Ptrs<'_>) -> Self {
        unsafe { src.read(context) }
    }
}

unsafe impl<K, V> SoaWrite for KeyValuePair<K, V>
where
    V: SoaWrite,
{
    #[inline]
    unsafe fn write(context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self) {
        unsafe { dst.write(context, value) }
    }
}

unsafe impl<K, V> SoaTrustedFields for KeyValuePair<K, V> where V: SoaTrustedFields {}

impl<'context, 'a, K, V> SoaToOwned<'context, 'a> for KeyValueRefs<'context, 'a, K, V>
where
    K: Clone,
    V: SoaWrite,
    for<'c, 'any> V::Refs<'c, 'any>: SoaToOwned<'c, 'any, Owned = V>,
{
    type Owned = KeyValuePair<K, V>;

    fn to_owned(&self, context: &<Self::Owned as Soa>::Context) -> Self::Owned {
        let Self { key, value } = self;

        let key = (*key).clone();
        let value = SoaToOwned::to_owned(value.as_inner(), context);
        KeyValuePair { key, value }
    }

    fn clone_into(&self, context: &<Self::Owned as Soa>::Context, target: &mut Self::Owned) {
        let Self { key, value } = self;
        let KeyValuePair {
            key: target_key,
            value: target_value,
        } = target;

        target_key.clone_from(key);
        SoaToOwned::clone_into(value.as_inner(), context, target_value);
    }
}
