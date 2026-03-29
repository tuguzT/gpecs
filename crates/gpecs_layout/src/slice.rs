#[inline]
pub const fn bytes_to_items<T>(count_in_bytes: usize) -> usize {
    match size_of::<T>() {
        0 => usize::MAX,
        item_size => count_in_bytes / item_size,
    }
}
