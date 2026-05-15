use crate::{
    assert::{assert_occupied, assert_vacant},
    key::{Epoch, SparseIndex},
};

pub trait SparseItem: Copy {
    type Index: SparseIndex;
    type Epoch: Epoch;

    fn occupied(epoch: Self::Epoch, dense_index: Self::Index) -> Self;
    fn vacant(epoch: Self::Epoch) -> Self;

    fn epoch(self) -> Self::Epoch;
    fn dense_index(self) -> Option<Self::Index>;

    #[inline]
    fn is_occupied(self) -> bool {
        self.dense_index().is_some()
    }

    #[inline]
    fn is_vacant(self) -> bool {
        !self.is_occupied()
    }
}

pub trait ArenaSparseItem: SparseItem {
    fn with_next_vacant(epoch: Self::Epoch, next_vacant: Self::Index) -> Self;

    fn index(self) -> (Self::Index, SparseIndexKind);

    #[inline]
    fn next_vacant(self) -> Option<Self::Index> {
        let (index, kind) = self.index();
        match kind {
            SparseIndexKind::Dense => {
                assert_occupied(&self);
                None
            }
            SparseIndexKind::NextVacant => {
                assert_vacant(&self);
                Some(index)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SparseIndexKind {
    Dense,
    NextVacant,
}
