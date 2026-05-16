use core::fmt::{self, Debug};

use bytemuck::{Pod, Zeroable};
use gpecs_sparse::item::{ArenaSparseItem, SparseIndexKind, SparseItem};

use crate::EntityEpoch;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct EntitySparseItem {
    index: u32,
    epoch_kind: u32,
}

impl EntitySparseItem {
    const BITS: u32 = u16::BITS;
    const LO_BITS_MASK: u32 = u16::MAX as u32;
    const HI_BITS_MASK: u32 = !Self::LO_BITS_MASK;

    #[inline]
    pub const fn new(index: u32, epoch: EntityEpoch, kind: SparseIndexKind) -> Self {
        let kind = sparse_index_kind_to_u32(kind);
        let epoch_kind = (kind << Self::BITS) | epoch.into_u32();
        Self { index, epoch_kind }
    }

    #[inline]
    pub const fn occupied(dense_index: u32, epoch: EntityEpoch) -> Self {
        Self::new(dense_index, epoch, SparseIndexKind::Dense)
    }

    #[inline]
    pub const fn vacant(next_vacant: u32, epoch: EntityEpoch) -> Self {
        Self::new(next_vacant, epoch, SparseIndexKind::NextVacant)
    }

    #[inline]
    pub const fn is_occupied(&self) -> bool {
        matches!(self.kind(), SparseIndexKind::Dense)
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        matches!(self.kind(), SparseIndexKind::NextVacant)
    }

    #[inline]
    pub const fn index(&self) -> u32 {
        let Self { index, .. } = *self;
        index
    }

    #[inline]
    pub const fn set_index(&mut self, index: u32) {
        self.index = index;
    }

    #[inline]
    pub const fn epoch(&self) -> EntityEpoch {
        let Self { epoch_kind, .. } = *self;
        let epoch = epoch_kind & Self::LO_BITS_MASK;
        unsafe { EntityEpoch::from_u32(epoch) }
    }

    #[inline]
    pub const fn set_epoch(&mut self, epoch: EntityEpoch) {
        self.epoch_kind = (self.epoch_kind & Self::HI_BITS_MASK) | epoch.into_u32();
    }

    #[inline]
    pub const fn kind(&self) -> SparseIndexKind {
        let Self { epoch_kind, .. } = *self;
        let kind = epoch_kind >> Self::BITS;
        sparse_index_kind_from_u32(kind)
    }

    #[inline]
    pub const fn set_kind(&mut self, kind: SparseIndexKind) {
        let epoch = Self::epoch(self);
        let kind = sparse_index_kind_to_u32(kind);
        self.epoch_kind = (kind << Self::BITS) | epoch.into_u32();
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<u32> {
        match self.kind() {
            SparseIndexKind::Dense => Some(self.index()),
            SparseIndexKind::NextVacant => None,
        }
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<u32> {
        match self.kind() {
            SparseIndexKind::Dense => None,
            SparseIndexKind::NextVacant => Some(self.index()),
        }
    }
}

impl Debug for EntitySparseItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = &self.index();
        let epoch = &self.epoch();
        let kind = &self.kind();

        f.debug_struct("EntitySparseItem")
            .field("index", index)
            .field("epoch", epoch)
            .field("kind", kind)
            .finish()
    }
}

impl SparseItem for EntitySparseItem {
    type Index = u32;
    type Epoch = EntityEpoch;

    #[inline]
    fn occupied(epoch: Self::Epoch, dense_index: Self::Index) -> Self {
        Self::occupied(dense_index, epoch)
    }

    #[inline]
    fn vacant(epoch: Self::Epoch) -> Self {
        Self::vacant(0, epoch)
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {
        Self::epoch(&self)
    }

    #[inline]
    fn dense_index(self) -> Option<Self::Index> {
        Self::dense_index(&self)
    }

    #[inline]
    fn is_occupied(self) -> bool {
        Self::is_occupied(&self)
    }

    #[inline]
    fn is_vacant(self) -> bool {
        Self::is_vacant(&self)
    }
}

impl ArenaSparseItem for EntitySparseItem {
    #[inline]
    fn with_next_vacant(epoch: Self::Epoch, next_vacant: Self::Index) -> Self {
        Self::vacant(next_vacant, epoch)
    }

    #[inline]
    fn index(self) -> (Self::Index, SparseIndexKind) {
        (Self::index(&self), Self::kind(&self))
    }

    #[inline]
    fn next_vacant(self) -> Option<Self::Index> {
        Self::next_vacant(&self)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct NoEpochEntitySparseItem {
    index: u32,
    kind: u32,
}

impl NoEpochEntitySparseItem {
    #[inline]
    pub const fn new(index: u32, kind: SparseIndexKind) -> Self {
        let kind = sparse_index_kind_to_u32(kind);
        Self { index, kind }
    }

    #[inline]
    pub const fn occupied(dense_index: u32) -> Self {
        Self::new(dense_index, SparseIndexKind::Dense)
    }

    #[inline]
    pub const fn vacant(next_vacant: u32) -> Self {
        Self::new(next_vacant, SparseIndexKind::NextVacant)
    }

    #[inline]
    pub const fn is_occupied(&self) -> bool {
        matches!(self.kind(), SparseIndexKind::Dense)
    }

    #[inline]
    pub const fn is_vacant(&self) -> bool {
        matches!(self.kind(), SparseIndexKind::NextVacant)
    }

    #[inline]
    pub const fn index(&self) -> u32 {
        let Self { index, .. } = *self;
        index
    }

    #[inline]
    pub const fn set_index(&mut self, index: u32) {
        self.index = index;
    }

    #[inline]
    pub const fn kind(&self) -> SparseIndexKind {
        let Self { kind, .. } = *self;
        sparse_index_kind_from_u32(kind)
    }

    #[inline]
    pub const fn set_kind(&mut self, kind: SparseIndexKind) {
        self.kind = sparse_index_kind_to_u32(kind);
    }

    #[inline]
    pub const fn dense_index(&self) -> Option<u32> {
        match self.kind() {
            SparseIndexKind::Dense => Some(self.index()),
            SparseIndexKind::NextVacant => None,
        }
    }

    #[inline]
    pub const fn next_vacant(&self) -> Option<u32> {
        match self.kind() {
            SparseIndexKind::Dense => None,
            SparseIndexKind::NextVacant => Some(self.index()),
        }
    }
}

impl Debug for NoEpochEntitySparseItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = &self.index();
        let kind = &self.kind();

        f.debug_struct("NoEpochEntitySparseItem")
            .field("index", index)
            .field("kind", kind)
            .finish()
    }
}

impl SparseItem for NoEpochEntitySparseItem {
    type Index = u32;
    type Epoch = ();

    #[inline]
    fn occupied((): Self::Epoch, dense_index: Self::Index) -> Self {
        Self::occupied(dense_index)
    }

    #[inline]
    fn vacant((): Self::Epoch) -> Self {
        Self::vacant(0)
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {}

    #[inline]
    fn dense_index(self) -> Option<Self::Index> {
        Self::dense_index(&self)
    }

    #[inline]
    fn is_occupied(self) -> bool {
        Self::is_occupied(&self)
    }

    #[inline]
    fn is_vacant(self) -> bool {
        Self::is_vacant(&self)
    }
}

impl ArenaSparseItem for NoEpochEntitySparseItem {
    #[inline]
    fn with_next_vacant((): Self::Epoch, next_vacant: Self::Index) -> Self {
        Self::vacant(next_vacant)
    }

    #[inline]
    fn index(self) -> (Self::Index, SparseIndexKind) {
        (Self::index(&self), Self::kind(&self))
    }

    #[inline]
    fn next_vacant(self) -> Option<Self::Index> {
        Self::next_vacant(&self)
    }
}

#[inline]
const fn sparse_index_kind_to_u32(kind: SparseIndexKind) -> u32 {
    match kind {
        SparseIndexKind::Dense => 0,
        SparseIndexKind::NextVacant => 1,
    }
}

#[inline]
const fn sparse_index_kind_from_u32(kind: u32) -> SparseIndexKind {
    match kind {
        0 => SparseIndexKind::Dense,
        1 => SparseIndexKind::NextVacant,
        _ => panic!("invalid sparse index kind bit representation"),
    }
}
