use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::replace,
};

use crate::{
    arena::EpochSparseArena,
    assert::{unwrap_dense_value, unwrap_dense_value_mut},
    key::Key,
    set::EpochSparseSet,
};

pub struct OccupiedEntry<'a, K, V, C>
where
    K: Key,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    key: K,
    dense_index: usize,
    container: &'a mut C,
    phantom: PhantomData<&'a mut V>,
}

impl<'a, K, V, C> OccupiedEntry<'a, K, V, C>
where
    K: Key,
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
    pub fn get(&self) -> &V {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let values = container.dense_values();
        unwrap_dense_value(values, *dense_index)
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let values = container.dense_values_mut();
        unwrap_dense_value_mut(values, *dense_index)
    }

    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let values = container.dense_values_mut();
        unwrap_dense_value_mut(values, dense_index)
    }

    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        let previous = self.get_mut();
        replace(previous, value)
    }

    #[inline]
    pub fn key(&self) -> K {
        let Self { key, .. } = self;
        *key
    }

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

    #[inline]
    pub fn replace_key(&mut self, key: K) -> Option<V> {
        let new_key = key;
        let Self { key, container, .. } = self;

        let value = container.remove(*key);
        let value = unwrap_entry_value(value);

        *key = new_key;
        container.insert(*key, value)
    }
}

impl<'a, K, V, C> Debug for OccupiedEntry<'a, K, V, C>
where
    K: Key + Debug,
    V: Debug,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, .. } = self;

        let value = self.get();
        f.debug_struct("OccupiedEntry")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

pub struct VacantEntry<'a, K, V, C>
where
    K: Key,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    key: K,
    container: &'a mut C,
    phantom: PhantomData<&'a mut V>,
}

impl<'a, K, V, C> VacantEntry<'a, K, V, C>
where
    K: Key,
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

    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        let Self { key, container, .. } = self;

        container.insert(key, value);

        let value = container.dense_values_mut().last_mut();
        unwrap_entry_value(value)
    }

    #[inline]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, C> {
        let Self {
            key,
            container,
            phantom,
        } = self;

        container.insert(key, value);
        let dense_index = container.dense_values().len() - 1;

        OccupiedEntry {
            key,
            dense_index,
            container,
            phantom,
        }
    }
}

impl<'a, K, V, C> Debug for VacantEntry<'a, K, V, C>
where
    K: Key + Debug,
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
{
    fn dense_values(&self) -> &[V];

    fn dense_values_mut(&mut self) -> &mut [V];

    fn insert(&mut self, key: K, value: V) -> Option<V>;

    fn remove(&mut self, key: K) -> Option<V>;

    fn swap_remove(&mut self, key: K) -> Option<V>;
}

impl<K, V> EpochSparseContainer<K, V> for EpochSparseSet<K, V>
where
    K: Key,
{
    fn dense_values(&self) -> &[V] {
        self.as_slice()
    }

    fn dense_values_mut(&mut self) -> &mut [V] {
        self.as_mut_slice()
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        EpochSparseSet::insert(self, key, value)
    }

    fn remove(&mut self, key: K) -> Option<V> {
        EpochSparseSet::remove(self, key)
    }

    fn swap_remove(&mut self, key: K) -> Option<V> {
        EpochSparseSet::swap_remove(self, key)
    }
}

impl<K, V> EpochSparseContainer<K, V> for EpochSparseArena<K, V>
where
    K: Key,
{
    fn dense_values(&self) -> &[V] {
        self.as_slice()
    }

    fn dense_values_mut(&mut self) -> &mut [V] {
        self.as_mut_slice()
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        EpochSparseArena::insert(self, key, value)
    }

    fn remove(&mut self, key: K) -> Option<V> {
        EpochSparseArena::remove(self, key)
    }

    fn swap_remove(&mut self, key: K) -> Option<V> {
        EpochSparseArena::swap_remove(self, key)
    }
}

macro_rules! generate_entry_types {
    ($container:ty) => {
        pub enum Entry<'a, K, V>
        where
            K: $crate::key::Key,
        {
            Occupied(OccupiedEntry<'a, K, V>),
            Vacant(VacantEntry<'a, K, V>),
        }

        impl<'a, K, V> Entry<'a, K, V>
        where
            K: $crate::key::Key,
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
            pub fn get(&self) -> Option<&V> {
                match self {
                    Self::Occupied(entry) => Some(entry.get()),
                    Self::Vacant(_) => None,
                }
            }

            #[inline]
            pub fn get_mut(&mut self) -> Option<&mut V> {
                match self {
                    Self::Occupied(entry) => Some(entry.get_mut()),
                    Self::Vacant(_) => None,
                }
            }

            #[inline]
            pub fn and_modify<F>(self, f: F) -> Self
            where
                F: FnOnce(&mut V),
            {
                match self {
                    Self::Occupied(mut entry) => {
                        f(entry.get_mut());
                        Self::Occupied(entry)
                    }
                    Self::Vacant(entry) => Self::Vacant(entry),
                }
            }

            #[inline]
            pub fn or_insert(self, default: V) -> &'a mut V {
                match self {
                    Self::Occupied(entry) => entry.into_mut(),
                    Self::Vacant(entry) => entry.insert(default),
                }
            }

            #[inline]
            pub fn or_insert_with<F>(self, default: F) -> &'a mut V
            where
                F: FnOnce() -> V,
            {
                match self {
                    Self::Occupied(entry) => entry.into_mut(),
                    Self::Vacant(entry) => entry.insert(default()),
                }
            }

            #[inline]
            pub fn or_default(self) -> &'a mut V
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
            pub fn replace_key(self, key: K) -> Self {
                match self {
                    Self::Occupied(mut entry) => {
                        entry.replace_key(key);
                        Self::Occupied(entry)
                    }
                    Self::Vacant(entry) => {
                        let container = entry.into_container();
                        container.entry(key)
                    }
                }
            }
        }

        impl<'a, K, V> core::fmt::Debug for Entry<'a, K, V>
        where
            K: core::fmt::Debug + $crate::key::Key,
            V: core::fmt::Debug,
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
        {
            inner: $crate::entry::OccupiedEntry<'a, K, V, $container>,
        }

        impl<'a, K, V> OccupiedEntry<'a, K, V>
        where
            K: $crate::key::Key,
        {
            #[inline]
            fn new(key: K, dense_index: usize, container: &'a mut $container) -> Self {
                let inner = $crate::entry::OccupiedEntry::new(key, dense_index, container);
                Self { inner }
            }

            #[inline]
            pub fn get(&self) -> &V {
                let Self { inner } = self;
                inner.get()
            }

            #[inline]
            pub fn get_mut(&mut self) -> &mut V {
                let Self { inner } = self;
                inner.get_mut()
            }

            #[inline]
            pub fn into_mut(self) -> &'a mut V {
                let Self { inner } = self;
                inner.into_mut()
            }

            #[inline]
            pub fn insert(&mut self, value: V) -> V {
                let Self { inner } = self;
                inner.insert(value)
            }

            #[inline]
            pub fn key(&self) -> K {
                let Self { inner } = self;
                inner.key()
            }

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

            #[inline]
            pub fn replace_key(&mut self, key: K) -> Option<V> {
                let Self { inner } = self;
                inner.replace_key(key)
            }
        }

        impl<'a, K, V> core::fmt::Debug for OccupiedEntry<'a, K, V>
        where
            K: core::fmt::Debug + $crate::key::Key,
            V: core::fmt::Debug,
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
        {
            inner: $crate::entry::VacantEntry<'a, K, V, $container>,
        }

        impl<'a, K, V> VacantEntry<'a, K, V>
        where
            K: $crate::key::Key,
        {
            #[inline]
            fn new(key: K, container: &'a mut $container) -> Self {
                let inner = $crate::entry::VacantEntry::new(key, container);
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

            #[inline]
            pub fn insert(self, value: V) -> &'a mut V {
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
            K: core::fmt::Debug + $crate::key::Key,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let Self { inner } = self;
                inner.fmt(f)
            }
        }
    };
}

pub(crate) use generate_entry_types;
