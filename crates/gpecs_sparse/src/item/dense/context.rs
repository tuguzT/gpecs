use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::soa::traits::RawSoa;

#[repr(transparent)]
pub struct DenseContext<K, V>
where
    V: RawSoa + ?Sized,
{
    context: V::Context,
    phantom: PhantomData<fn() -> K>,
}

impl<K, V> DenseContext<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub const fn from_inner(context: V::Context) -> Self {
        Self {
            context,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub const fn from_inner_ref(context: &V::Context) -> &Self {
        // SAFETY: Self is `#[repr(transparent)]` over `V::Context`.
        unsafe { NonNull::from_ref(context).cast().as_ref() }
    }

    #[inline]
    pub const fn from_inner_mut(context: &mut V::Context) -> &mut Self {
        // SAFETY: Self is `#[repr(transparent)]` over `V::Context`.
        unsafe { NonNull::from_mut(context).cast().as_mut() }
    }

    #[inline]
    pub fn into_inner(self) -> V::Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub const fn as_inner(&self) -> &V::Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub const fn as_inner_mut(&mut self) -> &mut V::Context {
        let Self { context, .. } = self;
        context
    }
}

impl<K, V> Debug for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { context, .. } = self;
        f.debug_tuple("DenseContext").field(context).finish()
    }
}

impl<K, V> Default for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let context = V::Context::default();
        Self::from_inner(context)
    }
}

impl<K, V> Clone for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref context,
            phantom,
        } = *self;

        Self {
            context: context.clone(),
            phantom,
        }
    }
}

impl<K, V> Copy for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Copy,
{
}

impl<K, V> PartialEq for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { context, phantom } = self;
        *context == other.context && *phantom == other.phantom
    }
}

impl<K, V> Eq for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Eq,
{
}

impl<K, V> PartialOrd for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { context, phantom } = self;

        match context.partial_cmp(&other.context) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        phantom.partial_cmp(&other.phantom)
    }
}

impl<K, V> Ord for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { context, phantom } = self;

        match context.cmp(&other.context) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        phantom.cmp(&other.phantom)
    }
}

impl<K, V> Hash for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { context, phantom } = self;

        context.hash(state);
        phantom.hash(state);
    }
}

impl<K, V> Deref for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
{
    type Target = V::Context;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

impl<K, V> DerefMut for DenseContext<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_inner_mut()
    }
}
