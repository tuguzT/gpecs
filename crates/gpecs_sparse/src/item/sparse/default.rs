use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use crate::{assert::unwrap_into_index, item::SparseItem, key::Key};

pub struct DefaultSparseItem<K>
where
    K: Key,
{
    pub kind: DefaultSparseItemKind<K::SparseIndex>,
    pub epoch: K::Epoch,
}

impl<K> DefaultSparseItem<K>
where
    K: Key,
{
    #[inline]
    pub const fn new(kind: DefaultSparseItemKind<K::SparseIndex>, epoch: K::Epoch) -> Self {
        Self { kind, epoch }
    }

    #[inline]
    pub const fn occupied(dense_index: K::SparseIndex, epoch: K::Epoch) -> Self {
        let kind = DefaultSparseItemKind::occupied(dense_index);
        Self::new(kind, epoch)
    }

    #[inline]
    pub const fn vacant(next_vacant: K::SparseIndex, epoch: K::Epoch) -> Self {
        let kind = DefaultSparseItemKind::vacant(next_vacant);
        Self::new(kind, epoch)
    }

    #[inline]
    pub const fn is_occupied(&self) -> bool {
        let Self { kind, .. } = self;
        kind.is_occupied()
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        let Self { kind, .. } = self;
        kind.is_vacant()
    }

    #[inline]
    pub const fn kind(&self) -> &DefaultSparseItemKind<K::SparseIndex> {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub const fn kind_mut(&mut self) -> &mut DefaultSparseItemKind<K::SparseIndex> {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub fn into_kind(self) -> DefaultSparseItemKind<K::SparseIndex> {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<&K::SparseIndex> {
        let Self { kind, .. } = self;
        kind.dense_index()
    }

    #[inline]
    pub const fn dense_index_mut(&mut self) -> Option<&mut K::SparseIndex> {
        let Self { kind, .. } = self;
        kind.dense_index_mut()
    }

    #[inline]
    pub fn into_dense_index(self) -> Option<K::SparseIndex> {
        let Self { kind, .. } = self;
        kind.into_dense_index()
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<&K::SparseIndex> {
        let Self { kind, .. } = self;
        kind.next_vacant()
    }

    #[inline]
    pub const fn next_vacant_mut(&mut self) -> Option<&mut K::SparseIndex> {
        let Self { kind, .. } = self;
        kind.next_vacant_mut()
    }

    #[inline]
    pub fn into_next_vacant(self) -> Option<K::SparseIndex> {
        let Self { kind, .. } = self;
        kind.into_next_vacant()
    }
}

impl<K> Debug for DefaultSparseItem<K>
where
    K: Key,
    K::SparseIndex: Debug,
    K::Epoch: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { kind, epoch } = self;
        f.debug_struct("DefaultSparseItem")
            .field("kind", kind)
            .field("epoch", epoch)
            .finish()
    }
}

impl<K> Clone for DefaultSparseItem<K>
where
    K: Key,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for DefaultSparseItem<K> where K: Key {}

impl<K> PartialEq for DefaultSparseItem<K>
where
    K: Key,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { kind, epoch } = self;

        let other = (&other.kind, &other.epoch);
        (kind, epoch) == other
    }
}

impl<K> Eq for DefaultSparseItem<K> where K: Key {}

impl<K> PartialOrd for DefaultSparseItem<K>
where
    K: Key,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K> Ord for DefaultSparseItem<K>
where
    K: Key,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { kind, epoch } = self;

        let other = (&other.kind, &other.epoch);
        (kind, epoch).cmp(&other)
    }
}

impl<K> Hash for DefaultSparseItem<K>
where
    K: Key,
    K::SparseIndex: Hash,
    K::Epoch: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { kind, epoch } = self;
        (kind, epoch).hash(state);
    }
}

impl<K> SparseItem for DefaultSparseItem<K>
where
    K: Key,
{
    type Index = K::SparseIndex;
    type Epoch = K::Epoch;

    #[inline]
    fn occupied(epoch: Self::Epoch, dense_index: Self::Index) -> Self {
        Self::occupied(dense_index, epoch)
    }

    #[inline]
    fn vacant(epoch: Self::Epoch) -> Self {
        Self::vacant(unwrap_into_index(0), epoch)
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {
        self.epoch
    }

    #[inline]
    fn dense_index(self) -> Option<Self::Index> {
        self.into_dense_index()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefaultSparseItemKind<I> {
    Occupied { dense_index: I },
    Vacant { next_vacant: I },
}

impl<I> DefaultSparseItemKind<I> {
    #[inline]
    pub const fn occupied(dense_index: I) -> Self {
        Self::Occupied { dense_index }
    }

    #[inline]
    pub const fn vacant(next_vacant: I) -> Self {
        Self::Vacant { next_vacant }
    }

    #[inline]
    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied { .. })
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant { .. })
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<&I> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub const fn dense_index_mut(&mut self) -> Option<&mut I> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub fn into_dense_index(self) -> Option<I> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<&I> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(next_vacant),
        }
    }

    #[inline]
    pub const fn next_vacant_mut(&mut self) -> Option<&mut I> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(next_vacant),
        }
    }

    #[inline]
    pub fn into_next_vacant(self) -> Option<I> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(next_vacant),
        }
    }
}
