use core::alloc::Layout;

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
