use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::Soa;

/// Type wrapper for [field descriptors](Soa::FieldDescriptors)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct FieldDescriptors<'context, T>
where
    T: Soa + ?Sized,
{
    inner: T::FieldDescriptors<'static>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, T> FieldDescriptors<'context, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [field descriptors](Soa::FieldDescriptors).
    #[inline]
    pub fn new(inner: T::FieldDescriptors<'context>) -> Self {
        Self {
            inner: unsafe { transmute::<T::FieldDescriptors<'_>, T::FieldDescriptors<'_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [field descriptors](Soa::FieldDescriptors).
    #[inline]
    pub fn as_inner(&self) -> &T::FieldDescriptors<'context> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [field descriptors](Soa::FieldDescriptors).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T::FieldDescriptors<'context> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [field descriptors](Soa::FieldDescriptors).
    #[inline]
    pub fn into_inner(self) -> T::FieldDescriptors<'context> {
        let Self { inner, .. } = self;
        T::upcast_field_descriptors(inner)
    }
}

impl<T> Debug for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("FieldDescriptors").field(inner).finish()
    }
}

impl<T> Default for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<T> Clone for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: Clone,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: Copy,
{
}

impl<T> PartialEq for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: Eq,
{
}

impl<T> PartialOrd for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner, phantom } = self;
        match inner.partial_cmp(&other.inner) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        phantom.partial_cmp(&other.phantom)
    }
}

impl<T> Ord for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner, phantom } = self;
        match inner.cmp(&other.inner) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        phantom.cmp(&other.phantom)
    }
}

impl<T> Hash for FieldDescriptors<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::FieldDescriptors<'any>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}

impl<'context, T> IntoIterator for FieldDescriptors<'context, T>
where
    T: Soa + ?Sized,
{
    type Item = <T::FieldDescriptors<'context> as IntoIterator>::Item;
    type IntoIter = <T::FieldDescriptors<'context> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}
