use core::{
    alloc::{Layout, LayoutError},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::soa::traits::{
    FieldDescriptor, FieldDescriptors, MutPtrs, NonNullPtrs, Ptrs, Refs, RefsMut, SliceMutPtrs,
    SlicePtrs, Slices, SlicesMut, Soa, SoaRead, SoaToOwned, SoaTrustedFields, SoaWrite,
};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct KeyValuePair<K, V>
where
    V: ?Sized,
{
    pub key: K,
    pub value: V,
}

impl<K, V> KeyValuePair<K, V> {
    #[inline]
    pub const fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> From<(K, V)> for KeyValuePair<K, V> {
    #[inline]
    fn from(value: (K, V)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValuePair<K, V>> for (K, V) {
    #[inline]
    fn from(value: KeyValuePair<K, V>) -> Self {
        let KeyValuePair { key, value } = value;
        (key, value)
    }
}

#[allow(unsafe_code)]
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

    type MutPtrs<'context> = KeyValueMutPtrs<'context, K, V>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs<'_> {
        KeyValueMutPtrs::dangling(context)
    }

    #[inline]
    unsafe fn ptrs_from_buffer<'context>(
        context: &'context Self::Context,
        buffer: *mut u8,
        capacity: usize,
    ) -> Self::MutPtrs<'context> {
        let keys = unsafe { Layout::array::<K>(capacity).unwrap_unchecked() };
        let values = unsafe { V::buffer_layout(context, capacity).unwrap_unchecked() };
        let (_, offset) = unsafe { keys.extend(values).unwrap_unchecked() };

        let key = buffer.cast();
        let buffer = unsafe { buffer.add(offset) };
        let value = MutPtrs::new(unsafe { V::ptrs_from_buffer(context, buffer, capacity) });
        KeyValueMutPtrs { key, value }
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
        let value = Refs::new(V::upcast_refs(value.into_inner()));
        KeyValueRefs { key, value }
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
        let value = RefsMut::new(V::upcast_refs_mut(value.into_inner()));
        KeyValueRefsMut { key, value }
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
    fn value_as_refs<'context, 'a>(
        context: &'context Self::Context,
        value: &'a Self,
    ) -> Self::Refs<'context, 'a>
    where
        Self: 'a,
        'a: 'context,
    {
        let KeyValuePair { key, value } = value;

        let value = Refs::new(V::value_as_refs(context, value));
        KeyValueRefs { key, value }
    }

    #[inline]
    fn mut_value_as_refs<'context, 'a>(
        context: &'context Self::Context,
        value: &'a mut Self,
    ) -> Self::RefsMut<'context, 'a>
    where
        Self: 'a,
        'a: 'context,
    {
        let KeyValuePair { key, value } = value;

        let value = RefsMut::new(V::mut_value_as_refs(context, value));
        KeyValueRefsMut { key, value }
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
        let KeyValueSlices { keys, values } = from;
        let values = Slices::new(V::upcast_slices(values.into_inner()));
        KeyValueSlices { keys, values }
    }

    type SlicesMut<'context, 'a>
        = KeyValueSlicesMut<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_slices_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::SlicesMut<'long, 'a_long>,
    ) -> Self::SlicesMut<'short, 'a_short> {
        let KeyValueSlicesMut { keys, values } = from;
        let values = SlicesMut::new(V::upcast_slices_mut(values.into_inner()));
        KeyValueSlicesMut { keys, values }
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

#[allow(unsafe_code)]
unsafe impl<K, V> SoaRead for KeyValuePair<K, V>
where
    V: SoaRead,
{
    #[inline]
    unsafe fn read(context: &Self::Context, src: Self::Ptrs<'_>) -> Self {
        let KeyValuePtrs { key, value } = src;
        Self {
            key: unsafe { ptr::read(key) },
            value: unsafe { V::read(context, value.into_inner()) },
        }
    }
}

#[allow(unsafe_code)]
unsafe impl<K, V> SoaWrite for KeyValuePair<K, V>
where
    V: SoaWrite,
{
    #[inline]
    unsafe fn write(context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self) {
        let KeyValueMutPtrs {
            key: key_ptr,
            value: value_ptr,
        } = dst;
        let Self { key, value } = value;

        unsafe {
            ptr::write(key_ptr, key);
            V::write(context, value_ptr.into_inner(), value);
        }
    }
}

#[allow(unsafe_code)]
unsafe impl<K, V> SoaTrustedFields for KeyValuePair<K, V> where V: SoaTrustedFields {}

pub struct KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
{
    key: FieldDescriptor,
    values: FieldDescriptors<'context, V>,
    phantom: PhantomData<fn() -> K>,
}

impl<'context, K, V> KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(context: &'context V::Context) -> Self {
        Self {
            key: FieldDescriptor::of::<K>(),
            values: FieldDescriptors::new(V::field_descriptors(context)),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, FieldDescriptors<'context, V>) {
        let Self { key, values, .. } = self;
        (key, values)
    }
}

impl<'context, K, V> Debug for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
    FieldDescriptors<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, values, .. } = self;
        f.debug_struct("KeyValueFieldLayouts")
            .field("key", key)
            .field("values", values)
            .finish()
    }
}

impl<'context, K, V> Clone for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
    FieldDescriptors<'context, V>: Clone,
{
    fn clone(&self) -> Self {
        let Self {
            key,
            ref values,
            phantom,
        } = *self;
        Self {
            key,
            values: values.clone(),
            phantom,
        }
    }
}

impl<'context, K, V> Copy for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
    FieldDescriptors<'context, V>: Copy,
{
}

impl<'context, K, V> IntoIterator for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa + ?Sized,
{
    type Item = FieldDescriptor;

    type IntoIter = iter::Chain<
        iter::Once<FieldDescriptor>,
        iter::Map<
            <V::FieldDescriptors<'context> as IntoIterator>::IntoIter,
            fn(<V::FieldDescriptors<'context> as IntoIterator>::Item) -> FieldDescriptor,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let Self { key, values, .. } = self;

        let f: fn(<V::FieldDescriptors<'context> as IntoIterator>::Item) -> _ =
            |desc| *desc.as_ref();
        let value = values.into_iter().map(f);
        iter::once(key).chain(value)
    }
}

pub struct KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    pub key: *const K,
    pub value: Ptrs<'context, V>,
}

impl<'context, K, V> KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn cast_mut(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.cast_mut();
        let value = MutPtrs::new(V::ptrs_cast_mut(context, value.into_inner()));
        KeyValueMutPtrs { key, value }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn add(self, context: &'context V::Context, offset: usize) -> Self {
        let Self { key, value } = self;

        let key = unsafe { key.add(offset) };
        let value = Ptrs::new(unsafe { V::ptrs_add(context, value.into_inner(), offset) });
        Self { key, value }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn offset_from(self, context: &V::Context, origin: KeyValuePtrs<'_, K, V>) -> isize {
        let Self { key, value } = self;
        let KeyValuePtrs {
            key: origin_key,
            value: origin_value,
        } = origin;

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset =
            unsafe { V::ptrs_offset_from(context, value.into_inner(), origin_value.into_inner()) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn deref<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueRefs<'context, 'a, K, V> {
        let Self { key, value } = self;

        let key = unsafe { &*key };
        let value = Refs::new(unsafe { V::ptrs_to_refs(context, value.into_inner()) });
        KeyValueRefs { key, value }
    }
}

impl<'context, K, V> From<(*const K, Ptrs<'context, V>)> for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (*const K, Ptrs<'context, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValuePtrs<'context, K, V>> for (*const K, Ptrs<'context, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValuePtrs<'context, K, V>) -> Self {
        let KeyValuePtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    Ptrs<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValuePtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    Ptrs<'context, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    Ptrs<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    Ptrs<'context, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;
        match key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        value.partial_cmp(&other.value)
    }
}

impl<'context, K, V> Ord for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    Ptrs<'context, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;
        match key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        value.cmp(&other.value)
    }
}

impl<'context, K, V> Hash for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    Ptrs<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValuePtrs<'_, K, V>
where
    V: Soa + ?Sized,
{
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<'context, K, V> Copy for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    Ptrs<'context, V>: Copy,
{
}

pub struct KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    pub key: *mut K,
    pub value: MutPtrs<'context, V>,
}

impl<'context, K, V> KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn dangling(context: &'context V::Context) -> Self {
        let key = ptr::dangling_mut();
        let value = MutPtrs::new(V::ptrs_dangling(context));
        Self { key, value }
    }

    #[inline]
    pub fn cast_const(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.cast_const();
        let value = Ptrs::new(V::ptrs_cast_const(context, value.into_inner()));
        KeyValuePtrs { key, value }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn add(self, context: &'context V::Context, offset: usize) -> Self {
        let Self { key, value } = self;

        let key = unsafe { key.add(offset) };
        let value = MutPtrs::new(unsafe { V::ptrs_add_mut(context, value.into_inner(), offset) });
        Self { key, value }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn offset_from(self, context: &V::Context, origin: KeyValuePtrs<'_, K, V>) -> isize {
        let Self { key, value } = self;
        let KeyValuePtrs {
            key: origin_key,
            value: origin_value,
        } = origin;

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset = unsafe {
            V::ptrs_offset_from_mut(context, value.into_inner(), origin_value.into_inner())
        };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn swap(self, context: &V::Context, with: KeyValueMutPtrs<'_, K, V>) {
        let Self {
            key: this_key,
            value: this_value,
        } = self;
        let KeyValueMutPtrs {
            key: with_key,
            value: with_value,
        } = with;

        unsafe {
            ptr::swap(this_key, with_key);
            V::ptrs_swap(context, this_value.into_inner(), with_value.into_inner());
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn copy_from(self, context: &V::Context, from: KeyValuePtrs<'_, K, V>, len: usize) {
        let Self {
            key: dst_key,
            value: dst_value,
        } = self;
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = from;

        unsafe {
            ptr::copy(src_key, dst_key, len);
            V::ptrs_copy(context, src_value.into_inner(), dst_value.into_inner(), len);
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn copy_from_rev(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V>,
        len: usize,
    ) {
        let Self {
            key: dst_key,
            value: dst_value,
        } = self;
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = from;

        unsafe {
            V::ptrs_copy_rev(context, src_value.into_inner(), dst_value.into_inner(), len);
            ptr::copy(src_key, dst_key, len);
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn copy_from_nonoverlapping(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V>,
        len: usize,
    ) {
        let Self {
            key: dst_key,
            value: dst_value,
        } = self;
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = from;

        let src_value = src_value.into_inner();
        let dst_value = dst_value.into_inner();
        unsafe {
            ptr::copy_nonoverlapping(src_key, dst_key, len);
            V::ptrs_copy_nonoverlapping(context, src_value, dst_value, len);
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn drop_in_place(self, context: &V::Context) {
        let Self { key, value } = self;

        unsafe {
            ptr::drop_in_place(key);
            V::ptrs_drop_in_place(context, value.into_inner());
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn deref<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueRefs<'context, 'a, K, V> {
        let Self { key, value } = self;

        let key = unsafe { &*key };
        let value = V::ptrs_cast_const(context, value.into_inner());
        let value = Refs::new(unsafe { V::ptrs_to_refs(context, value) });
        KeyValueRefs { key, value }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn deref_mut<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueRefsMut<'context, 'a, K, V> {
        let Self { key, value } = self;

        let key = unsafe { &mut *key };
        let value = RefsMut::new(unsafe { V::ptrs_to_refs_mut(context, value.into_inner()) });
        KeyValueRefsMut { key, value }
    }
}

impl<'context, K, V> From<(*mut K, MutPtrs<'context, V>)> for KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (*mut K, MutPtrs<'context, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValueMutPtrs<'context, K, V>> for (*mut K, MutPtrs<'context, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueMutPtrs<'context, K, V>) -> Self {
        let KeyValueMutPtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    MutPtrs<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueMutPtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    MutPtrs<'context, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    MutPtrs<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    MutPtrs<'context, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;
        match key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        value.partial_cmp(&other.value)
    }
}

impl<'context, K, V> Ord for KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    MutPtrs<'context, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;
        match key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        value.cmp(&other.value)
    }
}

impl<'context, K, V> Hash for KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    MutPtrs<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValueMutPtrs<'_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<'context, K, V> Copy for KeyValueMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    MutPtrs<'context, V>: Copy,
{
}

pub struct KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    pub key: NonNull<K>,
    pub value: NonNullPtrs<'context, V>,
}

impl<'context, K, V> KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn new_unchecked(
        context: &'context V::Context,
        ptrs: KeyValueMutPtrs<'context, K, V>,
    ) -> Self {
        let KeyValueMutPtrs { key, value } = ptrs;

        let key = unsafe { NonNull::new_unchecked(key) };
        let value = NonNullPtrs::new(unsafe { V::ptrs_to_nonnull(context, value.into_inner()) });
        Self { key, value }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.as_ptr().cast_const();
        let value = V::nonnull_to_ptrs(context, value.into_inner());
        let value = Ptrs::new(V::ptrs_cast_const(context, value));
        KeyValuePtrs { key, value }
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.as_ptr();
        let value = MutPtrs::new(V::nonnull_to_ptrs(context, value.into_inner()));
        KeyValueMutPtrs { key, value }
    }
}

impl<'context, K, V> From<(NonNull<K>, NonNullPtrs<'context, V>)>
    for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (NonNull<K>, NonNullPtrs<'context, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValueNonNullPtrs<'context, K, V>>
    for (NonNull<K>, NonNullPtrs<'context, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueNonNullPtrs<'context, K, V>) -> Self {
        let KeyValueNonNullPtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    NonNullPtrs<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueNonNullPtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    NonNullPtrs<'context, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    NonNullPtrs<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    NonNullPtrs<'context, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;
        match key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        value.partial_cmp(&other.value)
    }
}

impl<'context, K, V> Ord for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    NonNullPtrs<'context, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;
        match key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        value.cmp(&other.value)
    }
}

impl<'context, K, V> Hash for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    NonNullPtrs<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValueNonNullPtrs<'_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<'context, K, V> Copy for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    NonNullPtrs<'context, V>: Copy,
{
}

pub struct KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    pub key: &'a K,
    pub value: Refs<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_ref(key);
        let value = Ptrs::new(V::refs_as_ptrs(context, value.into_inner()));
        KeyValuePtrs { key, value }
    }
}

impl<'context, 'a, K, V> From<(&'a K, Refs<'context, 'a, V>)> for KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (&'a K, Refs<'context, 'a, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, 'a, K, V> From<KeyValueRefs<'context, 'a, K, V>> for (&'a K, Refs<'context, 'a, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueRefs<'context, 'a, K, V>) -> Self {
        let KeyValueRefs { key, value } = value;
        (key, value)
    }
}

impl<'context, 'a, K, V> Debug for KeyValueRefs<'context, 'a, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueRefs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, 'a, K, V> PartialEq for KeyValueRefs<'context, 'a, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, 'a, K, V> Eq for KeyValueRefs<'context, 'a, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueRefs<'context, 'a, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;
        match key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        value.partial_cmp(&other.value)
    }
}

impl<'context, 'a, K, V> Ord for KeyValueRefs<'context, 'a, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;
        match key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        value.cmp(&other.value)
    }
}

impl<'context, 'a, K, V> Hash for KeyValueRefs<'context, 'a, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<'context, 'a, K, V> Clone for KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<'context, 'a, K, V> Copy for KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Copy,
{
}

impl<'context, 'a, K, V> SoaToOwned<'context, 'a> for KeyValueRefs<'context, 'a, K, V>
where
    K: Clone,
    V: SoaWrite,
    V::Refs<'context, 'a>: SoaToOwned<'context, 'a, Owned = V>,
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

pub struct KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    pub key: &'a mut K,
    pub value: RefsMut<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_mut(key);
        let value = MutPtrs::new(V::refs_mut_as_ptrs(context, value.into_inner()));
        KeyValueMutPtrs { key, value }
    }

    #[inline]
    pub fn into_refs(self, context: &'context V::Context) -> KeyValueRefs<'context, 'a, K, V> {
        let Self { key, value } = self;

        let key = &*key;
        let value = Refs::new(V::refs_mut_as_refs(context, value.into_inner()));
        KeyValueRefs { key, value }
    }
}

impl<'context, 'a, K, V> From<(&'a mut K, RefsMut<'context, 'a, V>)>
    for KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (&'a mut K, RefsMut<'context, 'a, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, 'a, K, V> From<KeyValueRefsMut<'context, 'a, K, V>>
    for (&'a mut K, RefsMut<'context, 'a, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueRefsMut<'context, 'a, K, V>) -> Self {
        let KeyValueRefsMut { key, value } = value;
        (key, value)
    }
}

impl<'context, 'a, K, V> Debug for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueRefsMut")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, 'a, K, V> PartialEq for KeyValueRefsMut<'context, 'a, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, 'a, K, V> Eq for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueRefsMut<'context, 'a, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;
        match key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        value.partial_cmp(&other.value)
    }
}

impl<'context, 'a, K, V> Ord for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;
        match key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        value.cmp(&other.value)
    }
}

impl<'context, 'a, K, V> Hash for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

pub struct KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    keys: *const [K],
    values: SlicePtrs<'context, V>,
}

impl<'context, K, V> KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    #[track_caller]
    #[allow(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        context: &'context V::Context,
        keys: *const [K],
        values: V::SlicePtrs<'context>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = V::slice_ptrs_len(context, &values);
        assert_eq!(keys_len, values_len);

        #[allow(unsafe_code)]
        unsafe {
            Self::new_unchecked(keys, values)
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn new_unchecked(keys: *const [K], values: V::SlicePtrs<'context>) -> Self {
        let values = SlicePtrs::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn from_raw_parts(
        context: &'context V::Context,
        ptrs: KeyValuePtrs<'context, K, V>,
        len: usize,
    ) -> Self {
        let KeyValuePtrs { key, value } = ptrs;

        let keys = ptr::slice_from_raw_parts(key, len);
        let values = SlicePtrs::new(V::slices_from_raw_parts(context, value.into_inner(), len));
        Self { keys, values }
    }

    #[inline]
    pub fn into_parts(self) -> (*const [K], SlicePtrs<'context, V>) {
        let Self { keys, values } = self;
        (keys, values)
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slice_ptrs_len(context, values.as_inner())
    }

    #[inline]
    pub fn cast_mut(self, context: &'context V::Context) -> KeyValueSliceMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = keys.cast_mut();
        let values = SliceMutPtrs::new(V::slice_ptrs_cast_mut(context, values.into_inner()));
        KeyValueSliceMutPtrs { keys, values }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast(); // should be `keys.as_ptr()` but it's unstable
        let value = Ptrs::new(V::slice_ptrs_as_ptrs(context, values.into_inner()));
        KeyValuePtrs { key, value }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn deref<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlices<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { &*keys };
        let values = Slices::new(unsafe { V::slice_ptrs_to_slices(context, values.into_inner()) });
        KeyValueSlices { keys, values }
    }
}

impl<'context, K, V> From<KeyValueSlicePtrs<'context, K, V>>
    for (*const [K], SlicePtrs<'context, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSlicePtrs<'context, K, V>) -> Self {
        value.into_parts()
    }
}

impl<'context, K, V> Debug for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("KeyValueSlicePtrs")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: PartialEq,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, K, V> Eq for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: PartialOrd,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { keys, values } = self;
        match keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        values.partial_cmp(&other.values)
    }
}

impl<'context, K, V> Ord for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Ord,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { keys, values } = self;
        match keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        values.cmp(&other.values)
    }
}

impl<'context, K, V> Hash for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSlicePtrs<'_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, ref values } = *self;
        let values = values.clone();
        Self { keys, values }
    }
}

impl<'context, K, V> Copy for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Copy,
{
}

pub struct KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    keys: *mut [K],
    values: SliceMutPtrs<'context, V>,
}

impl<'context, K, V> KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    #[track_caller]
    #[allow(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        context: &'context V::Context,
        keys: *mut [K],
        values: V::SliceMutPtrs<'context>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = V::slice_mut_ptrs_len(context, &values);
        assert_eq!(keys_len, values_len);

        #[allow(unsafe_code)]
        unsafe {
            Self::new_unchecked(keys, values)
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn new_unchecked(keys: *mut [K], values: V::SliceMutPtrs<'context>) -> Self {
        let values = SliceMutPtrs::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn from_raw_parts(
        context: &'context V::Context,
        ptrs: KeyValueMutPtrs<'context, K, V>,
        len: usize,
    ) -> Self {
        let KeyValueMutPtrs { key, value } = ptrs;

        let keys = ptr::slice_from_raw_parts_mut(key, len);
        let values = V::slices_from_raw_parts_mut(context, value.into_inner(), len);
        let values = SliceMutPtrs::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn into_parts(self) -> (*mut [K], SliceMutPtrs<'context, V>) {
        let Self { keys, values } = self;
        (keys, values)
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slice_mut_ptrs_len(context, values.as_inner())
    }

    #[inline]
    pub fn cast_const(self, context: &'context V::Context) -> KeyValueSlicePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = keys.cast_const();
        let values = SlicePtrs::new(V::slice_ptrs_cast_const(context, values.into_inner()));
        KeyValueSlicePtrs { keys, values }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast_const().cast(); // should be `keys.as_ptr()` but it's unstable
        let values = V::slice_ptrs_cast_const(context, values.into_inner());
        let value = Ptrs::new(V::slice_ptrs_as_ptrs(context, values));
        KeyValuePtrs { key, value }
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast(); // should be `keys.as_ptr()` but it's unstable
        let value = MutPtrs::new(V::slice_mut_ptrs_as_ptrs(context, values.into_inner()));
        KeyValueMutPtrs { key, value }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn deref<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlices<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { &*keys };
        let values = V::slice_ptrs_cast_const(context, values.into_inner());
        let values = Slices::new(unsafe { V::slice_ptrs_to_slices(context, values) });
        KeyValueSlices { keys, values }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn deref_mut<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlicesMut<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { &mut *keys };
        let values = unsafe { V::slice_mut_ptrs_to_slices(context, values.into_inner()) };
        let values = SlicesMut::new(values);
        KeyValueSlicesMut { keys, values }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn drop_in_place(self, context: &V::Context) {
        let Self { keys, values } = self;

        unsafe {
            ptr::drop_in_place(keys);
            V::slices_drop_in_place(context, values.into_inner());
        }
    }
}

impl<'context, K, V> From<KeyValueSliceMutPtrs<'context, K, V>>
    for (*mut [K], SliceMutPtrs<'context, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSliceMutPtrs<'context, K, V>) -> Self {
        value.into_parts()
    }
}

impl<'context, K, V> Debug for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SliceMutPtrs<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("KeyValueSliceMutPtrs")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SliceMutPtrs<'context, V>: PartialEq,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, K, V> Eq for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SliceMutPtrs<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SliceMutPtrs<'context, V>: PartialOrd,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { keys, values } = self;
        match keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        values.partial_cmp(&other.values)
    }
}

impl<'context, K, V> Ord for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SliceMutPtrs<'context, V>: Ord,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { keys, values } = self;
        match keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        values.cmp(&other.values)
    }
}

impl<'context, K, V> Hash for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SliceMutPtrs<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSliceMutPtrs<'_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, ref values } = *self;
        let values = values.clone();
        Self { keys, values }
    }
}

impl<'context, K, V> Copy for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SliceMutPtrs<'context, V>: Copy,
{
}

pub struct KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    keys: &'a [K],
    values: Slices<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn new(
        context: &'context V::Context,
        keys: &'a [K],
        values: V::Slices<'context, 'a>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = V::slices_len(context, &values);
        assert_eq!(keys_len, values_len);

        #[allow(unsafe_code)]
        unsafe {
            Self::new_unchecked(keys, values)
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn new_unchecked(keys: &'a [K], values: V::Slices<'context, 'a>) -> Self {
        let values = Slices::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slices_len(context, values.as_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a [K], Slices<'context, 'a, V>) {
        let Self { keys, values } = self;
        (keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlicePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_ref(keys);
        let values = SlicePtrs::new(V::slices_as_slice_ptrs(context, values.into_inner()));
        KeyValueSlicePtrs { keys, values }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_ptr();
        let value = Ptrs::new(V::slices_as_ptrs(context, values.into_inner()));
        KeyValuePtrs { key, value }
    }
}

impl<'context, 'a, K, V> From<KeyValueSlices<'context, 'a, K, V>>
    for (&'a [K], Slices<'context, 'a, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSlices<'context, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<'context, 'a, K, V> Debug for KeyValueSlices<'context, 'a, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("KeyValueSlices")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'context, 'a, K, V> Default for KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Default::default(),
        }
    }
}

impl<'context, 'a, K, V> PartialEq for KeyValueSlices<'context, 'a, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, 'a, K, V> Eq for KeyValueSlices<'context, 'a, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueSlices<'context, 'a, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { keys, values } = self;
        match keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        values.partial_cmp(&other.values)
    }
}

impl<'context, 'a, K, V> Ord for KeyValueSlices<'context, 'a, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { keys, values } = self;
        match keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        values.cmp(&other.values)
    }
}

impl<'context, 'a, K, V> Hash for KeyValueSlices<'context, 'a, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<'context, 'a, K, V> Clone for KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, ref values } = *self;
        let values = values.clone();
        Self { keys, values }
    }
}

impl<'context, 'a, K, V> Copy for KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Copy,
{
}

pub struct KeyValueSlicesMut<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    keys: &'a mut [K],
    values: SlicesMut<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueSlicesMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn new(
        context: &'context V::Context,
        keys: &'a mut [K],
        values: V::SlicesMut<'context, 'a>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = V::slices_mut_len(context, &values);
        assert_eq!(keys_len, values_len);

        #[allow(unsafe_code)]
        unsafe {
            Self::new_unchecked(keys, values)
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn new_unchecked(keys: &'a mut [K], values: V::SlicesMut<'context, 'a>) -> Self {
        let values = SlicesMut::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slices_mut_len(context, values.as_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a mut [K], SlicesMut<'context, 'a, V>) {
        let Self { keys, values } = self;
        (keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlicePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_ref(keys);
        let values = V::slices_mut_as_slices(context, values.into_inner());
        let values = SlicePtrs::new(V::slices_as_slice_ptrs(context, values));
        KeyValueSlicePtrs { keys, values }
    }

    #[inline]
    pub fn into_slice_mut_ptrs(
        self,
        context: &'context V::Context,
    ) -> KeyValueSliceMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_mut(keys);
        let values = SliceMutPtrs::new(V::slices_mut_as_slice_ptrs(context, values.into_inner()));
        KeyValueSliceMutPtrs { keys, values }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_ptr();
        let values = V::slices_mut_as_slices(context, values.into_inner());
        let value = Ptrs::new(V::slices_as_ptrs(context, values));
        KeyValuePtrs { key, value }
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_mut_ptr();
        let value = MutPtrs::new(V::slices_mut_as_ptrs(context, values.into_inner()));
        KeyValueMutPtrs { key, value }
    }

    #[inline]
    pub fn into_slices(self, context: &'context V::Context) -> KeyValueSlices<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = &*keys;
        let values = Slices::new(V::slices_mut_as_slices(context, values.into_inner()));
        KeyValueSlices { keys, values }
    }
}

impl<'context, 'a, K, V> From<KeyValueSlicesMut<'context, 'a, K, V>>
    for (&'a mut [K], SlicesMut<'context, 'a, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSlicesMut<'context, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<'context, 'a, K, V> Debug for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("KeyValueSlicesMut")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'context, 'a, K, V> Default for KeyValueSlicesMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Default::default(),
        }
    }
}

impl<'context, 'a, K, V> PartialEq for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, 'a, K, V> Eq for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { keys, values } = self;
        match keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        values.partial_cmp(&other.values)
    }
}

impl<'context, 'a, K, V> Ord for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { keys, values } = self;
        match keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        values.cmp(&other.values)
    }
}

impl<'context, 'a, K, V> Hash for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}
