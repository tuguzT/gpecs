pub use crate::{
    item::{DefaultSparseItem, DefaultSparseItemKind, SparseItem},
    key::{Epoch, EpochKey, Key, SparseIndex},
    soa::prelude::*,
    view::{EpochSparseView, EpochSparseViewMut, SparseView, SparseViewMut},
};

#[cfg(feature = "alloc")]
pub use crate::{
    arena::{EpochSparseArena, SparseArena},
    set::{EpochSparseSet, SparseSet},
};
