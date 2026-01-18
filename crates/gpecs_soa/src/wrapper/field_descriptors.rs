use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{AllocSoa, AllocSoaContext};

type Inner<'ctx, T> = crate::traits::FieldDescriptors<'ctx, T>;

/// Type wrapper for [field descriptors](AllocSoaContext::FieldDescriptors)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct FieldDescriptors<'ctx, T>
where
    T: AllocSoa + ?Sized,
{
    inner: Inner<'static, T>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, T> FieldDescriptors<'ctx, T>
where
    T: AllocSoa + ?Sized,
{
    /// Creates self from the [field descriptors](AllocSoaContext::FieldDescriptors).
    #[inline]
    pub fn new(inner: Inner<'ctx, T>) -> Self {
        Self {
            inner: unsafe { transmute::<Inner<'_, T>, Inner<'_, T>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [field descriptors](AllocSoaContext::FieldDescriptors).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [field descriptors](AllocSoaContext::FieldDescriptors).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [field descriptors](AllocSoaContext::FieldDescriptors).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, T> {
        let Self { inner, .. } = self;
        T::Context::upcast_field_descriptors(inner)
    }
}

impl<T> Debug for FieldDescriptors<'_, T>
where
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("FieldDescriptors").field(inner).finish()
    }
}

impl<T> Default for FieldDescriptors<'_, T>
where
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Default,
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
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Clone,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for FieldDescriptors<'_, T>
where
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Copy,
{
}

impl<T> PartialEq for FieldDescriptors<'_, T>
where
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for FieldDescriptors<'_, T>
where
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Eq,
{
}

impl<T> PartialOrd for FieldDescriptors<'_, T>
where
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: PartialOrd,
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
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Ord,
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
    T: AllocSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}

impl<'ctx, T> IntoIterator for FieldDescriptors<'ctx, T>
where
    T: AllocSoa + ?Sized,
{
    type Item = <Inner<'ctx, T> as IntoIterator>::Item;
    type IntoIter = <Inner<'ctx, T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}
