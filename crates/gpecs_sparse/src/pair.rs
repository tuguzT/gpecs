use core::{
    alloc::{Layout, LayoutError},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::soa::traits::{FieldDescriptor, Soa, SoaToOwned, SoaTrustedFields};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct KeyValuePair<K, V> {
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
    V: Soa,
{
    type Context = V::Context;
    type Fields = (K, V::Fields);

    type FieldDescriptors<'context> = KeyValueFieldDescriptors<'context, K, V>;

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        let KeyValueFieldDescriptors {
            key,
            value,
            phantom,
        } = from;

        let value = V::upcast_field_descriptors(value);
        KeyValueFieldDescriptors {
            key,
            value,
            phantom,
        }
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
        let KeyValuePtrs { key, value } = from;
        let value = V::upcast_ptrs(value);
        KeyValuePtrs { key, value }
    }

    type MutPtrs<'context> = KeyValueMutPtrs<'context, K, V>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        let KeyValueMutPtrs { key, value } = from;
        let value = V::upcast_mut_ptrs(value);
        KeyValueMutPtrs { key, value }
    }

    #[inline]
    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs<'_> {
        KeyValueMutPtrs {
            key: ptr::dangling_mut(),
            value: V::ptrs_dangling(context),
        }
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
        let value = unsafe { V::ptrs_from_buffer(context, buffer.add(offset), capacity) };
        KeyValueMutPtrs { key, value }
    }

    #[inline]
    fn ptrs_cast_const<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValuePtrs {
            key: key.cast_const(),
            value: V::ptrs_cast_const(context, value),
        }
    }

    #[inline]
    fn ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: key.cast_mut(),
            value: V::ptrs_cast_mut(context, value),
        }
    }

    #[inline]
    unsafe fn ptrs_add<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        offset: usize,
    ) -> Self::Ptrs<'context> {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValuePtrs {
            key: unsafe { key.add(offset) },
            value: unsafe { V::ptrs_add(context, value, offset) },
        }
    }

    #[inline]
    unsafe fn ptrs_add_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        offset: usize,
    ) -> Self::MutPtrs<'context> {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: unsafe { key.add(offset) },
            value: unsafe { V::ptrs_add_mut(context, value, offset) },
        }
    }

    #[inline]
    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let KeyValuePtrs { key, value } = ptrs;
        let KeyValuePtrs {
            key: origin_key,
            value: origin_value,
        } = origin;

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset = unsafe { V::ptrs_offset_from(context, value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        let KeyValueMutPtrs { key, value } = ptrs;
        let KeyValuePtrs {
            key: origin_key,
            value: origin_value,
        } = origin;

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset = unsafe { V::ptrs_offset_from_mut(context, value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        let KeyValueMutPtrs {
            key: a_key,
            value: a_value,
        } = a;
        let KeyValueMutPtrs {
            key: b_key,
            value: b_value,
        } = b;

        unsafe {
            ptr::swap(a_key, b_key);
            V::ptrs_swap(context, a_value, b_value);
        }
    }

    #[inline]
    unsafe fn ptrs_copy(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = src;
        let KeyValueMutPtrs {
            key: dst_key,
            value: dst_value,
        } = dst;

        unsafe {
            ptr::copy(src_key, dst_key, len);
            V::ptrs_copy(context, src_value, dst_value, len);
        }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = src;
        let KeyValueMutPtrs {
            key: dst_key,
            value: dst_value,
        } = dst;

        unsafe {
            V::ptrs_copy_rev(context, src_value, dst_value, len);
            ptr::copy(src_key, dst_key, len);
        }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = src;
        let KeyValueMutPtrs {
            key: dst_key,
            value: dst_value,
        } = dst;

        unsafe {
            ptr::copy_nonoverlapping(src_key, dst_key, len);
            V::ptrs_copy_nonoverlapping(context, src_value, dst_value, len);
        }
    }

    #[inline]
    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs<'_>) -> Self {
        let KeyValuePtrs { key, value } = src;
        Self {
            key: unsafe { ptr::read(key) },
            value: unsafe { V::ptrs_read(context, value) },
        }
    }

    #[inline]
    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self) {
        let KeyValueMutPtrs {
            key: key_ptr,
            value: value_ptr,
        } = dst;
        let Self { key, value } = value;

        unsafe {
            ptr::write(key_ptr, key);
            V::ptrs_write(context, value_ptr, value);
        }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) {
        let KeyValueMutPtrs { key, value } = ptrs;

        unsafe {
            ptr::drop_in_place(key);
            V::ptrs_drop_in_place(context, value);
        }
    }

    type NonNullPtrs<'context> = KeyValueNonNullPtrs<'context, K, V>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        let KeyValueNonNullPtrs { key, value } = from;
        let value = V::upcast_nonnull_ptrs(value);
        KeyValueNonNullPtrs { key, value }
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::NonNullPtrs<'context> {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueNonNullPtrs {
            key: unsafe { NonNull::new_unchecked(key) },
            value: unsafe { V::ptrs_to_nonnull(context, value) },
        }
    }

    #[inline]
    fn nonnull_to_ptrs<'context>(
        context: &'context Self::Context,
        ptrs: Self::NonNullPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let KeyValueNonNullPtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: key.as_ptr(),
            value: V::nonnull_to_ptrs(context, value),
        }
    }

    type Refs<'context, 'a>
        = KeyValueRefs<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_refs<'a, 'short, 'long: 'short>(
        from: Self::Refs<'long, 'a>,
    ) -> Self::Refs<'short, 'a> {
        let KeyValueRefs { key, value } = from;
        let value = V::upcast_refs(value);
        KeyValueRefs { key, value }
    }

    type RefsMut<'context, 'a>
        = KeyValueRefsMut<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_refs_mut<'a, 'short, 'long: 'short>(
        from: Self::RefsMut<'long, 'a>,
    ) -> Self::RefsMut<'short, 'a> {
        let KeyValueRefsMut { key, value } = from;
        let value = V::upcast_refs_mut(value);
        KeyValueRefsMut { key, value }
    }

    #[inline]
    unsafe fn ptrs_to_refs<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::Refs<'context, 'a> {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueRefs {
            key: unsafe { &*key },
            value: unsafe { V::ptrs_to_refs(context, value) },
        }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::RefsMut<'context, 'a> {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueRefsMut {
            key: unsafe { &mut *key },
            value: unsafe { V::ptrs_to_refs_mut(context, value) },
        }
    }

    #[inline]
    fn refs_as_ptrs<'context>(
        context: &'context Self::Context,
        refs: Self::Refs<'context, '_>,
    ) -> Self::Ptrs<'context> {
        let KeyValueRefs { key, value } = refs;
        KeyValuePtrs {
            key: ptr::from_ref(key),
            value: V::refs_as_ptrs(context, value),
        }
    }

    #[inline]
    fn refs_mut_as_ptrs<'context>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        let KeyValueRefsMut { key, value } = refs;
        KeyValueMutPtrs {
            key: ptr::from_mut(key),
            value: V::refs_mut_as_ptrs(context, value),
        }
    }

    #[inline]
    fn refs_mut_as_refs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let KeyValueRefsMut { key, value } = refs;
        KeyValueRefs {
            key: &*key,
            value: V::refs_mut_as_refs(context, value),
        }
    }

    type SlicePtrs<'context> = KeyValueSlicePtrs<'context, K, V>;

    #[inline]
    fn upcast_slice_ptrs<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        let KeyValueSlicePtrs { keys, values } = from;
        let values = V::upcast_slice_ptrs(values);
        KeyValueSlicePtrs { keys, values }
    }

    type SliceMutPtrs<'context> = KeyValueSliceMutPtrs<'context, K, V>;

    #[inline]
    fn upcast_slice_mut_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        let KeyValueSliceMutPtrs { keys, values } = from;
        let values = V::upcast_slice_mut_ptrs(values);
        KeyValueSliceMutPtrs { keys, values }
    }

    #[inline]
    fn slices_from_raw_parts<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        len: usize,
    ) -> Self::SlicePtrs<'context> {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueSlicePtrs {
            keys: ptr::slice_from_raw_parts(key, len),
            values: V::slices_from_raw_parts(context, value, len),
        }
    }

    #[inline]
    fn slices_from_raw_parts_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        len: usize,
    ) -> Self::SliceMutPtrs<'context> {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueSliceMutPtrs {
            keys: ptr::slice_from_raw_parts_mut(key, len),
            values: V::slices_from_raw_parts_mut(context, value, len),
        }
    }

    #[inline]
    fn slice_ptrs_cast_const<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicePtrs<'context> {
        let KeyValueSliceMutPtrs { keys, values } = slices;
        KeyValueSlicePtrs {
            keys: keys.cast_const(),
            values: V::slice_ptrs_cast_const(context, values),
        }
    }

    #[inline]
    fn slice_ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::SliceMutPtrs<'context> {
        let KeyValueSlicePtrs { keys, values } = slices;
        KeyValueSliceMutPtrs {
            keys: keys.cast_mut(),
            values: V::slice_ptrs_cast_mut(context, values),
        }
    }

    #[inline]
    fn slice_ptrs_len(context: &Self::Context, slices: &Self::SlicePtrs<'_>) -> usize {
        let KeyValueSlicePtrs { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slice_ptrs_len(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    fn slice_mut_ptrs_len(context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
        let KeyValueSliceMutPtrs { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slice_mut_ptrs_len(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        let KeyValueSlicePtrs { keys, values } = slices;
        KeyValuePtrs {
            key: keys.cast(), // should be `keys.as_ptr()` but it's unstable
            value: V::slice_ptrs_as_ptrs(context, values),
        }
    }

    #[inline]
    fn slice_mut_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let KeyValueSliceMutPtrs { keys, values } = slices;
        KeyValueMutPtrs {
            key: keys.cast(), // should be `keys.as_mut_ptr()` but it's unstable
            value: V::slice_mut_ptrs_as_ptrs(context, values),
        }
    }

    type Slices<'context, 'a>
        = KeyValueSlices<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_slices<'a, 'short, 'long: 'short>(
        from: Self::Slices<'long, 'a>,
    ) -> Self::Slices<'short, 'a> {
        let KeyValueSlices { keys, values } = from;
        let values = V::upcast_slices(values);
        KeyValueSlices { keys, values }
    }

    type SlicesMut<'context, 'a>
        = KeyValueSlicesMut<'context, 'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn upcast_slices_mut<'a, 'short, 'long: 'short>(
        from: Self::SlicesMut<'long, 'a>,
    ) -> Self::SlicesMut<'short, 'a> {
        let KeyValueSlicesMut { keys, values } = from;
        let values = V::upcast_slices_mut(values);
        KeyValueSlicesMut { keys, values }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Slices<'context, 'a> {
        let len = Self::slice_ptrs_len(context, &slices);
        let KeyValueSlicePtrs { keys, values } = slices;

        KeyValueSlices {
            keys: unsafe { slice::from_raw_parts(keys.cast(), len) },
            values: unsafe { V::slice_ptrs_to_slices(context, values) },
        }
    }

    #[inline]
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicesMut<'context, 'a> {
        let len = Self::slice_mut_ptrs_len(context, &slices);
        let KeyValueSliceMutPtrs { keys, values } = slices;

        KeyValueSlicesMut {
            keys: unsafe { slice::from_raw_parts_mut(keys.cast(), len) },
            values: unsafe { V::slice_mut_ptrs_to_slices(context, values) },
        }
    }

    #[inline]
    #[track_caller]
    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_, '_>) -> usize {
        let KeyValueSlices { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slices_len(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    #[track_caller]
    fn slices_mut_len(context: &Self::Context, slices: &Self::SlicesMut<'_, '_>) -> usize {
        let KeyValueSlicesMut { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slices_mut_len(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    fn slices_as_slice_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::SlicePtrs<'context> {
        let KeyValueSlices { keys, values } = slices;
        KeyValueSlicePtrs {
            keys: ptr::from_ref(keys),
            values: V::slices_as_slice_ptrs(context, values),
        }
    }

    #[inline]
    fn slices_mut_as_slice_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::SliceMutPtrs<'context> {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueSliceMutPtrs {
            keys: ptr::from_mut(keys),
            values: V::slices_mut_as_slice_ptrs(context, values),
        }
    }

    #[inline]
    fn slices_mut_as_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a> {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueSlices {
            keys: &*keys,
            values: V::slices_mut_as_slices(context, values),
        }
    }

    #[inline]
    fn slices_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::Ptrs<'context> {
        let KeyValueSlices { keys, values } = slices;
        KeyValuePtrs {
            key: keys.as_ptr(),
            value: V::slices_as_ptrs(context, values),
        }
    }

    #[inline]
    fn slices_mut_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueMutPtrs {
            key: keys.as_mut_ptr(),
            value: V::slices_mut_as_ptrs(context, values),
        }
    }

    #[inline]
    unsafe fn slices_drop_in_place<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) {
        let KeyValueSliceMutPtrs { keys, values } = slices;

        unsafe {
            ptr::drop_in_place(keys);
            V::slices_drop_in_place(context, values);
        }
    }
}

#[allow(unsafe_code)]
unsafe impl<K, V> SoaTrustedFields for KeyValuePair<K, V> where V: SoaTrustedFields {}

pub struct KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa,
{
    key: FieldDescriptor,
    value: V::FieldDescriptors<'context>,
    phantom: PhantomData<fn() -> K>,
}

impl<'context, K, V> KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa,
{
    #[inline]
    pub fn new(context: &'context V::Context) -> Self {
        Self {
            key: FieldDescriptor::of::<K>(),
            phantom: PhantomData,
            value: V::field_descriptors(context),
        }
    }
}

impl<'context, K, V> Debug for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa,
    V::FieldDescriptors<'context>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value, .. } = self;
        f.debug_struct("KeyValueFieldLayouts")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, K, V> Clone for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa,
    V::FieldDescriptors<'context>: Clone,
{
    fn clone(&self) -> Self {
        let Self {
            key,
            ref value,
            phantom,
        } = *self;
        Self {
            key,
            value: value.clone(),
            phantom,
        }
    }
}

impl<'context, K, V> Copy for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa,
    V::FieldDescriptors<'context>: Copy,
{
}

impl<'context, K, V> IntoIterator for KeyValueFieldDescriptors<'context, K, V>
where
    V: Soa,
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
        let Self { key, value, .. } = self;

        let f: fn(<V::FieldDescriptors<'context> as IntoIterator>::Item) -> _ =
            |desc| *desc.as_ref();
        let value = value.into_iter().map(f);
        iter::once(key).chain(value)
    }
}

pub struct KeyValuePtrs<'context, K, V>
where
    V: Soa,
{
    pub key: *const K,
    pub value: V::Ptrs<'context>,
}

impl<'context, K, V> From<(*const K, V::Ptrs<'context>)> for KeyValuePtrs<'context, K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (*const K, V::Ptrs<'context>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValuePtrs<'context, K, V>> for (*const K, V::Ptrs<'context>)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValuePtrs<'context, K, V>) -> Self {
        let KeyValuePtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValuePtrs<'context, K, V>
where
    V: Soa,
    V::Ptrs<'context>: Debug,
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
    V: Soa,
    V::Ptrs<'context>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValuePtrs<'context, K, V>
where
    V: Soa,
    V::Ptrs<'context>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValuePtrs<'context, K, V>
where
    V: Soa,
    V::Ptrs<'context>: PartialOrd,
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
    V: Soa,
    V::Ptrs<'context>: Ord,
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
    V: Soa,
    V::Ptrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValuePtrs<'_, K, V>
where
    V: Soa,
{
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<'context, K, V> Copy for KeyValuePtrs<'context, K, V>
where
    V: Soa,
    V::Ptrs<'context>: Copy,
{
}

pub struct KeyValueMutPtrs<'context, K, V>
where
    V: Soa,
{
    pub key: *mut K,
    pub value: V::MutPtrs<'context>,
}

impl<'context, K, V> From<(*mut K, V::MutPtrs<'context>)> for KeyValueMutPtrs<'context, K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (*mut K, V::MutPtrs<'context>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValueMutPtrs<'context, K, V>> for (*mut K, V::MutPtrs<'context>)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueMutPtrs<'context, K, V>) -> Self {
        let KeyValueMutPtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValueMutPtrs<'context, K, V>
where
    V: Soa,
    V::MutPtrs<'context>: Debug,
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
    V: Soa,
    V::MutPtrs<'context>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValueMutPtrs<'context, K, V>
where
    V: Soa,
    V::MutPtrs<'context>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueMutPtrs<'context, K, V>
where
    V: Soa,
    V::MutPtrs<'context>: PartialOrd,
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
    V: Soa,
    V::MutPtrs<'context>: Ord,
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
    V: Soa,
    V::MutPtrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValueMutPtrs<'_, K, V>
where
    V: Soa,
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
    V: Soa,
    V::MutPtrs<'context>: Copy,
{
}

pub struct KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
{
    pub key: NonNull<K>,
    pub value: V::NonNullPtrs<'context>,
}

impl<'context, K, V> From<(NonNull<K>, V::NonNullPtrs<'context>)>
    for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (NonNull<K>, V::NonNullPtrs<'context>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValueNonNullPtrs<'context, K, V>>
    for (NonNull<K>, V::NonNullPtrs<'context>)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueNonNullPtrs<'context, K, V>) -> Self {
        let KeyValueNonNullPtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
    V::NonNullPtrs<'context>: Debug,
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
    V: Soa,
    V::NonNullPtrs<'context>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
    V::NonNullPtrs<'context>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
    V::NonNullPtrs<'context>: PartialOrd,
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
    V: Soa,
    V::NonNullPtrs<'context>: Ord,
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
    V: Soa,
    V::NonNullPtrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValueNonNullPtrs<'_, K, V>
where
    V: Soa,
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
    V: Soa,
    V::NonNullPtrs<'context>: Copy,
{
}

pub struct KeyValueRefs<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    pub key: &'a K,
    pub value: V::Refs<'context, 'a>,
}

impl<'context, 'a, K, V> From<(&'a K, V::Refs<'context, 'a>)> for KeyValueRefs<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: (&'a K, V::Refs<'context, 'a>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, 'a, K, V> From<KeyValueRefs<'context, 'a, K, V>> for (&'a K, V::Refs<'context, 'a>)
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: KeyValueRefs<'context, 'a, K, V>) -> Self {
        let KeyValueRefs { key, value } = value;
        (key, value)
    }
}

impl<'context, 'a, K, V> Debug for KeyValueRefs<'context, 'a, K, V>
where
    K: Debug + 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: Debug,
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
    K: PartialEq + 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, 'a, K, V> Eq for KeyValueRefs<'context, 'a, K, V>
where
    K: Eq + 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueRefs<'context, 'a, K, V>
where
    K: PartialOrd + 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: PartialOrd,
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
    K: Ord + 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: Ord,
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
    K: Hash + 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<'context, 'a, K, V> Clone for KeyValueRefs<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: Clone,
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
    K: 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: Copy,
{
}

impl<'context, 'a, K, V> SoaToOwned<'context, 'a> for KeyValueRefs<'context, 'a, K, V>
where
    K: Clone,
    V: Soa,
    V::Refs<'context, 'a>: SoaToOwned<'context, 'a, Owned = V>,
{
    type Owned = KeyValuePair<K, V>;

    fn to_owned(&self) -> Self::Owned {
        let Self { key, value } = self;
        KeyValuePair {
            key: (*key).clone(),
            value: value.to_owned(),
        }
    }

    fn clone_into(&self, target: &mut Self::Owned) {
        let Self { key, value } = self;
        let KeyValuePair {
            key: target_key,
            value: target_value,
        } = target;

        target_key.clone_from(key);
        value.clone_into(target_value);
    }
}

pub struct KeyValueRefsMut<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    pub key: &'a mut K,
    pub value: V::RefsMut<'context, 'a>,
}

impl<'context, 'a, K, V> From<(&'a mut K, V::RefsMut<'context, 'a>)>
    for KeyValueRefsMut<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: (&'a mut K, V::RefsMut<'context, 'a>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, 'a, K, V> From<KeyValueRefsMut<'context, 'a, K, V>>
    for (&'a mut K, V::RefsMut<'context, 'a>)
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: KeyValueRefsMut<'context, 'a, K, V>) -> Self {
        let KeyValueRefsMut { key, value } = value;
        (key, value)
    }
}

impl<'context, 'a, K, V> Debug for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Debug + 'a,
    V: Soa + 'a,
    V::RefsMut<'context, 'a>: Debug,
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
    K: PartialEq + 'a,
    V: Soa + 'a,
    V::RefsMut<'context, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, 'a, K, V> Eq for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Eq + 'a,
    V: Soa + 'a,
    V::RefsMut<'context, 'a>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueRefsMut<'context, 'a, K, V>
where
    K: PartialOrd + 'a,
    V: Soa + 'a,
    V::RefsMut<'context, 'a>: PartialOrd,
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
    K: Ord + 'a,
    V: Soa + 'a,
    V::RefsMut<'context, 'a>: Ord,
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
    K: Hash + 'a,
    V: Soa + 'a,
    V::RefsMut<'context, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

pub struct KeyValueSlicePtrs<'context, K, V>
where
    V: Soa,
{
    pub keys: *const [K],
    pub values: V::SlicePtrs<'context>,
}

impl<'context, K, V> From<(*const [K], V::SlicePtrs<'context>)>
    for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (*const [K], V::SlicePtrs<'context>)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<'context, K, V> From<KeyValueSlicePtrs<'context, K, V>>
    for (*const [K], V::SlicePtrs<'context>)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueSlicePtrs<'context, K, V>) -> Self {
        let KeyValueSlicePtrs { keys, values } = value;
        (keys, values)
    }
}

impl<'context, K, V> Debug for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa,
    V::SlicePtrs<'context>: Debug,
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
    V: Soa,
    V::SlicePtrs<'context>: PartialEq,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, K, V> Eq for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa,
    V::SlicePtrs<'context>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa,
    V::SlicePtrs<'context>: PartialOrd,
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
    V: Soa,
    V::SlicePtrs<'context>: Ord,
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
    V: Soa,
    V::SlicePtrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSlicePtrs<'_, K, V>
where
    V: Soa,
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
    V: Soa,
    V::SlicePtrs<'context>: Copy,
{
}

pub struct KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa,
{
    pub keys: *mut [K],
    pub values: V::SliceMutPtrs<'context>,
}

impl<'context, K, V> From<(*mut [K], V::SliceMutPtrs<'context>)>
    for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (*mut [K], V::SliceMutPtrs<'context>)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<'context, K, V> From<KeyValueSliceMutPtrs<'context, K, V>>
    for (*mut [K], V::SliceMutPtrs<'context>)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueSliceMutPtrs<'context, K, V>) -> Self {
        let KeyValueSliceMutPtrs { keys, values } = value;
        (keys, values)
    }
}

impl<'context, K, V> Debug for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa,
    V::SliceMutPtrs<'context>: Debug,
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
    V: Soa,
    V::SliceMutPtrs<'context>: PartialEq,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, K, V> Eq for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa,
    V::SliceMutPtrs<'context>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa,
    V::SliceMutPtrs<'context>: PartialOrd,
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
    V: Soa,
    V::SliceMutPtrs<'context>: Ord,
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
    V: Soa,
    V::SliceMutPtrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSliceMutPtrs<'_, K, V>
where
    V: Soa,
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
    V: Soa,
    V::SliceMutPtrs<'context>: Copy,
{
}

pub struct KeyValueSlices<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    pub keys: &'a [K],
    pub values: V::Slices<'context, 'a>,
}

impl<'context, 'a, K, V> From<(&'a [K], V::Slices<'context, 'a>)>
    for KeyValueSlices<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: (&'a [K], V::Slices<'context, 'a>)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<'context, 'a, K, V> From<KeyValueSlices<'context, 'a, K, V>>
    for (&'a [K], V::Slices<'context, 'a>)
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: KeyValueSlices<'context, 'a, K, V>) -> Self {
        let KeyValueSlices { keys, values } = value;
        (keys, values)
    }
}

impl<'context, 'a, K, V> Debug for KeyValueSlices<'context, 'a, K, V>
where
    K: Debug + 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Debug,
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
    K: 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Default,
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
    K: PartialEq + 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, 'a, K, V> Eq for KeyValueSlices<'context, 'a, K, V>
where
    K: Eq + 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueSlices<'context, 'a, K, V>
where
    K: PartialOrd + 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: PartialOrd,
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
    K: Ord + 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Ord,
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
    K: Hash + 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<'context, 'a, K, V> Clone for KeyValueSlices<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Clone,
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
    K: 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Copy,
{
}

pub struct KeyValueSlicesMut<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    pub keys: &'a mut [K],
    pub values: V::SlicesMut<'context, 'a>,
}

impl<'context, 'a, K, V> From<(&'a mut [K], V::SlicesMut<'context, 'a>)>
    for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: (&'a mut [K], V::SlicesMut<'context, 'a>)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<'context, 'a, K, V> From<KeyValueSlicesMut<'context, 'a, K, V>>
    for (&'a mut [K], V::SlicesMut<'context, 'a>)
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: KeyValueSlicesMut<'context, 'a, K, V>) -> Self {
        let KeyValueSlicesMut { keys, values } = value;
        (keys, values)
    }
}

impl<'context, 'a, K, V> Debug for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Debug + 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: Debug,
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
    K: 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: Default,
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
    K: PartialEq + 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, 'a, K, V> Eq for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Eq + 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: PartialOrd + 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: PartialOrd,
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
    K: Ord + 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: Ord,
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
    K: Hash + 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}
