use crate::soa::field::FieldDescriptor;

#[inline]
pub fn item_count<T>(desc: FieldDescriptor) -> usize {
    from_bytes_to_items::<T>(desc.layout().size())
}

#[inline]
pub fn from_bytes_to_items<T>(count_in_bytes: usize) -> usize {
    count_in_bytes.div_ceil(size_of::<T>())
}
