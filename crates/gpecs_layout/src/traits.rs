use core::alloc::Layout;

use crate::FfiLayout;

pub trait WithLayout {
    fn layout(&self) -> Layout;
}

impl<T> WithLayout for &T
where
    T: WithLayout + ?Sized,
{
    #[inline]
    fn layout(&self) -> Layout {
        (**self).layout()
    }
}

impl<T> WithLayout for &mut T
where
    T: WithLayout + ?Sized,
{
    #[inline]
    fn layout(&self) -> Layout {
        (**self).layout()
    }
}

impl WithLayout for Layout {
    #[inline]
    fn layout(&self) -> Layout {
        *self
    }
}

impl WithLayout for FfiLayout {
    #[inline]
    fn layout(&self) -> Layout {
        (*self).into()
    }
}
