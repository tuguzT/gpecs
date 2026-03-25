pub type BuildHasher = rustc_hash::FxBuildHasher;
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;
pub type IndexSet<T> = indexmap::IndexSet<T, BuildHasher>;
