use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use crate::key::Key;

pub struct SparseItem<K>
where
    K: Key,
{
    pub kind: SparseItemKind<K::SparseIndex>,
    pub epoch: K::Epoch,
}

impl<K> SparseItem<K>
where
    K: Key,
{
    #[inline]
    pub const fn new(kind: SparseItemKind<K::SparseIndex>, epoch: K::Epoch) -> Self {
        Self { kind, epoch }
    }

    #[inline]
    pub const fn occupied(dense_index: K::SparseIndex, epoch: K::Epoch) -> Self {
        let kind = SparseItemKind::occupied(dense_index);
        Self::new(kind, epoch)
    }

    #[inline]
    pub const fn vacant(next_vacant: K::SparseIndex, epoch: K::Epoch) -> Self {
        let kind = SparseItemKind::vacant(next_vacant);
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
    pub const fn kind(&self) -> &SparseItemKind<K::SparseIndex> {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub const fn kind_mut(&mut self) -> &mut SparseItemKind<K::SparseIndex> {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub fn into_kind(self) -> SparseItemKind<K::SparseIndex> {
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

impl<K> Debug for SparseItem<K>
where
    K: Key,
    K::SparseIndex: Debug,
    K::Epoch: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { kind, epoch } = self;
        f.debug_struct("SparseItem")
            .field("kind", kind)
            .field("epoch", epoch)
            .finish()
    }
}

#[expect(clippy::expl_impl_clone_on_copy)]
impl<K> Clone for SparseItem<K>
where
    K: Key,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for SparseItem<K> where K: Key {}

impl<K> PartialEq for SparseItem<K>
where
    K: Key,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { kind, epoch } = self;
        *kind == other.kind && *epoch == other.epoch
    }
}

impl<K> Eq for SparseItem<K> where K: Key {}

impl<K> PartialOrd for SparseItem<K>
where
    K: Key,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K> Ord for SparseItem<K>
where
    K: Key,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { kind, epoch } = self;

        match kind.cmp(&other.kind) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        epoch.cmp(&other.epoch)
    }
}

impl<K> Hash for SparseItem<K>
where
    K: Key,
    K::SparseIndex: Hash,
    K::Epoch: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { kind, epoch } = self;
        kind.hash(state);
        epoch.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SparseItemKind<I> {
    Occupied { dense_index: I },
    Vacant { next_vacant: I },
}

impl<I> SparseItemKind<I> {
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
