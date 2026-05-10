use core::{convert::Infallible, mem, ptr};

use gpecs_erased::ptr::slice::MutSliceItemPtr;

use crate::{
    Component,
    erased::{ErasedComponentMutPtr, ErasedComponentMutSlicePtr},
};

type Inner = unsafe fn(*mut Infallible);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ErasedDrop(Inner);

impl ErasedDrop {
    #[inline]
    pub fn of<C>() -> Option<Self>
    where
        C: Component,
    {
        if !mem::needs_drop::<C>() {
            return None;
        }

        let inner = erased_drop::<C>;
        let me = unsafe { Self::from_inner(inner) };
        Some(me)
    }

    #[inline]
    pub unsafe fn from_inner(inner: Inner) -> Self {
        Self(inner)
    }

    #[inline]
    pub fn into_inner(self) -> Inner {
        let Self(inner) = self;
        inner
    }

    #[inline]
    #[track_caller]
    pub unsafe fn drop_in_place<T>(self, to_drop: ErasedComponentMutPtr<T>)
    where
        T: MutSliceItemPtr,
    {
        let Self(inner) = self;

        let to_drop = unsafe { to_drop.as_mut_ptr().cast() };
        unsafe { inner(to_drop) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn drop_in_place_slice<T>(self, to_drop: ErasedComponentMutSlicePtr<T>)
    where
        T: MutSliceItemPtr,
    {
        for i in 0..to_drop.len() {
            let to_drop = unsafe { to_drop.component_ptr().add(i) };
            unsafe { self.drop_in_place(to_drop) }
        }
    }
}

unsafe fn erased_drop<C>(to_drop: *mut Infallible)
where
    C: Component,
{
    let to_drop = to_drop.cast();
    unsafe { ptr::drop_in_place::<C>(to_drop) };
}

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

impl<T> WithErasedDrop for &mut T
where
    T: WithErasedDrop + ?Sized,
{
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        (**self).erased_drop()
    }
}

impl<K, V> WithErasedDrop for (K, V)
where
    V: WithErasedDrop + ?Sized,
{
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        let (_, value) = self;
        value.erased_drop()
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
