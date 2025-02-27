use std::{ops::Deref, slice::Iter, vec::IntoIter};

use gpecs_sparse::{arena::EpochSparseArena, key::EpochKey};

pub type Id = EpochKey;

pub type TryReserveError = gpecs_sparse::error::TryReserveError;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdRegistry {
    inner: EpochSparseArena<Id, ()>,
}

impl IdRegistry {
    #[inline]
    pub fn new() -> Self {
        let inner = EpochSparseArena::new();
        Self { inner }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let inner = EpochSparseArena::with_capacity(capacity, capacity);
        Self { inner }
    }

    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let inner = EpochSparseArena::try_with_capacity(capacity, capacity)?;
        Ok(Self { inner })
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { inner } = self;
        inner.is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let Self { inner } = self;
        inner.reserve(additional, additional);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        let Self { inner } = self;
        inner.reserve_exact(additional, additional);
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { inner } = self;
        inner.try_reserve(additional, additional)
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let Self { inner } = self;
        inner.try_reserve_exact(additional, additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let Self { inner } = self;
        inner.dense_shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let Self { inner } = self;
        inner.dense_shrink_to(min_capacity);
    }

    #[inline]
    pub fn as_slice(&self) -> &[Id] {
        let Self { inner } = self;
        inner.as_keys_slice()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const Id {
        let Self { inner } = self;
        inner.as_keys_ptr()
    }

    #[inline]
    pub fn insert(&mut self, id: Id) {
        let Self { inner } = self;
        inner.insert(id, ());
    }

    #[inline]
    pub fn try_insert(&mut self, id: Id) -> Result<(), TryReserveError> {
        let Self { inner } = self;

        inner.try_insert(id, ())?;
        Ok(())
    }

    #[inline]
    pub fn push(&mut self) -> Id {
        let Self { inner } = self;
        inner.push(())
    }

    #[inline]
    pub fn try_push(&mut self) -> Result<Id, TryReserveError> {
        let Self { inner } = self;
        inner.try_push(())
    }

    #[inline]
    pub fn remove(&mut self, id: Id) {
        let Self { inner } = self;
        inner.remove(id);
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<usize> {
        let Self { inner } = self;
        inner.get_epoch(sparse_index)
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, id: Id) -> Option<Id> {
        let Self { inner } = self;
        inner.invalidate_epoch(id)
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        let Self { inner } = self;
        inner.truncate(len, usize::MAX);
    }

    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(Id) -> bool,
    {
        let Self { inner } = self;
        inner.retain(|id, _| f(id));
    }

    #[inline]
    pub fn contains(&self, id: Id) -> bool {
        let Self { inner } = self;
        inner.contains_key(id)
    }

    #[inline]
    pub fn clear(&mut self) {
        let Self { inner } = self;
        inner.clear();
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Id> {
        self.as_slice().iter()
    }
}

impl From<Vec<Id>> for IdRegistry {
    #[inline]
    fn from(value: Vec<Id>) -> Self {
        value.into_iter().collect()
    }
}

impl From<IdRegistry> for Vec<Id> {
    #[inline]
    fn from(storage: IdRegistry) -> Self {
        let IdRegistry { inner } = storage;
        inner.into_keys_vec()
    }
}

impl Clone for IdRegistry {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let Self { inner: this } = self;
        let Self { inner: source } = source;
        this.clone_from(source);
    }
}

impl AsRef<[Id]> for IdRegistry {
    #[inline]
    fn as_ref(&self) -> &[Id] {
        self.as_slice()
    }
}

impl AsRef<IdRegistry> for IdRegistry {
    #[inline]
    fn as_ref(&self) -> &IdRegistry {
        self
    }
}

impl AsMut<IdRegistry> for IdRegistry {
    #[inline]
    fn as_mut(&mut self) -> &mut IdRegistry {
        self
    }
}

impl Deref for IdRegistry {
    type Target = [Id];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a> IntoIterator for &'a IdRegistry {
    type Item = &'a Id;
    type IntoIter = Iter<'a, Id>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for IdRegistry {
    type Item = Id;
    type IntoIter = IntoIter<Id>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let vec: Vec<_> = self.into();
        vec.into_iter()
    }
}

impl FromIterator<Id> for IdRegistry {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Id>>(iter: T) -> Self {
        let inner = iter.into_iter().map(|id| (id, ())).collect();
        Self { inner }
    }
}

impl Extend<Id> for IdRegistry {
    #[inline]
    fn extend<T: IntoIterator<Item = Id>>(&mut self, iter: T) {
        let Self { inner } = self;
        inner.extend(iter.into_iter().map(|id| (id, ())));
    }
}
