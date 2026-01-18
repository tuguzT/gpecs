use crate::traits::AllocSoa;

use super::vec::SoaVec;

pub struct SetLenOnDrop<'a, T>
where
    T: AllocSoa + ?Sized,
{
    pub vec: &'a mut SoaVec<T>,
    pub local_len: usize,
}

impl<T> Drop for SetLenOnDrop<'_, T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    fn drop(&mut self) {
        let Self {
            ref mut vec,
            local_len,
        } = *self;

        unsafe {
            vec.set_len(local_len);
        }
    }
}
