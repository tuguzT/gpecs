use core_alloc::vec::Vec;

use crate::registry::traits::PushBackArray;

unsafe impl<T> PushBackArray for Vec<T> {
    type Item = T;

    #[inline]
    fn push(&mut self, value: Self::Item) {
        Vec::push(self, value);
    }
}
