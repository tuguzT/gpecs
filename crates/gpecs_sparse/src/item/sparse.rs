use crate::key::Key;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
