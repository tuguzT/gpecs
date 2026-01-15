use core::{
    alloc::Layout,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice,
};

use crate::storage::{AddressableUnit, AlignedSlice, AlignedSliceFromLayout};

pub struct AlignedInitSlice<T, A>
where
    T: ?Sized,
    A: AddressableUnit,
{
    phantom: PhantomData<fn() -> A>,
    inner: T,
}

impl<T, A> AlignedInitSlice<T, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(inner: T) -> Self {
        let phantom = PhantomData;
        Self { phantom, inner }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let Self { inner, .. } = self;
        inner
    }
}

impl<T, A> AlignedInitSlice<T, A>
where
    T: ?Sized,
    A: AddressableUnit,
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

impl<T, A> AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSlice<A>,
{
    #[inline]
    pub fn new(mut slice: T) -> Self {
        slice.as_mut_uninit_slice().fill_with(default_uninit);
        unsafe { Self::new_unchecked(slice) }
    }
}

impl<T, A> AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSliceFromLayout<A>,
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

impl<T, A> AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSlice<A> + ?Sized,
{
    #[inline]
    pub fn as_slice(&self) -> &[A] {
        let Self { inner, .. } = self;
        let slice = inner.as_uninit_slice();

        let data = slice.as_ptr().cast();
        let len = slice.len();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [A] {
        let Self { inner, .. } = self;
        let slice = inner.as_mut_uninit_slice();

        let data = slice.as_mut_ptr().cast();
        let len = slice.len();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const A {
        let Self { inner, .. } = self;
        inner.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut A {
        let Self { inner, .. } = self;
        inner.as_mut_ptr()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { inner, .. } = self;
        inner.layout()
    }
}

impl<T, A> Debug for AlignedInitSlice<T, A>
where
    T: Debug + ?Sized,
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("AlignedInitSlice").field(&inner).finish()
    }
}

#[expect(clippy::expl_impl_clone_on_copy, reason = "false positive")]
impl<T, A> Clone for AlignedInitSlice<T, A>
where
    T: Clone,
    A: AddressableUnit,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { phantom, ref inner } = *self;
        let inner = inner.clone();
        Self { phantom, inner }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let Self { phantom, inner } = self;

        inner.clone_from(&source.inner);
        phantom.clone_from(&source.phantom);
    }
}

impl<T, A> Copy for AlignedInitSlice<T, A>
where
    T: Copy,
    A: AddressableUnit,
{
}

impl<T, A> PartialEq for AlignedInitSlice<T, A>
where
    T: PartialEq + ?Sized,
    A: AddressableUnit,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { phantom, inner } = self;
        *phantom == other.phantom && *inner == other.inner
    }
}

impl<T, A> Eq for AlignedInitSlice<T, A>
where
    T: Eq + ?Sized,
    A: AddressableUnit,
{
}

impl<T, A> PartialOrd for AlignedInitSlice<T, A>
where
    T: PartialOrd + ?Sized,
    A: AddressableUnit,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { phantom, inner } = self;

        match phantom.partial_cmp(&other.phantom) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        inner.partial_cmp(&other.inner)
    }
}

impl<T, A> Ord for AlignedInitSlice<T, A>
where
    T: Ord + ?Sized,
    A: AddressableUnit,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { phantom, inner } = self;

        match phantom.cmp(&other.phantom) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        inner.cmp(&other.inner)
    }
}

impl<T, A> Hash for AlignedInitSlice<T, A>
where
    T: Hash + ?Sized,
    A: AddressableUnit,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { phantom, inner } = self;

        phantom.hash(state);
        inner.hash(state);
    }
}

impl<T, A> Deref for AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSlice<A> + ?Sized,
{
    type Target = [A];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, A> DerefMut for AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSlice<A> + ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, A> AsRef<[A]> for AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSlice<A> + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[A] {
        self
    }
}

impl<T, A> AsMut<[A]> for AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSlice<A> + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        self
    }
}

unsafe impl<T, A> AlignedSlice<A> for AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSlice<A> + ?Sized,
{
    #[inline]
    fn as_ptr(&self) -> *const A {
        Self::as_ptr(self)
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut A {
        Self::as_mut_ptr(self)
    }

    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(self)
    }
}

unsafe impl<T, A> AlignedSliceFromLayout<A> for AlignedInitSlice<T, A>
where
    A: AddressableUnit,
    T: AlignedSliceFromLayout<A>,
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
