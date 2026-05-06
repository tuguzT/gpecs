use crate::soa::{field::BufferLayout, layout::WithLayout};

pub unsafe trait FieldOffsets<T> {
    unsafe fn next(&mut self, desc: T) -> usize;
}

unsafe impl<T, U> FieldOffsets<T> for &mut U
where
    U: FieldOffsets<T> + ?Sized,
{
    #[inline]
    unsafe fn next(&mut self, desc: T) -> usize {
        unsafe { (**self).next(desc) }
    }
}

unsafe impl<T> FieldOffsets<T> for BufferLayout
where
    T: WithLayout,
{
    #[inline]
    unsafe fn next(&mut self, desc: T) -> usize {
        let layout = desc.layout();
        unsafe { self.extend_unchecked(layout) }
    }
}
