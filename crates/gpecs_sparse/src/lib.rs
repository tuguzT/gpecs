//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod arena;
pub mod iter;
pub mod key;
pub mod prelude;
pub mod set;
pub mod view;

mod algo;
mod assert;
mod entry;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SparseItem<E> {
    pub kind: SparseItemKind,
    pub epoch: E,
}

impl<E> SparseItem<E> {
    #[inline]
    pub const fn new(kind: SparseItemKind, epoch: E) -> Self {
        Self { kind, epoch }
    }

    #[inline]
    pub const fn occupied(dense_index: usize, epoch: E) -> Self {
        let kind = SparseItemKind::occupied(dense_index);
        Self::new(kind, epoch)
    }

    #[inline]
    pub const fn vacant(next_vacant: usize, epoch: E) -> Self {
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
    pub const fn kind(&self) -> &SparseItemKind {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub fn kind_mut(&mut self) -> &mut SparseItemKind {
        let Self { kind, .. } = self;
        kind
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<usize> {
        let Self { kind, .. } = self;
        kind.dense_index()
    }

    #[inline]
    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        let Self { kind, .. } = self;
        kind.dense_index_mut()
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<usize> {
        let Self { kind, .. } = self;
        kind.next_vacant()
    }

    #[inline]
    pub fn next_vacant_mut(&mut self) -> Option<&mut usize> {
        let Self { kind, .. } = self;
        kind.next_vacant_mut()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum SparseItemKind {
    Occupied { dense_index: usize },
    Vacant { next_vacant: usize },
}

impl SparseItemKind {
    #[inline]
    pub const fn occupied(dense_index: usize) -> Self {
        Self::Occupied { dense_index }
    }

    #[inline]
    pub const fn vacant(next_vacant: usize) -> Self {
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
    pub const fn dense_index(&self) -> Option<usize> {
        match self {
            Self::Occupied { dense_index } => Some(*dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub fn dense_index_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { dense_index } => Some(dense_index),
            Self::Vacant { .. } => None,
        }
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(*next_vacant),
        }
    }

    #[inline]
    pub fn next_vacant_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Vacant { next_vacant } => Some(next_vacant),
        }
    }
}
