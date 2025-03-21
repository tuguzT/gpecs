use alloc::vec::Vec;
use core::{
    alloc::{Layout, LayoutError},
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};
use gpecs_soa::traits::FieldDescriptor;

use crate::soa::traits::{Soa, SoaToOwned};

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

    type FieldDescriptors<'a> = KeyValueFieldDescriptors<'a, K, V>;

    #[inline]
    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_> {
        KeyValueFieldDescriptors::new(context)
    }

    type FieldOffsets<'a> = KeyValueFieldOffsets<'a, K, V>;

    #[inline]
    fn buffer_layout(
        context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
        KeyValueFieldOffsets::new(context, capacity)
    }

    type Ptrs = KeyValuePtrs<K, V>;
    type MutPtrs = KeyValueMutPtrs<K, V>;

    type ErasedPtrs = iter::Chain<iter::Once<*const u8>, <V::ErasedPtrs as IntoIterator>::IntoIter>;
    type ErasedMutPtrs =
        iter::Chain<iter::Once<*mut u8>, <V::ErasedMutPtrs as IntoIterator>::IntoIter>;

    #[inline]
    fn ptrs_erase(context: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs {
        let KeyValuePtrs { key, value } = ptrs;
        iter::once(key.cast()).chain(V::ptrs_erase(context, value))
    }

    #[inline]
    fn ptrs_erase_mut(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        iter::once(key.cast()).chain(V::ptrs_erase_mut(context, value))
    }

    #[inline]
    #[track_caller]
    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs {
        let mut ptrs = ptrs.into_iter();
        let key = ptrs
            .next()
            .expect("iterator should have at least one element");

        KeyValuePtrs {
            key: key.cast(),
            value: V::ptrs_restore(context, ptrs),
        }
    }

    #[inline]
    #[track_caller]
    fn ptrs_restore_mut(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs {
        let mut ptrs = ptrs.into_iter();
        let key = ptrs
            .next()
            .expect("iterator should have at least one element");

        KeyValueMutPtrs {
            key: key.cast(),
            value: V::ptrs_restore_mut(context, ptrs),
        }
    }

    #[inline]
    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs {
        KeyValueMutPtrs {
            key: ptr::dangling_mut(),
            value: V::ptrs_dangling(context),
        }
    }

    #[inline]
    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValuePtrs {
            key: key.cast_const(),
            value: V::ptrs_cast_const(context, value),
        }
    }

    #[inline]
    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: key.cast_mut(),
            value: V::ptrs_cast_mut(context, value),
        }
    }

    #[inline]
    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValuePtrs {
            key: unsafe { key.add(offset) },
            value: unsafe { V::ptrs_add(context, value, offset) },
        }
    }

    #[inline]
    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: unsafe { key.add(offset) },
            value: unsafe { V::ptrs_add_mut(context, value, offset) },
        }
    }

    #[inline]
    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
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
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
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
    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
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
    unsafe fn ptrs_copy(context: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
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
        src: Self::Ptrs,
        dst: Self::MutPtrs,
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
        src: Self::Ptrs,
        dst: Self::MutPtrs,
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
    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self {
        let KeyValuePtrs { key, value } = src;
        Self {
            key: unsafe { ptr::read(key) },
            value: unsafe { V::ptrs_read(context, value) },
        }
    }

    #[inline]
    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self) {
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
    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs) {
        let KeyValueMutPtrs { key, value } = ptrs;

        unsafe {
            ptr::drop_in_place(key);
            V::ptrs_drop_in_place(context, value);
        }
    }

    type NonNullPtrs = KeyValueNonNullPtrs<K, V>;

    #[inline]
    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueNonNullPtrs {
            key: unsafe { NonNull::new_unchecked(key) },
            value: unsafe { V::ptrs_to_nonnull(context, value) },
        }
    }

    #[inline]
    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        let KeyValueNonNullPtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: key.as_ptr(),
            value: V::nonnull_to_ptrs(context, value),
        }
    }

    type Vecs = KeyValueVecs<K, V>;

    #[inline]
    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs {
        KeyValueVecs {
            keys: Vec::with_capacity(capacity),
            values: V::vecs_with_capacity(context, capacity),
        }
    }

    #[inline]
    fn vecs_as_ptrs(context: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        let KeyValueVecs { keys, values } = vecs;
        KeyValuePtrs {
            key: keys.as_ptr(),
            value: V::vecs_as_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        let KeyValueVecs { keys, values } = vecs;
        KeyValueMutPtrs {
            key: keys.as_mut_ptr(),
            value: V::mut_vecs_as_ptrs(context, values),
        }
    }

    #[inline]
    #[track_caller]
    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize {
        let KeyValueVecs { keys, values } = vecs;

        let keys_len = keys.len();
        let values_len = V::vecs_len(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        let KeyValueVecs { keys, values } = vecs;

        unsafe {
            keys.set_len(len);
            V::vecs_set_len(context, values, len);
        }
    }

    type Refs<'a>
        = KeyValueRefs<'a, K, V>
    where
        Self: 'a;

    type RefsMut<'a>
        = KeyValueRefsMut<'a, K, V>
    where
        Self: 'a;

    #[inline]
    unsafe fn ptrs_to_refs<'a>(context: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueRefs {
            key: unsafe { &*key },
            value: unsafe { V::ptrs_to_refs(context, value) },
        }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a> {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueRefsMut {
            key: unsafe { &mut *key },
            value: unsafe { V::ptrs_to_refs_mut(context, value) },
        }
    }

    #[inline]
    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        let KeyValueRefs { key, value } = refs;
        KeyValuePtrs {
            key: ptr::from_ref(key),
            value: V::refs_as_ptrs(context, value),
        }
    }

    #[inline]
    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let KeyValueRefsMut { key, value } = refs;
        KeyValueMutPtrs {
            key: ptr::from_mut(key),
            value: V::mut_refs_as_ptrs(context, value),
        }
    }

    #[inline]
    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let KeyValueRefsMut { key, value } = refs;
        KeyValueRefs {
            key: &*key,
            value: V::mut_refs_as_refs(context, value),
        }
    }

    type SlicePtrs = KeyValueSlicePtrs<K, V>;

    type SliceMutPtrs = KeyValueSliceMutPtrs<K, V>;

    #[inline]
    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueSlicePtrs {
            keys: ptr::slice_from_raw_parts(key, len),
            values: V::slices_from_raw_parts(context, value, len),
        }
    }

    #[inline]
    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueSliceMutPtrs {
            keys: ptr::slice_from_raw_parts_mut(key, len),
            values: V::slices_from_raw_parts_mut(context, value, len),
        }
    }

    #[inline]
    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs {
        let KeyValueSliceMutPtrs { keys, values } = slices;
        KeyValueSlicePtrs {
            keys: keys.cast_const(),
            values: V::slice_ptrs_cast_const(context, values),
        }
    }

    #[inline]
    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        let KeyValueSlicePtrs { keys, values } = slices;
        KeyValueSliceMutPtrs {
            keys: keys.cast_mut(),
            values: V::slice_ptrs_cast_mut(context, values),
        }
    }

    #[inline]
    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize {
        let KeyValueSlicePtrs { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slice_ptrs_len(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    fn slice_ptrs_len_mut(context: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        let KeyValueSliceMutPtrs { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slice_ptrs_len_mut(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        let KeyValueSlicePtrs { keys, values } = slices;
        KeyValuePtrs {
            key: keys.cast(), // should be `keys.as_ptr()` but it's unstable
            value: V::slice_ptrs_as_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::MutPtrs {
        let KeyValueSliceMutPtrs { keys, values } = slices;
        KeyValueMutPtrs {
            key: keys.cast(), // should be `keys.as_mut_ptr()` but it's unstable
            value: V::mut_slice_ptrs_as_ptrs(context, values),
        }
    }

    type Slices<'a>
        = KeyValueSlices<'a, K, V>
    where
        Self: 'a;

    type SlicesMut<'a>
        = KeyValueSlicesMut<'a, K, V>
    where
        Self: 'a;

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a> {
        let KeyValueSlicePtrs { keys, values } = slices.clone();

        let len = Self::slice_ptrs_len(context, slices);
        KeyValueSlices {
            keys: unsafe { slice::from_raw_parts(keys.cast(), len) },
            values: unsafe { V::slice_ptrs_to_slices(context, values) },
        }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let KeyValueSliceMutPtrs { keys, values } = slices.clone();

        let len = Self::slice_ptrs_len_mut(context, slices);
        KeyValueSlicesMut {
            keys: unsafe { slice::from_raw_parts_mut(keys.cast(), len) },
            values: unsafe { V::slice_ptrs_to_slices_mut(context, values) },
        }
    }

    #[inline]
    #[track_caller]
    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        let KeyValueSlices { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slices_len(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    #[track_caller]
    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        let KeyValueSlicesMut { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slices_len_mut(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs {
        let KeyValueSlices { keys, values } = slices;
        KeyValueSlicePtrs {
            keys: ptr::from_ref(keys),
            values: V::slice_refs_as_slice_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueSliceMutPtrs {
            keys: ptr::from_mut(keys),
            values: V::mut_slice_refs_as_slice_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueSlices {
            keys: &*keys,
            values: V::mut_slices_as_slices(context, values),
        }
    }

    #[inline]
    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        let KeyValueSlices { keys, values } = slices;
        KeyValuePtrs {
            key: keys.as_ptr(),
            value: V::slice_refs_as_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueMutPtrs {
            key: keys.as_mut_ptr(),
            value: V::mut_slice_refs_as_ptrs(context, values),
        }
    }

    #[inline]
    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs) {
        let KeyValueSliceMutPtrs { keys, values } = slices;

        unsafe {
            ptr::drop_in_place(keys);
            V::slices_drop_in_place(context, values);
        }
    }
}

pub struct KeyValueFieldDescriptors<'a, K, V>
where
    V: Soa,
{
    key: FieldDescriptor,
    value: V::FieldDescriptors<'a>,
    phantom: PhantomData<fn() -> K>,
}

impl<'a, K, V> KeyValueFieldDescriptors<'a, K, V>
where
    V: Soa,
{
    #[inline]
    pub fn new(context: &'a V::Context) -> Self {
        Self {
            key: FieldDescriptor::of::<K>(),
            phantom: PhantomData,
            value: V::field_descriptors(context),
        }
    }
}

impl<'a, K, V> Debug for KeyValueFieldDescriptors<'a, K, V>
where
    V: Soa,
    V::FieldDescriptors<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value, .. } = self;
        f.debug_struct("KeyValueFieldLayouts")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'a, K, V> Clone for KeyValueFieldDescriptors<'a, K, V>
where
    V: Soa,
    V::FieldDescriptors<'a>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            phantom: self.phantom.clone(),
            value: self.value.clone(),
        }
    }
}

impl<'a, K, V> Copy for KeyValueFieldDescriptors<'a, K, V>
where
    V: Soa,
    V::FieldDescriptors<'a>: Copy,
{
}

impl<'a, K, V> IntoIterator for KeyValueFieldDescriptors<'a, K, V>
where
    V: Soa,
{
    type Item = FieldDescriptor;

    type IntoIter = iter::Chain<
        iter::Once<FieldDescriptor>,
        iter::Map<
            <V::FieldDescriptors<'a> as IntoIterator>::IntoIter,
            fn(<V::FieldDescriptors<'a> as IntoIterator>::Item) -> FieldDescriptor,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            key: key_layout,
            value: value_layouts,
            ..
        } = self;

        let f: fn(<V::FieldDescriptors<'a> as IntoIterator>::Item) -> _ = |layout| *layout.borrow();
        let value_layouts = value_layouts.into_iter().map(f);
        iter::once(key_layout).chain(value_layouts)
    }
}

pub struct KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
{
    offset_from_keys: usize,
    keys: PhantomData<fn() -> K>,
    values: V::FieldOffsets<'a>,
}

impl<'a, K, V> KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
{
    pub fn new(context: &'a V::Context, capacity: usize) -> Result<(Layout, Self), LayoutError> {
        let (mut layout, offsets) = V::buffer_layout(context, capacity)?;

        let keys = Layout::array::<K>(capacity)?;
        let offset_from_keys;
        (layout, offset_from_keys) = keys.extend(layout)?;

        let this = Self {
            offset_from_keys,
            keys: PhantomData,
            values: offsets,
        };
        Ok((layout, this))
    }
}

impl<'a, K, V> Debug for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
    V::FieldOffsets<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueFieldOffsets")
            .field("offset_from_keys", &self.offset_from_keys)
            .field("values", &self.values)
            .finish()
    }
}

impl<'a, K, V> PartialEq for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
    V::FieldOffsets<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.offset_from_keys == other.offset_from_keys
            && self.keys == other.keys
            && self.values == other.values
    }
}

impl<'a, K, V> Eq for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
    V::FieldOffsets<'a>: Eq,
{
}

impl<'a, K, V> PartialOrd for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
    V::FieldOffsets<'a>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.offset_from_keys.partial_cmp(&other.offset_from_keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<'a, K, V> Ord for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
    V::FieldOffsets<'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.offset_from_keys.cmp(&other.offset_from_keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.values.cmp(&other.values)
    }
}

impl<'a, K, V> Hash for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
    V::FieldOffsets<'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.offset_from_keys.hash(state);
        self.keys.hash(state);
        self.values.hash(state);
    }
}

impl<'a, K, V> Clone for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
    V::FieldOffsets<'a>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            offset_from_keys: self.offset_from_keys.clone(),
            keys: self.keys.clone(),
            values: self.values.clone(),
        }
    }
}

impl<'a, K, V> Copy for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
    V::FieldOffsets<'a>: Copy,
{
}

impl<'a, K, V> IntoIterator for KeyValueFieldOffsets<'a, K, V>
where
    V: Soa,
{
    type Item = usize;

    type IntoIter = iter::Chain<
        iter::Once<usize>,
        iter::Scan<
            <<V as Soa>::FieldOffsets<'a> as IntoIterator>::IntoIter,
            usize,
            fn(&mut usize, usize) -> Option<usize>,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            offset_from_keys,
            values,
            ..
        } = self;

        let key_offset = 0;
        let f: fn(&mut _, _) -> _ = |&mut offset_from_keys, offset| Some(offset + offset_from_keys);
        let value_offsets = values.into_iter().scan(offset_from_keys, f);
        iter::once(key_offset).chain(value_offsets)
    }
}

pub struct KeyValuePtrs<K, V>
where
    V: Soa,
{
    pub key: *const K,
    pub value: V::Ptrs,
}

impl<K, V> From<(*const K, V::Ptrs)> for KeyValuePtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (*const K, V::Ptrs)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValuePtrs<K, V>> for (*const K, V::Ptrs)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValuePtrs<K, V>) -> Self {
        let KeyValuePtrs { key, value } = value;
        (key, value)
    }
}

impl<K, V> Debug for KeyValuePtrs<K, V>
where
    V: Soa,
    V::Ptrs: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValuePtrs")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValuePtrs<K, V>
where
    V: Soa,
    V::Ptrs: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl<K, V> Eq for KeyValuePtrs<K, V>
where
    V: Soa,
    V::Ptrs: Eq,
{
}

impl<K, V> PartialOrd for KeyValuePtrs<K, V>
where
    V: Soa,
    V::Ptrs: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<K, V> Ord for KeyValuePtrs<K, V>
where
    V: Soa,
    V::Ptrs: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<K, V> Hash for KeyValuePtrs<K, V>
where
    V: Soa,
    V::Ptrs: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

impl<K, V> Clone for KeyValuePtrs<K, V>
where
    V: Soa,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            value: self.value.clone(),
        }
    }
}

impl<K, V> Copy for KeyValuePtrs<K, V>
where
    V: Soa,
    V::Ptrs: Copy,
{
}

pub struct KeyValueMutPtrs<K, V>
where
    V: Soa,
{
    pub key: *mut K,
    pub value: V::MutPtrs,
}

impl<K, V> From<(*mut K, V::MutPtrs)> for KeyValueMutPtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (*mut K, V::MutPtrs)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValueMutPtrs<K, V>> for (*mut K, V::MutPtrs)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueMutPtrs<K, V>) -> Self {
        let KeyValueMutPtrs { key, value } = value;
        (key, value)
    }
}

impl<K, V> Debug for KeyValueMutPtrs<K, V>
where
    V: Soa,
    V::MutPtrs: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueMutPtrs")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueMutPtrs<K, V>
where
    V: Soa,
    V::MutPtrs: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl<K, V> Eq for KeyValueMutPtrs<K, V>
where
    V: Soa,
    V::MutPtrs: Eq,
{
}

impl<K, V> PartialOrd for KeyValueMutPtrs<K, V>
where
    V: Soa,
    V::MutPtrs: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<K, V> Ord for KeyValueMutPtrs<K, V>
where
    V: Soa,
    V::MutPtrs: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<K, V> Hash for KeyValueMutPtrs<K, V>
where
    V: Soa,
    V::MutPtrs: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

impl<K, V> Clone for KeyValueMutPtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            value: self.value.clone(),
        }
    }
}

impl<K, V> Copy for KeyValueMutPtrs<K, V>
where
    V: Soa,
    V::MutPtrs: Copy,
{
}

pub struct KeyValueNonNullPtrs<K, V>
where
    V: Soa,
{
    pub key: NonNull<K>,
    pub value: V::NonNullPtrs,
}

impl<K, V> From<(NonNull<K>, V::NonNullPtrs)> for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (NonNull<K>, V::NonNullPtrs)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValueNonNullPtrs<K, V>> for (NonNull<K>, V::NonNullPtrs)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueNonNullPtrs<K, V>) -> Self {
        let KeyValueNonNullPtrs { key, value } = value;
        (key, value)
    }
}

impl<K, V> Debug for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
    V::NonNullPtrs: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueNonNullPtrs")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
    V::NonNullPtrs: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl<K, V> Eq for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
    V::NonNullPtrs: Eq,
{
}

impl<K, V> PartialOrd for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
    V::NonNullPtrs: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<K, V> Ord for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
    V::NonNullPtrs: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<K, V> Hash for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
    V::NonNullPtrs: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

impl<K, V> Clone for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            value: self.value.clone(),
        }
    }
}

impl<K, V> Copy for KeyValueNonNullPtrs<K, V>
where
    V: Soa,
    V::NonNullPtrs: Copy,
{
}

pub struct KeyValueVecs<K, V>
where
    V: Soa,
{
    pub keys: Vec<K>,
    pub values: V::Vecs,
}

impl<K, V> From<(Vec<K>, V::Vecs)> for KeyValueVecs<K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (Vec<K>, V::Vecs)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<K, V> From<KeyValueVecs<K, V>> for (Vec<K>, V::Vecs)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueVecs<K, V>) -> Self {
        let KeyValueVecs { keys, values } = value;
        (keys, values)
    }
}

impl<K, V> Debug for KeyValueVecs<K, V>
where
    K: Debug,
    V: Soa,
    V::Vecs: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueVecs")
            .field("keys", &self.keys)
            .field("values", &self.values)
            .finish()
    }
}

impl<K, V> Default for KeyValueVecs<K, V>
where
    V: Soa,
    V::Vecs: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Default::default(),
        }
    }
}

impl<K, V> PartialEq for KeyValueVecs<K, V>
where
    K: PartialEq,
    V: Soa,
    V::Vecs: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.values == other.values
    }
}

impl<K, V> Eq for KeyValueVecs<K, V>
where
    K: Eq,
    V: Soa,
    V::Vecs: Eq,
{
}

impl<K, V> PartialOrd for KeyValueVecs<K, V>
where
    K: PartialOrd,
    V: Soa,
    V::Vecs: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<K, V> Ord for KeyValueVecs<K, V>
where
    K: Ord,
    V: Soa,
    V::Vecs: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.values.cmp(&other.values)
    }
}

impl<K, V> Hash for KeyValueVecs<K, V>
where
    K: Hash,
    V: Soa,
    V::Vecs: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}

impl<K, V> Clone for KeyValueVecs<K, V>
where
    K: Clone,
    V: Soa,
    V::Vecs: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            values: self.values.clone(),
        }
    }
}

pub struct KeyValueRefs<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    pub key: &'a K,
    pub value: V::Refs<'a>,
}

impl<'a, K, V> From<(&'a K, V::Refs<'a>)> for KeyValueRefs<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: (&'a K, V::Refs<'a>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'a, K, V> From<KeyValueRefs<'a, K, V>> for (&'a K, V::Refs<'a>)
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: KeyValueRefs<'a, K, V>) -> Self {
        let KeyValueRefs { key, value } = value;
        (key, value)
    }
}

impl<'a, K, V> Debug for KeyValueRefs<'a, K, V>
where
    K: Debug + 'a,
    V: Soa + 'a,
    V::Refs<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueRefs")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<'a, K, V> PartialEq for KeyValueRefs<'a, K, V>
where
    K: PartialEq + 'a,
    V: Soa + 'a,
    V::Refs<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl<'a, K, V> Eq for KeyValueRefs<'a, K, V>
where
    K: Eq + 'a,
    V: Soa + 'a,
    V::Refs<'a>: Eq,
{
}

impl<'a, K, V> PartialOrd for KeyValueRefs<'a, K, V>
where
    K: PartialOrd + 'a,
    V: Soa + 'a,
    V::Refs<'a>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<'a, K, V> Ord for KeyValueRefs<'a, K, V>
where
    K: Ord + 'a,
    V: Soa + 'a,
    V::Refs<'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<'a, K, V> Hash for KeyValueRefs<'a, K, V>
where
    K: Hash + 'a,
    V: Soa + 'a,
    V::Refs<'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

impl<'a, K, V> Clone for KeyValueRefs<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
    V::Refs<'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            value: self.value.clone(),
        }
    }
}

impl<'a, K, V> Copy for KeyValueRefs<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
    V::Refs<'a>: Copy,
{
}

impl<'a, K, V> SoaToOwned<'a> for KeyValueRefs<'a, K, V>
where
    K: Clone,
    V: Soa,
    V::Refs<'a>: SoaToOwned<'a, Owned = V>,
{
    type Owned
        = KeyValuePair<K, V>
    where
        Self: 'a;

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

pub struct KeyValueRefsMut<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    pub key: &'a mut K,
    pub value: V::RefsMut<'a>,
}

impl<'a, K, V> From<(&'a mut K, V::RefsMut<'a>)> for KeyValueRefsMut<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: (&'a mut K, V::RefsMut<'a>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'a, K, V> From<KeyValueRefsMut<'a, K, V>> for (&'a mut K, V::RefsMut<'a>)
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: KeyValueRefsMut<'a, K, V>) -> Self {
        let KeyValueRefsMut { key, value } = value;
        (key, value)
    }
}

impl<'a, K, V> Debug for KeyValueRefsMut<'a, K, V>
where
    K: Debug + 'a,
    V: Soa + 'a,
    V::RefsMut<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueRefsMut")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<'a, K, V> PartialEq for KeyValueRefsMut<'a, K, V>
where
    K: PartialEq + 'a,
    V: Soa + 'a,
    V::RefsMut<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl<'a, K, V> Eq for KeyValueRefsMut<'a, K, V>
where
    K: Eq + 'a,
    V: Soa + 'a,
    V::RefsMut<'a>: Eq,
{
}

impl<'a, K, V> PartialOrd for KeyValueRefsMut<'a, K, V>
where
    K: PartialOrd + 'a,
    V: Soa + 'a,
    V::RefsMut<'a>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<'a, K, V> Ord for KeyValueRefsMut<'a, K, V>
where
    K: Ord + 'a,
    V: Soa + 'a,
    V::RefsMut<'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<'a, K, V> Hash for KeyValueRefsMut<'a, K, V>
where
    K: Hash + 'a,
    V: Soa + 'a,
    V::RefsMut<'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

pub struct KeyValueSlicePtrs<K, V>
where
    V: Soa,
{
    pub keys: *const [K],
    pub values: V::SlicePtrs,
}

impl<K, V> From<(*const [K], V::SlicePtrs)> for KeyValueSlicePtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (*const [K], V::SlicePtrs)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<K, V> From<KeyValueSlicePtrs<K, V>> for (*const [K], V::SlicePtrs)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueSlicePtrs<K, V>) -> Self {
        let KeyValueSlicePtrs { keys, values } = value;
        (keys, values)
    }
}

impl<K, V> Debug for KeyValueSlicePtrs<K, V>
where
    V: Soa,
    V::SlicePtrs: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueSlicePtrs")
            .field("keys", &self.keys)
            .field("values", &self.values)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueSlicePtrs<K, V>
where
    V: Soa,
    V::SlicePtrs: PartialEq,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.values == other.values
    }
}

impl<K, V> Eq for KeyValueSlicePtrs<K, V>
where
    V: Soa,
    V::SlicePtrs: Eq,
{
}

impl<K, V> PartialOrd for KeyValueSlicePtrs<K, V>
where
    V: Soa,
    V::SlicePtrs: PartialOrd,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<K, V> Ord for KeyValueSlicePtrs<K, V>
where
    V: Soa,
    V::SlicePtrs: Ord,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.values.cmp(&other.values)
    }
}

impl<K, V> Hash for KeyValueSlicePtrs<K, V>
where
    V: Soa,
    V::SlicePtrs: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSlicePtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            keys: self.keys,
            values: self.values.clone(),
        }
    }
}

impl<K, V> Copy for KeyValueSlicePtrs<K, V>
where
    V: Soa,
    V::SlicePtrs: Copy,
{
}

pub struct KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
{
    pub keys: *mut [K],
    pub values: V::SliceMutPtrs,
}

impl<K, V> From<(*mut [K], V::SliceMutPtrs)> for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn from(value: (*mut [K], V::SliceMutPtrs)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<K, V> From<KeyValueSliceMutPtrs<K, V>> for (*mut [K], V::SliceMutPtrs)
where
    V: Soa,
{
    #[inline]
    fn from(value: KeyValueSliceMutPtrs<K, V>) -> Self {
        let KeyValueSliceMutPtrs { keys, values } = value;
        (keys, values)
    }
}

impl<K, V> Debug for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
    V::SliceMutPtrs: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueSliceMutPtrs")
            .field("keys", &self.keys)
            .field("values", &self.values)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
    V::SliceMutPtrs: PartialEq,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.values == other.values
    }
}

impl<K, V> Eq for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
    V::SliceMutPtrs: Eq,
{
}

impl<K, V> PartialOrd for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
    V::SliceMutPtrs: PartialOrd,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<K, V> Ord for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
    V::SliceMutPtrs: Ord,
{
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.values.cmp(&other.values)
    }
}

impl<K, V> Hash for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
    V::SliceMutPtrs: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            keys: self.keys,
            values: self.values.clone(),
        }
    }
}

impl<K, V> Copy for KeyValueSliceMutPtrs<K, V>
where
    V: Soa,
    V::SliceMutPtrs: Copy,
{
}

pub struct KeyValueSlices<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    pub keys: &'a [K],
    pub values: V::Slices<'a>,
}

impl<'a, K, V> From<(&'a [K], V::Slices<'a>)> for KeyValueSlices<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: (&'a [K], V::Slices<'a>)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<'a, K, V> From<KeyValueSlices<'a, K, V>> for (&'a [K], V::Slices<'a>)
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: KeyValueSlices<'a, K, V>) -> Self {
        let KeyValueSlices { keys, values } = value;
        (keys, values)
    }
}

impl<'a, K, V> Debug for KeyValueSlices<'a, K, V>
where
    K: Debug + 'a,
    V: Soa + 'a,
    V::Slices<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueSlices")
            .field("keys", &self.keys)
            .field("values", &self.values)
            .finish()
    }
}

impl<'a, K, V> Default for KeyValueSlices<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
    V::Slices<'a>: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Default::default(),
        }
    }
}

impl<'a, K, V> PartialEq for KeyValueSlices<'a, K, V>
where
    K: PartialEq + 'a,
    V: Soa + 'a,
    V::Slices<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.values == other.values
    }
}

impl<'a, K, V> Eq for KeyValueSlices<'a, K, V>
where
    K: Eq + 'a,
    V: Soa + 'a,
    V::Slices<'a>: Eq,
{
}

impl<'a, K, V> PartialOrd for KeyValueSlices<'a, K, V>
where
    K: PartialOrd + 'a,
    V: Soa + 'a,
    V::Slices<'a>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<'a, K, V> Ord for KeyValueSlices<'a, K, V>
where
    K: Ord + 'a,
    V: Soa + 'a,
    V::Slices<'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.values.cmp(&other.values)
    }
}

impl<'a, K, V> Hash for KeyValueSlices<'a, K, V>
where
    K: Hash + 'a,
    V: Soa + 'a,
    V::Slices<'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}

impl<'a, K, V> Clone for KeyValueSlices<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
    V::Slices<'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            keys: self.keys,
            values: self.values.clone(),
        }
    }
}

impl<'a, K, V> Copy for KeyValueSlices<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
    V::Slices<'a>: Copy,
{
}

pub struct KeyValueSlicesMut<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    pub keys: &'a mut [K],
    pub values: V::SlicesMut<'a>,
}

impl<'a, K, V> From<(&'a mut [K], V::SlicesMut<'a>)> for KeyValueSlicesMut<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: (&'a mut [K], V::SlicesMut<'a>)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<'a, K, V> From<KeyValueSlicesMut<'a, K, V>> for (&'a mut [K], V::SlicesMut<'a>)
where
    K: 'a,
    V: Soa + 'a,
{
    #[inline]
    fn from(value: KeyValueSlicesMut<'a, K, V>) -> Self {
        let KeyValueSlicesMut { keys, values } = value;
        (keys, values)
    }
}

impl<'a, K, V> Debug for KeyValueSlicesMut<'a, K, V>
where
    K: Debug + 'a,
    V: Soa + 'a,
    V::SlicesMut<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueSlicesMut")
            .field("keys", &self.keys)
            .field("values", &self.values)
            .finish()
    }
}

impl<'a, K, V> Default for KeyValueSlicesMut<'a, K, V>
where
    K: 'a,
    V: Soa + 'a,
    V::SlicesMut<'a>: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Default::default(),
        }
    }
}

impl<'a, K, V> PartialEq for KeyValueSlicesMut<'a, K, V>
where
    K: PartialEq + 'a,
    V: Soa + 'a,
    V::SlicesMut<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.values == other.values
    }
}

impl<'a, K, V> Eq for KeyValueSlicesMut<'a, K, V>
where
    K: Eq + 'a,
    V: Soa + 'a,
    V::SlicesMut<'a>: Eq,
{
}

impl<'a, K, V> PartialOrd for KeyValueSlicesMut<'a, K, V>
where
    K: PartialOrd + 'a,
    V: Soa + 'a,
    V::SlicesMut<'a>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<'a, K, V> Ord for KeyValueSlicesMut<'a, K, V>
where
    K: Ord + 'a,
    V: Soa + 'a,
    V::SlicesMut<'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.values.cmp(&other.values)
    }
}

impl<'a, K, V> Hash for KeyValueSlicesMut<'a, K, V>
where
    K: Hash + 'a,
    V: Soa + 'a,
    V::SlicesMut<'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}
