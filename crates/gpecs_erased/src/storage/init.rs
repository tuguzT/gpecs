use core::{
    alloc::Layout,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice,
};

use crate::storage::{AlignedStorage, AlignedStorageFromLayout};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct AlignedInitStorage<T>
where
    T: ?Sized,
{
    inner: T,
}

impl<T> AlignedInitStorage<T> {
    #[inline]
    pub unsafe fn new_unchecked(inner: T) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let Self { inner, .. } = self;
        inner
    }
}

impl<T> AlignedInitStorage<T>
where
    T: ?Sized,
{
    #[inline]
    pub fn as_inner(&self) -> &T {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn as_mut_inner(&mut self) -> &mut T {
        let Self { inner, .. } = self;
        inner
    }
}

impl<T> AlignedInitStorage<T>
where
    T: AlignedStorage<Item: Default>,
{
    #[inline]
    pub fn new(mut slice: T) -> Self {
        slice.as_mut_uninit_slice().fill_with(default_uninit);
        unsafe { Self::new_unchecked(slice) }
    }
}

impl<T> AlignedInitStorage<T>
where
    T: AlignedStorageFromLayout<Item: Default>,
{
    #[inline]
    pub fn from_layout(layout: Layout) -> Result<Self, T::Error> {
        let slice = T::from_layout(layout)?;
        let me = Self::new(slice);
        Ok(me)
    }

    #[inline]
    pub fn set_layout(&mut self, layout: Layout) -> Result<(), T::Error> {
        let Self { inner, .. } = self;

        let old_len = inner.layout().size();
        inner.set_layout(layout)?;

        if let Some(remainder) = inner.as_mut_uninit_slice().get_mut(old_len..) {
            remainder.fill_with(default_uninit);
        }
        Ok(())
    }
}

impl<T> AlignedInitStorage<T>
where
    T: AlignedStorage + ?Sized,
{
    #[inline]
    pub fn as_slice(&self) -> &[T::Item] {
        let Self { inner, .. } = self;
        let slice = inner.as_uninit_slice();

        let data = slice.as_ptr().cast();
        let len = slice.len();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T::Item] {
        let Self { inner, .. } = self;
        let slice = inner.as_mut_uninit_slice();

        let data = slice.as_mut_ptr().cast();
        let len = slice.len();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { inner, .. } = self;
        inner.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T::Item {
        let Self { inner, .. } = self;
        inner.as_mut_ptr()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { inner, .. } = self;
        inner.layout()
    }
}

impl<T> Deref for AlignedInitStorage<T>
where
    T: AlignedStorage + ?Sized,
{
    type Target = [T::Item];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for AlignedInitStorage<T>
where
    T: AlignedStorage + ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, U> AsRef<U> for AlignedInitStorage<T>
where
    T: AlignedStorage + ?Sized,
    U: ?Sized,
    <Self as Deref>::Target: AsRef<U>,
{
    fn as_ref(&self) -> &U {
        self.deref().as_ref()
    }
}

impl<T, U> AsMut<U> for AlignedInitStorage<T>
where
    T: AlignedStorage + ?Sized,
    U: ?Sized,
    <Self as Deref>::Target: AsMut<U>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut U {
        self.deref_mut().as_mut()
    }
}

unsafe impl<T> AlignedStorage for AlignedInitStorage<T>
where
    T: AlignedStorage + ?Sized,
{
    type Item = T::Item;

    #[inline]
    fn as_ptr(&self) -> *const T::Item {
        Self::as_ptr(self)
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T::Item {
        Self::as_mut_ptr(self)
    }

    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(self)
    }
}

unsafe impl<T> AlignedStorageFromLayout for AlignedInitStorage<T>
where
    T: AlignedStorageFromLayout<Item: Default>,
{
    type Error = T::Error;

    #[inline]
    fn from_layout(layout: Layout) -> Result<Self, Self::Error> {
        Self::from_layout(layout)
    }

    #[inline]
    fn set_layout(&mut self, layout: Layout) -> Result<(), Self::Error> {
        Self::set_layout(self, layout)
    }
}

#[inline]
fn default_uninit<T>() -> MaybeUninit<T>
where
    T: Default,
{
    MaybeUninit::new(T::default())
}
