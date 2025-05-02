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

#[cfg(feature = "alloc")]
use core_alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::soa::traits::SoaVecs;
use crate::soa::{
    traits::{Soa, SoaToOwned, SoaTrustedFields},
    FieldDescriptor,
};

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
    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_> {
        KeyValueFieldDescriptors::new(context)
    }

    type FieldOffsets<'context> = KeyValueFieldOffsets<'context, K, V>;

    #[inline]
    fn buffer_layout(
        context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
        KeyValueFieldOffsets::new(context, capacity)
    }

    type Ptrs<'context> = KeyValuePtrs<'context, K, V>;
    type MutPtrs<'context> = KeyValueMutPtrs<'context, K, V>;

    type ErasedPtrs<'context> =
        iter::Chain<iter::Once<*const u8>, <V::ErasedPtrs<'context> as IntoIterator>::IntoIter>;
    type ErasedMutPtrs<'context> =
        iter::Chain<iter::Once<*mut u8>, <V::ErasedMutPtrs<'context> as IntoIterator>::IntoIter>;

    #[inline]
    fn ptrs_erase<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::ErasedPtrs<'context> {
        let KeyValuePtrs { key, value } = ptrs;
        iter::once(key.cast()).chain(V::ptrs_erase(context, value))
    }

    #[inline]
    fn ptrs_erase_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::ErasedMutPtrs<'context> {
        let KeyValueMutPtrs { key, value } = ptrs;
        iter::once(key.cast()).chain(V::ptrs_erase_mut(context, value))
    }

    #[inline]
    #[track_caller]
    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs<'_> {
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
    ) -> Self::MutPtrs<'_> {
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
    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs<'_> {
        KeyValueMutPtrs {
            key: ptr::dangling_mut(),
            value: V::ptrs_dangling(context),
        }
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

    type RefsMut<'context, 'a>
        = KeyValueRefsMut<'context, 'a, K, V>
    where
        Self: 'a;

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
    fn mut_refs_as_ptrs<'context>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        let KeyValueRefsMut { key, value } = refs;
        KeyValueMutPtrs {
            key: ptr::from_mut(key),
            value: V::mut_refs_as_ptrs(context, value),
        }
    }

    #[inline]
    fn mut_refs_as_refs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        let KeyValueRefsMut { key, value } = refs;
        KeyValueRefs {
            key: &*key,
            value: V::mut_refs_as_refs(context, value),
        }
    }

    type SlicePtrs<'context> = KeyValueSlicePtrs<'context, K, V>;

    type SliceMutPtrs<'context> = KeyValueSliceMutPtrs<'context, K, V>;

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
    fn slice_ptrs_len_mut(context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
        let KeyValueSliceMutPtrs { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slice_ptrs_len_mut(context, values);
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
    fn mut_slice_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        let KeyValueSliceMutPtrs { keys, values } = slices;
        KeyValueMutPtrs {
            key: keys.cast(), // should be `keys.as_mut_ptr()` but it's unstable
            value: V::mut_slice_ptrs_as_ptrs(context, values),
        }
    }

    type Slices<'context, 'a>
        = KeyValueSlices<'context, 'a, K, V>
    where
        Self: 'a;

    type SlicesMut<'context, 'a>
        = KeyValueSlicesMut<'context, 'a, K, V>
    where
        Self: 'a;

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
    unsafe fn slice_ptrs_to_slices_mut<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicesMut<'context, 'a> {
        let len = Self::slice_ptrs_len_mut(context, &slices);
        let KeyValueSliceMutPtrs { keys, values } = slices;

        KeyValueSlicesMut {
            keys: unsafe { slice::from_raw_parts_mut(keys.cast(), len) },
            values: unsafe { V::slice_ptrs_to_slices_mut(context, values) },
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
    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_, '_>) -> usize {
        let KeyValueSlicesMut { keys, values } = slices;

        let keys_len = keys.len();
        let values_len = V::slices_len_mut(context, values);
        assert_eq!(keys_len, values_len);
        keys_len
    }

    #[inline]
    fn slice_refs_as_slice_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::SlicePtrs<'context> {
        let KeyValueSlices { keys, values } = slices;
        KeyValueSlicePtrs {
            keys: ptr::from_ref(keys),
            values: V::slice_refs_as_slice_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_slice_refs_as_slice_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::SliceMutPtrs<'context> {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueSliceMutPtrs {
            keys: ptr::from_mut(keys),
            values: V::mut_slice_refs_as_slice_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_slices_as_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a> {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueSlices {
            keys: &*keys,
            values: V::mut_slices_as_slices(context, values),
        }
    }

    #[inline]
    fn slice_refs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::Ptrs<'context> {
        let KeyValueSlices { keys, values } = slices;
        KeyValuePtrs {
            key: keys.as_ptr(),
            value: V::slice_refs_as_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_slice_refs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        let KeyValueSlicesMut { keys, values } = slices;
        KeyValueMutPtrs {
            key: keys.as_mut_ptr(),
            value: V::mut_slice_refs_as_ptrs(context, values),
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
#[cfg(feature = "alloc")]
unsafe impl<K, V> SoaVecs for KeyValuePair<K, V>
where
    V: SoaVecs,
{
    type Vecs = KeyValueVecs<K, V>;

    #[inline]
    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs {
        KeyValueVecs {
            keys: Vec::with_capacity(capacity),
            values: V::vecs_with_capacity(context, capacity),
        }
    }

    #[inline]
    fn vecs_as_ptrs<'context>(
        context: &'context Self::Context,
        vecs: &Self::Vecs,
    ) -> Self::Ptrs<'context> {
        let KeyValueVecs { keys, values } = vecs;
        KeyValuePtrs {
            key: keys.as_ptr(),
            value: V::vecs_as_ptrs(context, values),
        }
    }

    #[inline]
    fn mut_vecs_as_ptrs<'context>(
        context: &'context Self::Context,
        vecs: &mut Self::Vecs,
    ) -> Self::MutPtrs<'context> {
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
        Self {
            key: self.key.clone(),
            phantom: self.phantom.clone(),
            value: self.value.clone(),
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

pub struct KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
{
    offset_from_keys: usize,
    keys: PhantomData<fn() -> K>,
    values: V::FieldOffsets<'context>,
}

impl<'context, K, V> KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
{
    pub fn new(
        context: &'context V::Context,
        capacity: usize,
    ) -> Result<(Layout, Self), LayoutError> {
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

impl<'context, K, V> Debug for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
    V::FieldOffsets<'context>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueFieldOffsets")
            .field("offset_from_keys", &self.offset_from_keys)
            .field("values", &self.values)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
    V::FieldOffsets<'context>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.offset_from_keys == other.offset_from_keys
            && self.keys == other.keys
            && self.values == other.values
    }
}

impl<'context, K, V> Eq for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
    V::FieldOffsets<'context>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
    V::FieldOffsets<'context>: PartialOrd,
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

impl<'context, K, V> Ord for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
    V::FieldOffsets<'context>: Ord,
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

impl<'context, K, V> Hash for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
    V::FieldOffsets<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.offset_from_keys.hash(state);
        self.keys.hash(state);
        self.values.hash(state);
    }
}

impl<'context, K, V> Clone for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
    V::FieldOffsets<'context>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            offset_from_keys: self.offset_from_keys.clone(),
            keys: self.keys.clone(),
            values: self.values.clone(),
        }
    }
}

impl<'context, K, V> Copy for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
    V::FieldOffsets<'context>: Copy,
{
}

impl<'context, K, V> IntoIterator for KeyValueFieldOffsets<'context, K, V>
where
    V: Soa,
{
    type Item = usize;

    type IntoIter = iter::Chain<
        iter::Once<usize>,
        iter::Scan<
            <<V as Soa>::FieldOffsets<'context> as IntoIterator>::IntoIter,
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
        f.debug_struct("KeyValuePtrs")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValuePtrs<'context, K, V>
where
    V: Soa,
    V::Ptrs<'context>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
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
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<'context, K, V> Ord for KeyValuePtrs<'context, K, V>
where
    V: Soa,
    V::Ptrs<'context>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<'context, K, V> Hash for KeyValuePtrs<'context, K, V>
where
    V: Soa,
    V::Ptrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

impl<K, V> Clone for KeyValuePtrs<'_, K, V>
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
        f.debug_struct("KeyValueMutPtrs")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueMutPtrs<'context, K, V>
where
    V: Soa,
    V::MutPtrs<'context>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
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
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<'context, K, V> Ord for KeyValueMutPtrs<'context, K, V>
where
    V: Soa,
    V::MutPtrs<'context>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<'context, K, V> Hash for KeyValueMutPtrs<'context, K, V>
where
    V: Soa,
    V::MutPtrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

impl<K, V> Clone for KeyValueMutPtrs<'_, K, V>
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
        f.debug_struct("KeyValueNonNullPtrs")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
    V::NonNullPtrs<'context>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
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
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<'context, K, V> Ord for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
    V::NonNullPtrs<'context>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<'context, K, V> Hash for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
    V::NonNullPtrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

impl<K, V> Clone for KeyValueNonNullPtrs<'_, K, V>
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

impl<'context, K, V> Copy for KeyValueNonNullPtrs<'context, K, V>
where
    V: Soa,
    V::NonNullPtrs<'context>: Copy,
{
}

#[cfg(feature = "alloc")]
pub struct KeyValueVecs<K, V>
where
    V: SoaVecs,
{
    pub keys: Vec<K>,
    pub values: V::Vecs,
}

#[cfg(feature = "alloc")]
impl<K, V> From<(Vec<K>, V::Vecs)> for KeyValueVecs<K, V>
where
    V: SoaVecs,
{
    #[inline]
    fn from(value: (Vec<K>, V::Vecs)) -> Self {
        let (keys, values) = value;
        Self { keys, values }
    }
}

#[cfg(feature = "alloc")]
impl<K, V> From<KeyValueVecs<K, V>> for (Vec<K>, V::Vecs)
where
    V: SoaVecs,
{
    #[inline]
    fn from(value: KeyValueVecs<K, V>) -> Self {
        let KeyValueVecs { keys, values } = value;
        (keys, values)
    }
}

#[cfg(feature = "alloc")]
impl<K, V> Debug for KeyValueVecs<K, V>
where
    K: Debug,
    V: SoaVecs,
    V::Vecs: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyValueVecs")
            .field("keys", &self.keys)
            .field("values", &self.values)
            .finish()
    }
}

#[cfg(feature = "alloc")]
impl<K, V> Default for KeyValueVecs<K, V>
where
    V: SoaVecs,
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

#[cfg(feature = "alloc")]
impl<K, V> PartialEq for KeyValueVecs<K, V>
where
    K: PartialEq,
    V: SoaVecs,
    V::Vecs: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.values == other.values
    }
}

#[cfg(feature = "alloc")]
impl<K, V> Eq for KeyValueVecs<K, V>
where
    K: Eq,
    V: SoaVecs,
    V::Vecs: Eq,
{
}

#[cfg(feature = "alloc")]
impl<K, V> PartialOrd for KeyValueVecs<K, V>
where
    K: PartialOrd,
    V: SoaVecs,
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

#[cfg(feature = "alloc")]
impl<K, V> Ord for KeyValueVecs<K, V>
where
    K: Ord,
    V: SoaVecs,
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

#[cfg(feature = "alloc")]
impl<K, V> Hash for KeyValueVecs<K, V>
where
    K: Hash,
    V: SoaVecs,
    V::Vecs: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}

#[cfg(feature = "alloc")]
impl<K, V> Clone for KeyValueVecs<K, V>
where
    K: Clone,
    V: SoaVecs,
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
        f.debug_struct("KeyValueRefs")
            .field("key", &self.key)
            .field("value", &self.value)
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
        self.key == other.key && self.value == other.value
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
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<'context, 'a, K, V> Ord for KeyValueRefs<'context, 'a, K, V>
where
    K: Ord + 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<'context, 'a, K, V> Hash for KeyValueRefs<'context, 'a, K, V>
where
    K: Hash + 'a,
    V: Soa + 'a,
    V::Refs<'context, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
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
        Self {
            key: self.key,
            value: self.value.clone(),
        }
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
        f.debug_struct("KeyValueRefsMut")
            .field("key", &self.key)
            .field("value", &self.value)
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
        self.key == other.key && self.value == other.value
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
        match self.key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl<'context, 'a, K, V> Ord for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Ord + 'a,
    V: Soa + 'a,
    V::RefsMut<'context, 'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl<'context, 'a, K, V> Hash for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Hash + 'a,
    V: Soa + 'a,
    V::RefsMut<'context, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
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
        f.debug_struct("KeyValueSlicePtrs")
            .field("keys", &self.keys)
            .field("values", &self.values)
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
        self.keys == other.keys && self.values == other.values
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
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<'context, K, V> Ord for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa,
    V::SlicePtrs<'context>: Ord,
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

impl<'context, K, V> Hash for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa,
    V::SlicePtrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSlicePtrs<'_, K, V>
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
        f.debug_struct("KeyValueSliceMutPtrs")
            .field("keys", &self.keys)
            .field("values", &self.values)
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
        self.keys == other.keys && self.values == other.values
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
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<'context, K, V> Ord for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa,
    V::SliceMutPtrs<'context>: Ord,
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

impl<'context, K, V> Hash for KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa,
    V::SliceMutPtrs<'context>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSliceMutPtrs<'_, K, V>
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
        f.debug_struct("KeyValueSlices")
            .field("keys", &self.keys)
            .field("values", &self.values)
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
        self.keys == other.keys && self.values == other.values
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
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<'context, 'a, K, V> Ord for KeyValueSlices<'context, 'a, K, V>
where
    K: Ord + 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.values.cmp(&other.values)
    }
}

impl<'context, 'a, K, V> Hash for KeyValueSlices<'context, 'a, K, V>
where
    K: Hash + 'a,
    V: Soa + 'a,
    V::Slices<'context, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
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
        Self {
            keys: self.keys,
            values: self.values.clone(),
        }
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
        f.debug_struct("KeyValueSlicesMut")
            .field("keys", &self.keys)
            .field("values", &self.values)
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
        self.keys == other.keys && self.values == other.values
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
        match self.keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.values.partial_cmp(&other.values)
    }
}

impl<'context, 'a, K, V> Ord for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Ord + 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.values.cmp(&other.values)
    }
}

impl<'context, 'a, K, V> Hash for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Hash + 'a,
    V: Soa + 'a,
    V::SlicesMut<'context, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
        self.values.hash(state);
    }
}
