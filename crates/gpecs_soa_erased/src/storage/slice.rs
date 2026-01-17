use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Debug, Display},
    hash::{self, Hash},
    marker::PhantomData,
    mem::MaybeUninit,
};

use crate::{
    error::{InsufficientLenError, NotAlignedError, check_ptr_align, check_sufficient_len},
    storage::{AddressableUnit, AlignedStorage},
};

pub struct AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: ?Sized,
{
    phantom: PhantomData<fn() -> A>,
    layout: Layout,
    inner: T,
}

impl<T, A> AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
{
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

impl<T, A> AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: AsRef<[MaybeUninit<A>]>,
{
    #[inline]
    pub fn new(inner: T, layout: Layout) -> Result<Self, AlignedUninitStorageError> {
        let slice = inner.as_ref();
        check_sufficient_len(slice.len() * size_of::<A>(), layout.size())?;
        check_ptr_align(slice.as_ptr().cast(), layout)?;

        let me = unsafe { Self::new_unchecked(inner, layout) };
        Ok(me)
    }
}

impl<T, A> AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
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

impl<T, A> AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: AsRef<[MaybeUninit<A>]> + ?Sized,
{
    #[inline]
    pub fn as_slice(&self) -> &[MaybeUninit<A>] {
        let Self { inner, .. } = self;
        inner.as_ref()
    }
}

impl<T, A> AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: AsMut<[MaybeUninit<A>]> + ?Sized,
{
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [MaybeUninit<A>] {
        let Self { inner, .. } = self;
        inner.as_mut()
    }
}

impl<T, A> AsRef<[MaybeUninit<A>]> for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: AsRef<[MaybeUninit<A>]> + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<A>] {
        self.as_slice()
    }
}

impl<T, A> AsMut<[MaybeUninit<A>]> for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: AsMut<[MaybeUninit<A>]> + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [MaybeUninit<A>] {
        self.as_mut_slice()
    }
}

impl<T, A> Debug for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
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

#[expect(clippy::expl_impl_clone_on_copy, reason = "false positive")]
impl<T, A> Clone for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
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

impl<T, A> Copy for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: Copy,
{
}

impl<T, A> PartialEq for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
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

impl<T, A> Eq for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: Eq + ?Sized,
{
}

impl<T, A> Hash for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
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

unsafe impl<T, A> AlignedStorage<A> for AlignedUninitStorage<T, A>
where
    A: AddressableUnit,
    T: AsRef<[MaybeUninit<A>]> + AsMut<[MaybeUninit<A>]> + ?Sized,
{
    #[inline]
    fn as_ptr(&self) -> *const A {
        let slice = self.as_slice();
        slice.as_ptr().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut A {
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
