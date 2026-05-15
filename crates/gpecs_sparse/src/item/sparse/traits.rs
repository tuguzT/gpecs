use crate::key::{Epoch, SparseIndex};

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
