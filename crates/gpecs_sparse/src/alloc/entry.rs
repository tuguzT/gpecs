use core::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use crate::{
    arena::EpochSparseArena,
    assert::unwrap_dense,
    error::TryModifyError,
    key::Key,
    set::EpochSparseSet,
    soa::{
        mem::replace as soa_replace,
        slice::{SoaSlices, SoaSlicesMut},
        traits::{Soa, SoaRead, SoaWrite},
    },
};

use super::assert::try_replace_key_failed;

pub struct OccupiedEntry<'a, K, V, C>
where
    K: Key,
    V: Soa + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    key: K,
    dense_index: usize,
    container: &'a mut C,
    phantom: PhantomData<&'a mut V>,
}

impl<'a, K, V, C> OccupiedEntry<'a, K, V, C>
where
    K: Key + 'a,
    V: Soa + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    #[inline]
    pub(crate) fn new(key: K, dense_index: usize, container: &'a mut C) -> Self {
        Self {
            key,
            dense_index,
            container,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> V::Refs<'_, '_> {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let values = container.slices();
        unwrap_dense(values, *dense_index)
    }

    #[inline]
    pub fn get_mut(&mut self) -> V::RefsMut<'_, '_> {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let values = container.slices_mut();
        unwrap_dense(values, *dense_index)
    }

    #[inline]
    pub fn into_mut(self) -> V::RefsMut<'a, 'a> {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let values = container.slices_mut();
        unwrap_dense(values, dense_index)
    }

    #[inline]
    pub fn key(&self) -> K {
        let Self { key, .. } = self;
        *key
    }
}

impl<'a, K, V, C> OccupiedEntry<'a, K, V, C>
where
    K: Key + 'a,
    V: SoaRead,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    #[inline]
    pub fn remove(self) -> V {
        let Self { key, container, .. } = self;

        let value = container.remove(key);
        unwrap_entry_value(value)
    }

    #[inline]
    pub fn swap_remove(self) -> V {
        let Self { key, container, .. } = self;

        let value = container.swap_remove(key);
        unwrap_entry_value(value)
    }
}

impl<'a, K, V, C> OccupiedEntry<'a, K, V, C>
where
    K: Key + 'a,
    V: SoaRead + SoaWrite,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let (context, values) = container.slices_mut().into_slices_with_context();
        let values = SoaSlicesMut::<V>::new(context, values);
        let previous = unwrap_dense(values, *dense_index);
        soa_replace(context, previous, value)
    }

    #[inline]
    #[track_caller]
    pub fn replace_key(&mut self, key: K) -> Option<V> {
        self.try_replace_key(key)
            .unwrap_or_else(|error| try_replace_key_failed(error.kind))
    }

    #[inline]
    pub fn try_replace_key(&mut self, key: K) -> Result<Option<V>, TryModifyError<K, V>> {
        let new_key = key;
        let Self { key, container, .. } = self;

        let value = container.remove(*key);
        let value = unwrap_entry_value(value);

        *key = new_key;
        container.try_insert(*key, value)
    }
}

impl<K, V, C> Debug for OccupiedEntry<'_, K, V, C>
where
    K: Key + Debug,
    V: Soa + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
    for<'c, 'any> V::Refs<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, .. } = self;

        let value = &self.get();
        f.debug_struct("OccupiedEntry")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

pub struct VacantEntry<'a, K, V, C>
where
    K: Key,
    V: Soa + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    key: K,
    container: &'a mut C,
    phantom: PhantomData<&'a mut V>,
}

impl<'a, K, V, C> VacantEntry<'a, K, V, C>
where
    K: Key + 'a,
    V: Soa + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    #[inline]
    pub(crate) fn new(key: K, container: &'a mut C) -> Self {
        Self {
            key,
            container,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn into_container(self) -> &'a mut C {
        let Self { container, .. } = self;
        container
    }

    #[inline]
    pub fn key(&self) -> K {
        let Self { key, .. } = self;
        *key
    }
}

impl<'a, K, V, C> VacantEntry<'a, K, V, C>
where
    K: Key + 'a,
    V: SoaRead + SoaWrite,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    #[inline]
    pub fn insert(self, value: V) -> V::RefsMut<'a, 'a> {
        let Self { key, container, .. } = self;

        if container.try_insert(key, value).is_err() {
            unreachable!()
        }

        let value = container.slices_mut().into_iter().last();
        unwrap_entry_value(value)
    }

    #[inline]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, C> {
        let Self {
            key,
            container,
            phantom,
        } = self;

        if container.try_insert(key, value).is_err() {
            unreachable!()
        }
        let dense_index = container.slices().len() - 1;

        OccupiedEntry {
            key,
            dense_index,
            container,
            phantom,
        }
    }
}

impl<K, V, C> Debug for VacantEntry<'_, K, V, C>
where
    K: Key + Debug,
    V: Soa + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, .. } = self;
        f.debug_struct("VacantEntry").field("key", key).finish()
    }
}

#[cold]
#[track_caller]
#[inline(never)]
const fn unwrap_entry_value_failed() -> ! {
    panic!("value by provided key should exist")
}

#[inline]
#[track_caller]
fn unwrap_entry_value<T>(value: Option<T>) -> T {
    let Some(value) = value else {
        unwrap_entry_value_failed()
    };
    value
}

pub trait EpochSparseContainer<K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    fn slices(&self) -> SoaSlices<'_, '_, V>;

    fn slices_mut(&mut self) -> SoaSlicesMut<'_, '_, V>;

    fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryModifyError<K, V>>
    where
        V: SoaRead + SoaWrite;

    fn remove(&mut self, key: K) -> Option<V>
    where
        V: SoaRead;

    fn swap_remove(&mut self, key: K) -> Option<V>
    where
        V: SoaRead;
}

impl<K, V> EpochSparseContainer<K, V> for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn slices(&self) -> SoaSlices<'_, '_, V> {
        Self::slices(self)
    }

    #[inline]
    fn slices_mut(&mut self) -> SoaSlicesMut<'_, '_, V> {
        Self::slices_mut(self)
    }

    #[inline]
    fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryModifyError<K, V>>
    where
        V: SoaRead + SoaWrite,
    {
        Self::try_insert(self, key, value)
    }

    #[inline]
    fn remove(&mut self, key: K) -> Option<V>
    where
        V: SoaRead,
    {
        Self::remove(self, key)
    }

    #[inline]
    fn swap_remove(&mut self, key: K) -> Option<V>
    where
        V: SoaRead,
    {
        Self::swap_remove(self, key)
    }
}

impl<K, V> EpochSparseContainer<K, V> for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn slices(&self) -> SoaSlices<'_, '_, V> {
        Self::slices(self)
    }

    #[inline]
    fn slices_mut(&mut self) -> SoaSlicesMut<'_, '_, V> {
        Self::slices_mut(self)
    }

    #[inline]
    fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryModifyError<K, V>>
    where
        V: SoaRead + SoaWrite,
    {
        Self::try_insert(self, key, value)
    }

    #[inline]
    fn remove(&mut self, key: K) -> Option<V>
    where
        V: SoaRead,
    {
        Self::remove(self, key)
    }

    #[inline]
    fn swap_remove(&mut self, key: K) -> Option<V>
    where
        V: SoaRead,
    {
        Self::swap_remove(self, key)
    }
}

macro_rules! generate_entry_types {
    ($container:ty) => {
        pub enum Entry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::Soa + ?Sized,
        {
            Occupied(OccupiedEntry<'a, K, V>),
            Vacant(VacantEntry<'a, K, V>),
        }

        impl<'a, K, V> Entry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::Soa + ?Sized,
        {
            #[inline]
            pub const fn is_occupied(&self) -> bool {
                matches!(self, Self::Occupied(_))
            }

            #[inline]
            pub const fn is_vacant(&self) -> bool {
                matches!(self, Self::Vacant(_))
            }

            #[inline]
            pub fn key(&self) -> K {
                match self {
                    Self::Occupied(entry) => entry.key(),
                    Self::Vacant(entry) => entry.key(),
                }
            }

            #[inline]
            pub fn get(&self) -> Option<V::Refs<'_, '_>> {
                match self {
                    Self::Occupied(entry) => Some(entry.get()),
                    Self::Vacant(_) => None,
                }
            }

            #[inline]
            pub fn get_mut(&mut self) -> Option<V::RefsMut<'_, '_>> {
                match self {
                    Self::Occupied(entry) => Some(entry.get_mut()),
                    Self::Vacant(_) => None,
                }
            }

            #[inline]
            #[must_use]
            pub fn and_modify<F>(self, f: F) -> Self
            where
                F: FnOnce(V::RefsMut<'_, '_>),
            {
                match self {
                    Self::Occupied(mut entry) => {
                        f(entry.get_mut());
                        Self::Occupied(entry)
                    }
                    Self::Vacant(entry) => Self::Vacant(entry),
                }
            }
        }

        impl<'a, K, V> Entry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::SoaRead + $crate::soa::traits::SoaWrite,
        {
            #[inline]
            pub fn or_insert(self, default: V) -> V::RefsMut<'a, 'a> {
                match self {
                    Self::Occupied(entry) => entry.into_mut(),
                    Self::Vacant(entry) => entry.insert(default),
                }
            }

            #[inline]
            pub fn or_insert_with<F>(self, default: F) -> V::RefsMut<'a, 'a>
            where
                F: FnOnce() -> V,
            {
                match self {
                    Self::Occupied(entry) => entry.into_mut(),
                    Self::Vacant(entry) => entry.insert(default()),
                }
            }

            #[inline]
            pub fn or_default(self) -> V::RefsMut<'a, 'a>
            where
                V: Default,
            {
                match self {
                    Self::Occupied(entry) => entry.into_mut(),
                    Self::Vacant(entry) => entry.insert(Default::default()),
                }
            }

            #[inline]
            pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
                match self {
                    Self::Occupied(mut entry) => {
                        entry.insert(value);
                        entry
                    }
                    Self::Vacant(entry) => entry.insert_entry(value),
                }
            }

            #[inline]
            #[must_use]
            #[track_caller]
            pub fn replace_key(self, key: K) -> Self {
                self.try_replace_key(key)
                    .unwrap_or_else(|error| $crate::alloc::assert::try_replace_key_failed(error))
            }

            #[inline]
            pub fn try_replace_key(self, key: K) -> Result<Self, TryModifyErrorKind<K>> {
                match self {
                    Self::Occupied(mut entry) => {
                        entry.try_replace_key(key).map_err(|error| error.kind)?;
                        Ok(Self::Occupied(entry))
                    }
                    Self::Vacant(entry) => {
                        let container = entry.into_container();
                        Ok(container.try_entry(key)?)
                    }
                }
            }
        }

        impl<K, V> core::fmt::Debug for Entry<'_, K, V>
        where
            K: $crate::key::Key + core::fmt::Debug,
            V: $crate::soa::traits::Soa + core::fmt::Debug + ?Sized,
            for<'c, 'any> V::Refs<'c, 'any>: Debug,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Self::Occupied(entry) => f.debug_tuple("Occupied").field(entry).finish(),
                    Self::Vacant(entry) => f.debug_tuple("Vacant").field(entry).finish(),
                }
            }
        }

        #[repr(transparent)]
        pub struct OccupiedEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::Soa + ?Sized,
        {
            inner: $crate::alloc::entry::OccupiedEntry<'a, K, V, $container>,
        }

        impl<'a, K, V> OccupiedEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::Soa + ?Sized,
        {
            #[inline]
            fn new(key: K, dense_index: usize, container: &'a mut $container) -> Self {
                let inner = $crate::alloc::entry::OccupiedEntry::new(key, dense_index, container);
                Self { inner }
            }

            #[inline]
            pub fn get(&self) -> V::Refs<'_, '_> {
                let Self { inner } = self;
                inner.get()
            }

            #[inline]
            pub fn get_mut(&mut self) -> V::RefsMut<'_, '_> {
                let Self { inner } = self;
                inner.get_mut()
            }

            #[inline]
            pub fn into_mut(self) -> V::RefsMut<'a, 'a> {
                let Self { inner } = self;
                inner.into_mut()
            }

            #[inline]
            pub fn key(&self) -> K {
                let Self { inner } = self;
                inner.key()
            }
        }

        impl<'a, K, V> OccupiedEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::SoaRead,
        {
            #[inline]
            pub fn remove(self) -> V {
                let Self { inner } = self;
                inner.remove()
            }

            #[inline]
            pub fn swap_remove(self) -> V {
                let Self { inner } = self;
                inner.swap_remove()
            }
        }

        impl<'a, K, V> OccupiedEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::SoaRead + $crate::soa::traits::SoaWrite,
        {
            #[inline]
            pub fn insert(&mut self, value: V) -> V {
                let Self { inner } = self;
                inner.insert(value)
            }

            #[inline]
            #[track_caller]
            pub fn replace_key(&mut self, key: K) -> Option<V> {
                let Self { inner } = self;
                inner.replace_key(key)
            }

            #[inline]
            pub fn try_replace_key(&mut self, key: K) -> Result<Option<V>, TryModifyError<K, V>> {
                let Self { inner } = self;
                inner.try_replace_key(key)
            }
        }

        impl<K, V> core::fmt::Debug for OccupiedEntry<'_, K, V>
        where
            K: $crate::key::Key + core::fmt::Debug,
            V: $crate::soa::traits::Soa + core::fmt::Debug + ?Sized,
            for<'c, 'any> V::Refs<'c, 'any>: core::fmt::Debug,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let Self { inner } = self;
                inner.fmt(f)
            }
        }

        #[repr(transparent)]
        pub struct VacantEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::Soa + ?Sized,
        {
            inner: $crate::alloc::entry::VacantEntry<'a, K, V, $container>,
        }

        impl<'a, K, V> VacantEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::Soa + ?Sized,
        {
            #[inline]
            fn new(key: K, container: &'a mut $container) -> Self {
                let inner = $crate::alloc::entry::VacantEntry::new(key, container);
                Self { inner }
            }

            #[inline]
            fn into_container(self) -> &'a mut $container {
                let Self { inner } = self;
                inner.into_container()
            }

            #[inline]
            pub fn key(&self) -> K {
                let Self { inner } = self;
                inner.key()
            }
        }

        impl<'a, K, V> VacantEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::SoaRead + $crate::soa::traits::SoaWrite,
        {
            #[inline]
            pub fn insert(self, value: V) -> V::RefsMut<'a, 'a> {
                let Self { inner } = self;
                inner.insert(value)
            }

            #[inline]
            pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
                let inner = self.inner.insert_entry(value);
                OccupiedEntry { inner }
            }
        }

        impl<'a, K, V> core::fmt::Debug for VacantEntry<'a, K, V>
        where
            K: $crate::key::Key + core::fmt::Debug,
            V: $crate::soa::traits::Soa + ?Sized,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let Self { inner } = self;
                inner.fmt(f)
            }
        }
    };
}

pub(super) use generate_entry_types;
