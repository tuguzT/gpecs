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

use gpecs_soa::Soa;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct KeyValuePair<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> KeyValuePair<K, V> {
    pub const fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> From<(K, V)> for KeyValuePair<K, V> {
    fn from(value: (K, V)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValuePair<K, V>> for (K, V) {
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
    type FieldLayouts = KeyValueFieldLayouts<K, V>;

    fn field_layouts() -> Self::FieldLayouts {
        Default::default()
    }

    fn buffer_layout(
        capacity: usize,
    ) -> Result<(Layout, impl IntoIterator<Item = usize>), LayoutError> {
        let (mut layout, offsets) = V::buffer_layout(capacity)?;

        let keys = Layout::array::<K>(capacity)?;
        let offset_from_keys;
        (layout, offset_from_keys) = keys.extend(layout)?;

        let key_offset = 0;
        let offsets = offsets
            .into_iter()
            .map(move |offset| offset + offset_from_keys);
        let offsets = iter::once(key_offset).chain(offsets);

        Ok((layout, offsets))
    }

    type Ptrs = KeyValuePtrs<K, V>;

    type MutPtrs = KeyValueMutPtrs<K, V>;

    fn ptrs_dangling() -> Self::MutPtrs {
        KeyValueMutPtrs {
            key: ptr::dangling_mut(),
            value: V::ptrs_dangling(),
        }
    }

    #[track_caller]
    unsafe fn ptrs(ptr: *mut u8, offsets: impl IntoIterator<Item = usize>) -> Self::MutPtrs {
        let mut offsets = offsets.into_iter();
        let key_offset = offsets
            .next()
            .expect("iterator should have at least one element");

        KeyValueMutPtrs {
            key: unsafe { ptr.add(key_offset).cast() },
            value: unsafe { V::ptrs(ptr, offsets) },
        }
    }

    fn ptrs_cast_const(ptrs: Self::MutPtrs) -> Self::Ptrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValuePtrs {
            key: key.cast_const(),
            value: V::ptrs_cast_const(value),
        }
    }

    fn ptrs_cast_mut(ptrs: Self::Ptrs) -> Self::MutPtrs {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: key.cast_mut(),
            value: V::ptrs_cast_mut(value),
        }
    }

    unsafe fn ptrs_add(ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValuePtrs {
            key: unsafe { key.add(offset) },
            value: unsafe { V::ptrs_add(value, offset) },
        }
    }

    unsafe fn ptrs_add_mut(ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: unsafe { key.add(offset) },
            value: unsafe { V::ptrs_add_mut(value, offset) },
        }
    }

    unsafe fn ptrs_offset_from(ptrs: Self::Ptrs, origin: Self::Ptrs) -> isize {
        let KeyValuePtrs { key, value } = ptrs;
        let KeyValuePtrs {
            key: origin_key,
            value: origin_value,
        } = origin;

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset = unsafe { V::ptrs_offset_from(value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    unsafe fn ptrs_offset_from_mut(ptrs: Self::MutPtrs, origin: Self::Ptrs) -> isize {
        let KeyValueMutPtrs { key, value } = ptrs;
        let KeyValuePtrs {
            key: origin_key,
            value: origin_value,
        } = origin;

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset = unsafe { V::ptrs_offset_from_mut(value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    unsafe fn ptrs_swap(a: Self::MutPtrs, b: Self::MutPtrs) {
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
            V::ptrs_swap(a_value, b_value);
        }
    }

    unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
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
            V::ptrs_copy(src_value, dst_value, len);
        }
    }

    unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = src;
        let KeyValueMutPtrs {
            key: dst_key,
            value: dst_value,
        } = dst;

        unsafe {
            V::ptrs_copy_rev(src_value, dst_value, len);
            ptr::copy(src_key, dst_key, len);
        }
    }

    unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
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
            V::ptrs_copy_nonoverlapping(src_value, dst_value, len);
        }
    }

    unsafe fn ptrs_read(src: Self::Ptrs) -> Self {
        let KeyValuePtrs { key, value } = src;
        Self {
            key: unsafe { ptr::read(key) },
            value: unsafe { V::ptrs_read(value) },
        }
    }

    unsafe fn ptrs_write(dst: Self::MutPtrs, value: Self) {
        let KeyValueMutPtrs {
            key: key_ptr,
            value: value_ptr,
        } = dst;
        let Self { key, value } = value;

        unsafe {
            ptr::write(key_ptr, key);
            V::ptrs_write(value_ptr, value);
        }
    }

    unsafe fn ptrs_drop_in_place(ptrs: Self::MutPtrs) {
        let KeyValueMutPtrs { key, value } = ptrs;

        unsafe {
            ptr::drop_in_place(key);
            V::ptrs_drop_in_place(value);
        }
    }

    type NonNullPtrs = KeyValueNonNullPtrs<K, V>;

    unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueNonNullPtrs {
            key: unsafe { NonNull::new_unchecked(key) },
            value: unsafe { V::ptrs_to_nonnull(value) },
        }
    }

    fn nonnull_to_ptrs(ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        let KeyValueNonNullPtrs { key, value } = ptrs;
        KeyValueMutPtrs {
            key: key.as_ptr(),
            value: V::nonnull_to_ptrs(value),
        }
    }

    type Vecs = KeyValueVecs<K, V>;

    fn vecs_with_capacity(capacity: usize) -> Self::Vecs {
        KeyValueVecs {
            keys: Vec::with_capacity(capacity),
            values: V::vecs_with_capacity(capacity),
        }
    }

    fn vecs_as_ptrs(vecs: &Self::Vecs) -> Self::Ptrs {
        let KeyValueVecs { keys, values } = vecs;
        KeyValuePtrs {
            key: keys.as_ptr(),
            value: V::vecs_as_ptrs(values),
        }
    }

    fn mut_vecs_as_ptrs(vecs: &mut Self::Vecs) -> Self::MutPtrs {
        let KeyValueVecs { keys, values } = vecs;
        KeyValueMutPtrs {
            key: keys.as_mut_ptr(),
            value: V::mut_vecs_as_ptrs(values),
        }
    }

    fn vecs_len(vecs: &Self::Vecs) -> usize {
        let KeyValueVecs { keys, values } = vecs;

        let keys_len = keys.len();
        let values_len = V::vecs_len(values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    unsafe fn vecs_set_len(vecs: &mut Self::Vecs, len: usize) {
        let KeyValueVecs { keys, values } = vecs;

        unsafe {
            keys.set_len(len);
            V::vecs_set_len(values, len);
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

    unsafe fn ptrs_to_refs<'a>(ptrs: Self::Ptrs) -> Self::Refs<'a> {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueRefs {
            key: unsafe { &*key },
            value: unsafe { V::ptrs_to_refs(value) },
        }
    }

    unsafe fn ptrs_to_refs_mut<'a>(ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueRefsMut {
            key: unsafe { &mut *key },
            value: unsafe { V::ptrs_to_refs_mut(value) },
        }
    }

    fn refs_as_ptrs(refs: Self::Refs<'_>) -> Self::Ptrs {
        let KeyValueRefs { key, value } = refs;
        KeyValuePtrs {
            key: ptr::from_ref(key),
            value: V::refs_as_ptrs(value),
        }
    }

    fn mut_refs_as_ptrs(refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let KeyValueRefsMut { key, value } = refs;
        KeyValueMutPtrs {
            key: ptr::from_mut(key),
            value: V::mut_refs_as_ptrs(value),
        }
    }

    fn mut_refs_as_refs(refs: Self::RefsMut<'_>) -> Self::Refs<'_> {
        let KeyValueRefsMut { key, value } = refs;
        KeyValueRefs {
            key: &*key,
            value: V::mut_refs_as_refs(value),
        }
    }

    type SlicePtrs = KeyValueSlicePtrs<K, V>;

    type SliceMutPtrs = KeyValueSliceMutPtrs<K, V>;

    fn slices_from_raw_parts(ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs {
        let KeyValuePtrs { key, value } = ptrs;
        KeyValueSlicePtrs {
            keys: ptr::slice_from_raw_parts(key, len),
            values: V::slices_from_raw_parts(value, len),
        }
    }

    fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs {
        let KeyValueMutPtrs { key, value } = ptrs;
        KeyValueSliceMutPtrs {
            keys: ptr::slice_from_raw_parts_mut(key, len),
            values: V::slices_from_raw_parts_mut(value, len),
        }
    }

    fn slices_len(slices: Self::SlicePtrs) -> usize {
        let KeyValueSlicePtrs { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slices_len(values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    fn slices_len_mut(slices: Self::SliceMutPtrs) -> usize {
        let KeyValueSliceMutPtrs { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slices_len_mut(values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    type Slices<'a>
        = KeyValueSlices<'a, K, V>
    where
        Self: 'a;

    type SlicesMut<'a>
        = KeyValueSlicesMut<'a, K, V>
    where
        Self: 'a;

    unsafe fn slice_ptrs_to_slices<'a>(slices: Self::SlicePtrs) -> Self::Slices<'a> {
        let KeyValueSlicePtrs { keys, values } = slices;
        KeyValueSlices {
            keys: unsafe { slice::from_raw_parts(keys.cast(), Self::slices_len(slices)) },
            values: unsafe { V::slice_ptrs_to_slices(values) },
        }
    }

    unsafe fn slice_ptrs_to_slices_mut<'a>(slices: Self::SliceMutPtrs) -> Self::SlicesMut<'a> {
        let KeyValueSliceMutPtrs { keys, values } = slices;
        KeyValueSlicesMut {
            keys: unsafe { slice::from_raw_parts_mut(keys.cast(), Self::slices_len_mut(slices)) },
            values: unsafe { V::slice_ptrs_to_slices_mut(values) },
        }
    }

    fn slice_refs_as_slice_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs {
        let KeyValueSlices { keys, values } = slices;
        KeyValueSlicePtrs {
            keys: ptr::from_ref(keys),
            values: V::slice_refs_as_slice_ptrs(values),
        }
    }

    fn mut_slice_refs_as_slice_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueSliceMutPtrs {
            keys: ptr::from_mut(keys),
            values: V::mut_slice_refs_as_slice_ptrs(values),
        }
    }

    fn mut_slices_as_slices(slices: Self::SlicesMut<'_>) -> Self::Slices<'_> {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueSlices {
            keys: &*keys,
            values: V::mut_slices_as_slices(values),
        }
    }

    fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::Ptrs {
        let KeyValueSlices { keys, values } = slices;
        KeyValuePtrs {
            key: keys.as_ptr(),
            value: V::slice_refs_as_ptrs(values),
        }
    }

    fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueMutPtrs {
            key: keys.as_mut_ptr(),
            value: V::mut_slice_refs_as_ptrs(values),
        }
    }

    unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs) {
        let KeyValueSliceMutPtrs { keys, values } = slices;

        unsafe {
            ptr::drop_in_place(keys);
            V::slices_drop_in_place(values);
        }
    }
}

pub struct KeyValueFieldLayouts<K, V>
where
    V: Soa,
{
    key_layout: Layout,
    key_phantom: PhantomData<fn() -> K>,
    value_layouts: V::FieldLayouts,
}

impl<K, V> Debug for KeyValueFieldLayouts<K, V>
where
    V: Soa,
    V::FieldLayouts: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueFieldLayouts")
            .field("key_layout", &self.key_layout)
            .field("value_layouts", &self.value_layouts)
            .finish()
    }
}

impl<K, V> Default for KeyValueFieldLayouts<K, V>
where
    V: Soa,
{
    fn default() -> Self {
        Self {
            key_layout: Layout::new::<K>(),
            key_phantom: PhantomData,
            value_layouts: V::field_layouts(),
        }
    }
}

impl<K, V> PartialEq for KeyValueFieldLayouts<K, V>
where
    V: Soa,
    V::FieldLayouts: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key_layout == other.key_layout
            && self.key_phantom == other.key_phantom
            && self.value_layouts == other.value_layouts
    }
}

impl<K, V> Eq for KeyValueFieldLayouts<K, V>
where
    V: Soa,
    V::FieldLayouts: Eq,
{
}

impl<K, V> Hash for KeyValueFieldLayouts<K, V>
where
    V: Soa,
    V::FieldLayouts: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key_layout.hash(state);
        self.key_phantom.hash(state);
        self.value_layouts.hash(state);
    }
}

impl<K, V> Clone for KeyValueFieldLayouts<K, V>
where
    V: Soa,
    V::FieldLayouts: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key_layout: self.key_layout.clone(),
            key_phantom: self.key_phantom.clone(),
            value_layouts: self.value_layouts.clone(),
        }
    }
}

impl<K, V> Copy for KeyValueFieldLayouts<K, V>
where
    V: Soa,
    V::FieldLayouts: Copy,
{
}

impl<K, V> IntoIterator for KeyValueFieldLayouts<K, V>
where
    V: Soa,
{
    type Item = Layout;

    type IntoIter = iter::Chain<
        iter::Once<Layout>,
        iter::Map<
            <V::FieldLayouts as IntoIterator>::IntoIter,
            fn(<V::FieldLayouts as IntoIterator>::Item) -> Layout,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            key_layout,
            value_layouts,
            ..
        } = self;

        let f: fn(<V::FieldLayouts as IntoIterator>::Item) -> _ = |layout| *layout.borrow();
        let value_layouts = value_layouts.into_iter().map(f);
        iter::once(key_layout).chain(value_layouts)
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
    fn from(value: (*const K, V::Ptrs)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValuePtrs<K, V>> for (*const K, V::Ptrs)
where
    V: Soa,
{
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
        *self
    }
}

impl<K, V> Copy for KeyValuePtrs<K, V> where V: Soa {}

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
    fn from(value: (*mut K, V::MutPtrs)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValueMutPtrs<K, V>> for (*mut K, V::MutPtrs)
where
    V: Soa,
{
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
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for KeyValueMutPtrs<K, V> where V: Soa {}

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
    fn from(value: (NonNull<K>, V::NonNullPtrs)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValueNonNullPtrs<K, V>> for (NonNull<K>, V::NonNullPtrs)
where
    V: Soa,
{
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
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for KeyValueNonNullPtrs<K, V> where V: Soa {}

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
    fn from(value: (Vec<K>, V::Vecs)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<K, V> From<KeyValueVecs<K, V>> for (Vec<K>, V::Vecs)
where
    V: Soa,
{
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
    fn from(value: (*const [K], V::SlicePtrs)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<K, V> From<KeyValueSlicePtrs<K, V>> for (*const [K], V::SlicePtrs)
where
    V: Soa,
{
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
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for KeyValueSlicePtrs<K, V> where V: Soa {}

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
    fn from(value: (*mut [K], V::SliceMutPtrs)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

impl<K, V> From<KeyValueSliceMutPtrs<K, V>> for (*mut [K], V::SliceMutPtrs)
where
    V: Soa,
{
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
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for KeyValueSliceMutPtrs<K, V> where V: Soa {}

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
