use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
};

use crate::traits::Soa;

#[repr(transparent)]
pub struct NonNullPtrs<'context, T>
where
    T: Soa,
{
    inner: T::NonNullPtrs<'static>,
    _phantom: PhantomData<&'context ()>,
}

impl<'context, T> NonNullPtrs<'context, T>
where
    T: Soa,
{
    #[inline]
    pub fn new(inner: T::NonNullPtrs<'context>) -> Self {
        Self {
            inner: unsafe { transmute(inner) },
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_inner(&self) -> &T::NonNullPtrs<'context> {
        let Self { inner, .. } = self;
        unsafe { transmute(inner) }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn as_inner_mut(&mut self) -> &mut T::NonNullPtrs<'context> {
        let Self { inner, .. } = self;
        unsafe { transmute(inner) }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn into_inner(self) -> T::NonNullPtrs<'context> {
        let Self { inner, .. } = self;
        unsafe { transmute(inner) }
    }
}

impl<T> Debug for NonNullPtrs<'_, T>
where
    T: Soa,
    for<'any> T::NonNullPtrs<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.as_inner();
        f.debug_tuple("NonNullPtrs").field(inner).finish()
    }
}

impl<T> Default for NonNullPtrs<'_, T>
where
    T: Soa,
    for<'any> T::NonNullPtrs<'any>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<T> Clone for NonNullPtrs<'_, T>
where
    T: Soa,
{
    fn clone(&self) -> Self {
        let Self { inner, _phantom } = self;
        Self {
            inner: inner.clone(),
            _phantom: _phantom.clone(),
        }
    }
}

impl<T> Copy for NonNullPtrs<'_, T>
where
    T: Soa,
    for<'any> T::NonNullPtrs<'any>: Copy,
{
}

impl<T> PartialEq for NonNullPtrs<'_, T>
where
    T: Soa,
    for<'any> T::NonNullPtrs<'any>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, _phantom } = self;
        *inner == other.inner && *_phantom == other._phantom
    }
}

impl<T> Eq for NonNullPtrs<'_, T>
where
    T: Soa,
    for<'any> T::NonNullPtrs<'any>: Eq,
{
}

impl<T> PartialOrd for NonNullPtrs<'_, T>
where
    T: Soa,
    for<'any> T::NonNullPtrs<'any>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner, _phantom } = self;

        match inner.partial_cmp(&other.inner) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        _phantom.partial_cmp(&other._phantom)
    }
}

impl<T> Ord for NonNullPtrs<'_, T>
where
    T: Soa,
    for<'any> T::NonNullPtrs<'any>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner, _phantom } = self;
        match inner.cmp(&other.inner) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        _phantom.cmp(&other._phantom)
    }
}

impl<T> Hash for NonNullPtrs<'_, T>
where
    T: Soa,
    for<'any> T::NonNullPtrs<'any>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
        self._phantom.hash(state);
    }
}
