use gpecs_archetype::storage;

use crate::{archetype::ErasedDropMeta, bundle::erased::ErasedBundle};

pub use storage::*;

pub type ArchetypeStorage = storage::ArchetypeStorage<ErasedBundle<ErasedDropMeta>>;
