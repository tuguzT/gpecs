use crate::component::erased::ErasedDrop;

pub trait WithErasedDrop {
    fn erased_drop(&self) -> Option<ErasedDrop>;
}

impl<T> WithErasedDrop for &T
where
    T: WithErasedDrop + ?Sized,
{
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        (**self).erased_drop()
    }
}

impl WithErasedDrop for ErasedDrop {
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        Some(*self)
    }
}

impl WithErasedDrop for Option<ErasedDrop> {
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        *self
    }
}
