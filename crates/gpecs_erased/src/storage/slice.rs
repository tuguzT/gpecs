use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Debug, Display},
    hash::{self, Hash},
    marker::PhantomData,
};

use crate::{
    error::{InsufficientLenError, NotAlignedError, check_ptr_align, check_sufficient_len},
    storage::AlignedStorage,
};

pub struct AlignedStorageSlice<T, U>
where
    T: ?Sized,
{
    phantom: PhantomData<fn() -> U>,
    layout: Layout,
    inner: T,
}

impl<T, U> AlignedStorageSlice<T, U> {
    #[inline]
    pub unsafe fn new_unchecked(inner: T, layout: Layout) -> Self {
        Self {
            phantom: PhantomData,
            layout,
            inner,
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let Self { inner, .. } = self;
        inner
    }
}

impl<T, U> AlignedStorageSlice<T, U>
where
    T: AsRef<[U]>,
{
    #[inline]
    pub fn new(inner: T, layout: Layout) -> Result<Self, AlignedUninitStorageError> {
        let slice = inner.as_ref();
        check_sufficient_len(size_of_val(slice), layout.size())?;
        check_ptr_align(slice.as_ptr().cast(), layout)?;

        let me = unsafe { Self::new_unchecked(inner, layout) };
        Ok(me)
    }
}

impl<T, U> AlignedStorageSlice<T, U>
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

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }
}

impl<T, U> AlignedStorageSlice<T, U>
where
    T: AsRef<[U]> + ?Sized,
{
    #[inline]
    pub fn as_slice(&self) -> &[U] {
        let Self { inner, .. } = self;
        inner.as_ref()
    }
}

impl<T, U> AlignedStorageSlice<T, U>
where
    T: AsMut<[U]> + ?Sized,
{
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [U] {
        let Self { inner, .. } = self;
        inner.as_mut()
    }
}

impl<T, U> AsRef<[U]> for AlignedStorageSlice<T, U>
where
    T: AsRef<[U]> + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slice()
    }
}

impl<T, U> AsMut<[U]> for AlignedStorageSlice<T, U>
where
    T: AsMut<[U]> + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slice()
    }
}

impl<T, U> Debug for AlignedStorageSlice<T, U>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layout, inner, .. } = self;
        f.debug_struct("AlignedUninitSlice")
            .field("layout", layout)
            .field("inner", &inner)
            .finish()
    }
}

impl<T, U> Clone for AlignedStorageSlice<T, U>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let Self {
            phantom,
            layout,
            ref inner,
        } = *self;

        Self {
            phantom,
            layout,
            inner: inner.clone(),
        }
    }
}

impl<T, U> Copy for AlignedStorageSlice<T, U> where T: Copy {}

impl<T, U> PartialEq for AlignedStorageSlice<T, U>
where
    T: PartialEq + ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            phantom,
            layout,
            inner,
        } = self;

        *phantom == other.phantom && *layout == other.layout && *inner == other.inner
    }
}

impl<T, U> Eq for AlignedStorageSlice<T, U> where T: Eq + ?Sized {}

impl<T, U> Hash for AlignedStorageSlice<T, U>
where
    T: Hash + ?Sized,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            phantom,
            layout,
            inner,
        } = self;

        phantom.hash(state);
        layout.hash(state);
        inner.hash(state);
    }
}

unsafe impl<T, U> AlignedStorage for AlignedStorageSlice<T, U>
where
    T: AsRef<[U]> + AsMut<[U]> + ?Sized,
{
    type Item = U;

    #[inline]
    fn as_ptr(&self) -> *const U {
        let slice = self.as_slice();
        slice.as_ptr().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut U {
        let slice = self.as_mut_slice();
        slice.as_mut_ptr().cast()
    }

    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(self)
    }
}

#[derive(Debug, Clone)]
pub enum AlignedUninitStorageError {
    NotAligned(NotAlignedError),
    InsufficientLen(InsufficientLenError),
}

impl From<NotAlignedError> for AlignedUninitStorageError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<InsufficientLenError> for AlignedUninitStorageError {
    #[inline]
    fn from(error: InsufficientLenError) -> Self {
        Self::InsufficientLen(error)
    }
}

impl Display for AlignedUninitStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::InsufficientLen(error) => Display::fmt(error, f),
        }
    }
}

impl Error for AlignedUninitStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::InsufficientLen(error) => Some(error),
        }
    }
}
