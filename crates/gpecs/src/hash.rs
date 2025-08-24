use rustc_hash::FxBuildHasher;

pub type IndexMap<K, V> = indexmap::IndexMap<K, V, FxBuildHasher>;
pub type IndexSet<T> = indexmap::IndexSet<T, FxBuildHasher>;
