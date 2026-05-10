use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use gpecs_soa::{
    traits::{SlicesMut, Soa, SoaContext, SoaOwned},
    wrapper,
};

use crate::{KeyValueMutSlicePtrs, KeyValueSlicePtrs, KeyValueSlices};

pub struct KeyValueMutSlices<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    keys: &'a mut [K],
    values: wrapper::SlicesMut<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V> KeyValueMutSlices<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn new(
        context: &'ctx V::Context,
        keys: &'a mut [K],
        values: SlicesMut<'ctx, 'a, V>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = context.mut_slices_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: &'a mut [K], values: SlicesMut<'ctx, 'a, V>) -> Self {
        let values = wrapper::SlicesMut::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        context.mut_slices_len(values.as_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a mut [K], SlicesMut<'ctx, 'a, V>) {
        let Self { keys, values } = self;
        (keys, values.into_inner())
    }

    #[inline]
    pub fn into_slice_ptrs(self, context: &'ctx V::Context) -> KeyValueSlicePtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_ref(keys);
        let values = context.mut_slices_as_slices(values.into_inner());
        let values = context.slices_as_slice_ptrs(values);
        unsafe { KeyValueSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_mut_slice_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutSlicePtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_mut(keys);
        let values = context.mut_slices_as_mut_slice_ptrs(values.into_inner());
        unsafe { KeyValueMutSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_slices(self, context: &'ctx V::Context) -> KeyValueSlices<'ctx, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = &*keys;
        let values = context.mut_slices_as_slices(values.into_inner());
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }
}

impl<'ctx, 'a, K, V> From<KeyValueMutSlices<'ctx, 'a, K, V>>
    for (&'a mut [K], SlicesMut<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn from(value: KeyValueMutSlices<'ctx, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueMutSlices<'_, '_, K, V>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;

        f.debug_struct("KeyValueMutSlices")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Default for KeyValueMutSlices<'_, '_, K, V>
where
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = SlicesMut::<V>::default();
        unsafe { Self::new_unchecked(keys, values) }
    }
}

impl<K, V> PartialEq for KeyValueMutSlices<'_, '_, K, V>
where
    K: PartialEq,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;

        let other = (&other.keys, &other.values);
        (keys, values) == other
    }
}

impl<K, V> Eq for KeyValueMutSlices<'_, '_, K, V>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueMutSlices<'_, '_, K, V>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { keys, values } = self;

        let other = (&other.keys, &other.values);
        (keys, values).partial_cmp(&other)
    }
}

impl<K, V> Ord for KeyValueMutSlices<'_, '_, K, V>
where
    K: Ord,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { keys, values } = self;

        let other = (&other.keys, &other.values);
        (keys, values).cmp(&other)
    }
}

impl<K, V> Hash for KeyValueMutSlices<'_, '_, K, V>
where
    K: Hash,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        (keys, values).hash(state);
    }
}
