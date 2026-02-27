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
        self,
        slice::{SoaSliceMutPtrs, SoaSlicePtrs, SoaSlices, SoaSlicesMut},
        traits::{
            AllocSoa, MutPtrs, Ptrs, RawSoa, Refs, RefsMut, Soa, SoaContext, SoaRead, SoaWrite,
        },
    },
};

use super::assert::try_replace_key_failed;

pub struct OccupiedEntry<'a, K, V, C>
where
    K: Key + 'a,
    V: AllocSoa + ?Sized + 'a,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    key: K,
    dense_index: usize,
    container: &'a mut C,
    phantom: PhantomData<fn() -> V>,
}

impl<'a, K, V, C> OccupiedEntry<'a, K, V, C>
where
    K: Key,
    V: AllocSoa + ?Sized,
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
    pub fn context(&self) -> &V::Context {
        let Self { container, .. } = self;
        container.context()
    }

    #[inline]
    pub fn key(&self) -> K {
        let Self { key, .. } = self;
        *key
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, V> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, Ptrs<'_, V>) {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let (context, values) = container.slices().into_raw_iter_with_context();
        let ptrs = unwrap_dense(values, *dense_index);
        (context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, V> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&V::Context, MutPtrs<'_, V>) {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let (context, values) = container.mut_slices().into_raw_iter_mut_with_context();
        let ptrs = unwrap_dense(values, *dense_index);
        (context, ptrs)
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'a, V> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'a V::Context, Ptrs<'a, V>) {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let (context, values) = container.mut_slices().into_raw_iter_with_context();
        let ptrs = unwrap_dense(values, dense_index);
        (context, ptrs)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> MutPtrs<'a, V> {
        let (_, ptrs) = self.into_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'a V::Context, MutPtrs<'a, V>) {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let (context, values) = container.mut_slices().into_raw_iter_mut_with_context();
        let ptrs = unwrap_dense(values, dense_index);
        (context, ptrs)
    }

    #[inline]
    pub fn remove<R>(self) -> R
    where
        V: SoaRead<R>,
    {
        let Self { key, container, .. } = self;

        let value = container.remove(key);
        unwrap_entry_value(value)
    }

    #[inline]
    pub fn swap_remove<R>(self) -> R
    where
        V: SoaRead<R>,
    {
        let Self { key, container, .. } = self;

        let value = container.swap_remove(key);
        unwrap_entry_value(value)
    }

    #[inline]
    pub fn insert<R>(&mut self, value: V) -> R
    where
        V: SoaRead<R> + SoaWrite,
    {
        let Self {
            dense_index,
            container,
            ..
        } = self;

        let (context, values) = container.mut_slices().into_raw_iter_mut_with_context();
        let previous = unwrap_dense(values, *dense_index);
        unsafe { soa::ptr::replace(context, previous, value) }
    }

    #[inline]
    pub fn try_replace_key(&mut self, key: K) -> Result<Option<V>, TryModifyError<K, V>>
    where
        V: SoaRead<V> + SoaWrite,
    {
        let new_key = key;
        let Self { key, container, .. } = self;

        let value = container.remove(*key);
        let value = unwrap_entry_value(value);

        *key = new_key;
        container.try_insert(*key, value)
    }

    #[inline]
    #[track_caller]
    pub fn replace_key(&mut self, key: K) -> Option<V>
    where
        V: SoaRead<V> + SoaWrite,
    {
        self.try_replace_key(key)
            .unwrap_or_else(|error| try_replace_key_failed(error.kind))
    }
}

impl<'a, K, V, C> OccupiedEntry<'a, K, V, C>
where
    K: Key,
    V: AllocSoa + Soa<'a> + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    #[inline]
    pub fn into_mut(self) -> RefsMut<'a, 'a, V> {
        let (_, refs) = self.into_mut_with_context();
        refs
    }

    #[inline]
    pub fn into_mut_with_context(self) -> (&'a V::Context, RefsMut<'a, 'a, V>) {
        let (context, ptrs) = self.into_mut_ptrs_with_context();
        let refs = unsafe { context.mut_ptrs_to_mut_refs(ptrs) };
        (context, refs)
    }
}

impl<'a, K, V, C> OccupiedEntry<'_, K, V, C>
where
    K: Key,
    V: AllocSoa + Soa<'a> + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    #[inline]
    pub fn get(&'a self) -> Refs<'a, 'a, V> {
        let (_, refs) = self.get_with_context();
        refs
    }

    #[inline]
    pub fn get_with_context(&'a self) -> (&'a V::Context, Refs<'a, 'a, V>) {
        let (context, ptrs) = self.as_ptrs_with_context();
        let refs = unsafe { context.ptrs_to_refs(ptrs) };
        (context, refs)
    }

    #[inline]
    pub fn get_mut(&'a mut self) -> RefsMut<'a, 'a, V> {
        let (_, refs) = self.get_mut_with_context();
        refs
    }

    #[inline]
    pub fn get_mut_with_context(&'a mut self) -> (&'a V::Context, RefsMut<'a, 'a, V>) {
        let (context, ptrs) = self.as_mut_ptrs_with_context();
        let refs = unsafe { context.mut_ptrs_to_mut_refs(ptrs) };
        (context, refs)
    }
}

impl<K, V, C> Debug for OccupiedEntry<'_, K, V, C>
where
    K: Key + Debug,
    V: AllocSoa + ?Sized,
    C: EpochSparseContainer<K, V> + ?Sized,
    for<'ctx, 'a> V: Soa<'a, Context: SoaContext<'a, Refs<'ctx>: Debug>>,
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
    K: Key + 'a,
    V: AllocSoa + ?Sized + 'a,
    C: EpochSparseContainer<K, V> + ?Sized,
{
    key: K,
    container: &'a mut C,
    phantom: PhantomData<fn() -> V>,
}

impl<'a, K, V, C> VacantEntry<'a, K, V, C>
where
    K: Key,
    V: AllocSoa + ?Sized,
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
    pub fn context(&self) -> &V::Context {
        let Self { container, .. } = self;
        container.context()
    }

    #[inline]
    pub fn key(&self) -> K {
        let Self { key, .. } = self;
        *key
    }

    #[inline]
    pub fn insert<R>(self, value: V) -> OccupiedEntry<'a, K, V, C>
    where
        V: SoaRead<R> + SoaWrite,
    {
        let Self { key, container, .. } = self;

        if container.try_insert(key, value).is_err() {
            unreachable!()
        }

        let dense_index = container.slices().len() - 1;
        OccupiedEntry::new(key, dense_index, container)
    }
}

impl<K, V, C> Debug for VacantEntry<'_, K, V, C>
where
    K: Key + Debug,
    V: AllocSoa + ?Sized,
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
    V: AllocSoa + ?Sized,
{
    fn context(&self) -> &V::Context;

    fn slices(&self) -> SoaSlices<'_, '_, V>;

    fn mut_slices(&mut self) -> SoaSlicesMut<'_, '_, V>;

    fn try_insert<R>(&mut self, key: K, value: V) -> Result<Option<R>, TryModifyError<K, V>>
    where
        V: SoaRead<R> + SoaWrite;

    fn remove<R>(&mut self, key: K) -> Option<R>
    where
        V: SoaRead<R>;

    fn swap_remove<R>(&mut self, key: K) -> Option<R>
    where
        V: SoaRead<R>;
}

impl<K, V> EpochSparseContainer<K, V> for EpochSparseSet<K, V>
where
    K: Key,
    V: AllocSoa + ?Sized,
{
    #[inline]
    fn context(&self) -> &<V as RawSoa>::Context {
        Self::context(self)
    }

    #[inline]
    fn slices(&self) -> SoaSlices<'_, '_, V> {
        let (dense, _) = self.as_view().into_parts();
        let (context, slices) = dense.into_slice_ptrs_with_context();
        let (_, values) = slices.into_parts();
        unsafe { SoaSlicePtrs::new(context.as_inner(), values).deref() }
    }

    #[inline]
    fn mut_slices(&mut self) -> SoaSlicesMut<'_, '_, V> {
        let (dense, _) = self.as_mut_view().into_parts();
        let (context, slices) = dense
            .into_mut_slice_ptrs()
            .into_mut_slice_ptrs_with_context();
        let (_, values) = slices.into_parts();
        unsafe { SoaSliceMutPtrs::new(context.as_inner(), values).deref_mut() }
    }

    #[inline]
    fn try_insert<R>(&mut self, key: K, value: V) -> Result<Option<R>, TryModifyError<K, V>>
    where
        V: SoaRead<R> + SoaWrite,
    {
        Self::try_insert(self, key, value)
    }

    #[inline]
    fn remove<R>(&mut self, key: K) -> Option<R>
    where
        V: SoaRead<R>,
    {
        Self::remove(self, key)
    }

    #[inline]
    fn swap_remove<R>(&mut self, key: K) -> Option<R>
    where
        V: SoaRead<R>,
    {
        Self::swap_remove(self, key)
    }
}

impl<K, V> EpochSparseContainer<K, V> for EpochSparseArena<K, V>
where
    K: Key,
    V: AllocSoa + ?Sized,
{
    #[inline]
    fn context(&self) -> &<V as RawSoa>::Context {
        Self::context(self)
    }

    #[inline]
    fn slices(&self) -> SoaSlices<'_, '_, V> {
        let (dense, _) = self.as_view().into_parts();
        let (context, slices) = dense.into_slice_ptrs_with_context();
        let (_, values) = slices.into_parts();
        unsafe { SoaSlicePtrs::new(context.as_inner(), values).deref() }
    }

    #[inline]
    fn mut_slices(&mut self) -> SoaSlicesMut<'_, '_, V> {
        let (dense, _) = self.as_mut_view().into_parts();
        let (context, slices) = dense
            .into_mut_slice_ptrs()
            .into_mut_slice_ptrs_with_context();
        let (_, values) = slices.into_parts();
        unsafe { SoaSliceMutPtrs::new(context.as_inner(), values).deref_mut() }
    }

    #[inline]
    fn try_insert<R>(&mut self, key: K, value: V) -> Result<Option<R>, TryModifyError<K, V>>
    where
        V: SoaRead<R> + SoaWrite,
    {
        Self::try_insert(self, key, value)
    }

    #[inline]
    fn remove<R>(&mut self, key: K) -> Option<R>
    where
        V: SoaRead<R>,
    {
        Self::remove(self, key)
    }

    #[inline]
    fn swap_remove<R>(&mut self, key: K) -> Option<R>
    where
        V: SoaRead<R>,
    {
        Self::swap_remove(self, key)
    }
}

macro_rules! generate_entry_types {
    ($container:ty) => {
        pub enum Entry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::AllocSoa + ?Sized,
        {
            Occupied(OccupiedEntry<'a, K, V>),
            Vacant(VacantEntry<'a, K, V>),
        }

        impl<'a, K, V> Entry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::AllocSoa + ?Sized,
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
            pub fn context(&self) -> &V::Context {
                match self {
                    Self::Occupied(entry) => entry.context(),
                    Self::Vacant(entry) => entry.context(),
                }
            }

            #[inline]
            pub fn key(&self) -> K {
                match self {
                    Self::Occupied(entry) => entry.key(),
                    Self::Vacant(entry) => entry.key(),
                }
            }

            #[inline]
            pub fn as_ptrs(&self) -> Option<Ptrs<'_, V>> {
                let (_, ptrs) = self.as_ptrs_with_context();
                ptrs
            }

            #[inline]
            pub fn as_ptrs_with_context(&self) -> (&V::Context, Option<Ptrs<'_, V>>) {
                match self {
                    Self::Occupied(entry) => {
                        let (context, ptrs) = entry.as_ptrs_with_context();
                        (context, Some(ptrs))
                    }
                    Self::Vacant(entry) => (entry.context(), None),
                }
            }

            #[inline]
            pub fn as_mut_ptrs(&mut self) -> Option<MutPtrs<'_, V>> {
                let (_, ptrs) = self.as_mut_ptrs_with_context();
                ptrs
            }

            #[inline]
            pub fn as_mut_ptrs_with_context(&mut self) -> (&V::Context, Option<MutPtrs<'_, V>>) {
                match self {
                    Self::Occupied(entry) => {
                        let (context, ptrs) = entry.as_mut_ptrs_with_context();
                        (context, Some(ptrs))
                    }
                    Self::Vacant(entry) => (entry.context(), None),
                }
            }

            #[inline]
            pub fn or_insert<R>(self, default: V) -> OccupiedEntry<'a, K, V>
            where
                V: SoaRead<R> + SoaWrite,
            {
                match self {
                    Self::Occupied(entry) => entry,
                    Self::Vacant(entry) => entry.insert(default),
                }
            }

            #[inline]
            pub fn or_insert_with<R, F>(self, default: F) -> OccupiedEntry<'a, K, V>
            where
                V: SoaRead<R> + SoaWrite,
                F: FnOnce() -> V,
            {
                match self {
                    Self::Occupied(entry) => entry,
                    Self::Vacant(entry) => entry.insert(default()),
                }
            }

            #[inline]
            pub fn or_default<R>(self) -> OccupiedEntry<'a, K, V>
            where
                V: SoaRead<R> + SoaWrite + Default,
            {
                match self {
                    Self::Occupied(entry) => entry,
                    Self::Vacant(entry) => entry.insert(Default::default()),
                }
            }

            #[inline]
            pub fn insert<R>(self, value: V) -> OccupiedEntry<'a, K, V>
            where
                V: SoaRead<R> + SoaWrite,
            {
                match self {
                    Self::Occupied(mut entry) => {
                        entry.insert(value);
                        entry
                    }
                    Self::Vacant(entry) => entry.insert(value),
                }
            }

            #[inline]
            #[must_use]
            #[track_caller]
            pub fn replace_key(self, key: K) -> Self
            where
                V: SoaRead<V> + SoaWrite,
            {
                self.try_replace_key(key)
                    .unwrap_or_else(|error| $crate::alloc::assert::try_replace_key_failed(error))
            }

            #[inline]
            pub fn try_replace_key(self, key: K) -> Result<Self, TryModifyErrorKind<K>>
            where
                V: SoaRead<V> + SoaWrite,
            {
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

        impl<K, V> Entry<'_, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::AllocSoa + $crate::soa::traits::SoaOwned + ?Sized,
        {
            #[inline]
            #[must_use]
            pub fn and_modify<F>(self, f: F) -> Self
            where
                F: FnOnce(&V::Context, $crate::soa::traits::RefsMut<'_, '_, V>),
            {
                match self {
                    Self::Occupied(mut entry) => {
                        let (context, refs) = entry.get_mut_with_context();
                        f(context, refs);
                        Self::Occupied(entry)
                    }
                    Self::Vacant(entry) => Self::Vacant(entry),
                }
            }
        }

        impl<'a, K, V> Entry<'_, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::AllocSoa + $crate::soa::traits::Soa<'a> + ?Sized,
        {
            #[inline]
            pub fn get(&'a self) -> Option<$crate::soa::traits::Refs<'a, 'a, V>> {
                let (_, refs) = self.get_with_context();
                refs
            }

            #[inline]
            pub fn get_with_context(
                &'a self,
            ) -> (&'a V::Context, Option<$crate::soa::traits::Refs<'a, 'a, V>>) {
                match self {
                    Self::Occupied(entry) => {
                        let (context, refs) = entry.get_with_context();
                        (context, Some(refs))
                    }
                    Self::Vacant(entry) => (entry.context(), None),
                }
            }

            #[inline]
            pub fn get_mut(&'a mut self) -> Option<$crate::soa::traits::RefsMut<'a, 'a, V>> {
                let (_, refs) = self.get_mut_with_context();
                refs
            }

            #[inline]
            pub fn get_mut_with_context(
                &'a mut self,
            ) -> (
                &'a V::Context,
                Option<$crate::soa::traits::RefsMut<'a, 'a, V>>,
            ) {
                match self {
                    Self::Occupied(entry) => {
                        let (context, refs) = entry.get_mut_with_context();
                        (context, Some(refs))
                    }
                    Self::Vacant(entry) => (entry.context(), None),
                }
            }
        }

        impl<K, V> core::fmt::Debug for Entry<'_, K, V>
        where
            K: $crate::key::Key + core::fmt::Debug,
            V: $crate::soa::traits::AllocSoa + ?Sized,
            for<'ctx, 'a> V: $crate::soa::traits::Soa<
                    'a,
                    Context: $crate::soa::traits::SoaContext<'a, Refs<'ctx>: Debug>,
                >,
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
            V: $crate::soa::traits::AllocSoa + ?Sized,
        {
            inner: $crate::alloc::entry::OccupiedEntry<'a, K, V, $container>,
        }

        impl<'a, K, V> OccupiedEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::AllocSoa + ?Sized,
        {
            #[inline]
            fn new(key: K, dense_index: usize, container: &'a mut $container) -> Self {
                let inner = $crate::alloc::entry::OccupiedEntry::new(key, dense_index, container);
                Self { inner }
            }

            #[inline]
            pub fn context(&self) -> &V::Context {
                let Self { inner } = self;
                inner.context()
            }

            #[inline]
            pub fn key(&self) -> K {
                let Self { inner } = self;
                inner.key()
            }

            #[inline]
            pub fn as_ptrs(&self) -> Ptrs<'_, V> {
                let Self { inner } = self;
                inner.as_ptrs()
            }

            #[inline]
            pub fn as_ptrs_with_context(&self) -> (&V::Context, Ptrs<'_, V>) {
                let Self { inner } = self;
                inner.as_ptrs_with_context()
            }

            #[inline]
            pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, V> {
                let Self { inner } = self;
                inner.as_mut_ptrs()
            }

            #[inline]
            pub fn as_mut_ptrs_with_context(&mut self) -> (&V::Context, MutPtrs<'_, V>) {
                let Self { inner } = self;
                inner.as_mut_ptrs_with_context()
            }

            #[inline]
            pub fn into_ptrs(self) -> Ptrs<'a, V> {
                let Self { inner } = self;
                inner.into_ptrs()
            }

            #[inline]
            pub fn into_ptrs_with_context(self) -> (&'a V::Context, Ptrs<'a, V>) {
                let Self { inner } = self;
                inner.into_ptrs_with_context()
            }

            #[inline]
            pub fn into_mut_ptrs(self) -> MutPtrs<'a, V> {
                let Self { inner } = self;
                inner.into_mut_ptrs()
            }

            #[inline]
            pub fn into_mut_ptrs_with_context(self) -> (&'a V::Context, MutPtrs<'a, V>) {
                let Self { inner } = self;
                inner.into_mut_ptrs_with_context()
            }

            #[inline]
            pub fn swap_remove<R>(self) -> R
            where
                V: SoaRead<R>,
            {
                let Self { inner } = self;
                inner.swap_remove()
            }

            #[inline]
            pub fn remove<R>(self) -> R
            where
                V: SoaRead<R>,
            {
                let Self { inner } = self;
                inner.remove()
            }

            #[inline]
            pub fn insert<R>(&mut self, value: V) -> R
            where
                V: SoaRead<R> + SoaWrite,
            {
                let Self { inner } = self;
                inner.insert(value)
            }

            #[inline]
            #[track_caller]
            pub fn replace_key(&mut self, key: K) -> Option<V>
            where
                V: SoaRead<V> + SoaWrite,
            {
                let Self { inner } = self;
                inner.replace_key(key)
            }

            #[inline]
            pub fn try_replace_key(&mut self, key: K) -> Result<Option<V>, TryModifyError<K, V>>
            where
                V: SoaRead<V> + SoaWrite,
            {
                let Self { inner } = self;
                inner.try_replace_key(key)
            }
        }

        impl<'a, K, V> OccupiedEntry<'_, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::AllocSoa + $crate::soa::traits::Soa<'a> + ?Sized,
        {
            #[inline]
            pub fn get(&'a self) -> $crate::soa::traits::Refs<'a, 'a, V> {
                let Self { inner } = self;
                inner.get()
            }

            #[inline]
            pub fn get_with_context(
                &'a self,
            ) -> (&'a V::Context, $crate::soa::traits::Refs<'a, 'a, V>) {
                let Self { inner } = self;
                inner.get_with_context()
            }

            #[inline]
            pub fn get_mut(&'a mut self) -> $crate::soa::traits::RefsMut<'a, 'a, V> {
                let Self { inner } = self;
                inner.get_mut()
            }

            #[inline]
            pub fn get_mut_with_context(
                &'a mut self,
            ) -> (&'a V::Context, $crate::soa::traits::RefsMut<'a, 'a, V>) {
                let Self { inner } = self;
                inner.get_mut_with_context()
            }
        }

        impl<'a, K, V> OccupiedEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::AllocSoa + $crate::soa::traits::Soa<'a> + ?Sized,
        {
            #[inline]
            pub fn into_mut(self) -> $crate::soa::traits::RefsMut<'a, 'a, V> {
                let Self { inner } = self;
                inner.into_mut()
            }

            #[inline]
            pub fn into_mut_with_context(
                self,
            ) -> (&'a V::Context, $crate::soa::traits::RefsMut<'a, 'a, V>) {
                let Self { inner } = self;
                inner.into_mut_with_context()
            }
        }

        impl<K, V> core::fmt::Debug for OccupiedEntry<'_, K, V>
        where
            K: $crate::key::Key + core::fmt::Debug,
            V: $crate::soa::traits::AllocSoa + ?Sized,
            for<'ctx, 'a> V: $crate::soa::traits::Soa<
                    'a,
                    Context: $crate::soa::traits::SoaContext<'a, Refs<'ctx>: core::fmt::Debug>,
                >,
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
            V: $crate::soa::traits::AllocSoa + ?Sized,
        {
            inner: $crate::alloc::entry::VacantEntry<'a, K, V, $container>,
        }

        impl<'a, K, V> VacantEntry<'a, K, V>
        where
            K: $crate::key::Key,
            V: $crate::soa::traits::AllocSoa + ?Sized,
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
            pub fn context(&self) -> &V::Context {
                let Self { inner } = self;
                inner.context()
            }

            #[inline]
            pub fn key(&self) -> K {
                let Self { inner } = self;
                inner.key()
            }

            #[inline]
            pub fn insert<R>(self, value: V) -> OccupiedEntry<'a, K, V>
            where
                V: SoaRead<R> + SoaWrite,
            {
                let Self { inner } = self;
                let inner = inner.insert(value);
                OccupiedEntry { inner }
            }
        }

        impl<'a, K, V> core::fmt::Debug for VacantEntry<'a, K, V>
        where
            K: $crate::key::Key + core::fmt::Debug,
            V: $crate::soa::traits::AllocSoa + ?Sized,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let Self { inner } = self;
                inner.fmt(f)
            }
        }
    };
}

pub(super) use generate_entry_types;
