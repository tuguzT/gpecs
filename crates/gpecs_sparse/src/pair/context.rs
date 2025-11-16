use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::soa::traits::Soa;

#[repr(transparent)]
pub struct KeyValuePairContext<K, V>
where
    V: Soa + ?Sized,
{
    context: V::Context,
    phantom: PhantomData<fn() -> K>,
}

impl<K, V> KeyValuePairContext<K, V>
where
    V: Soa + ?Sized,
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

impl<K, V> Default for KeyValuePairContext<K, V>
where
    V: Soa + ?Sized,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let context = V::Context::default();
        Self::from_inner(context)
    }
}

impl<K, V> Deref for KeyValuePairContext<K, V>
where
    V: Soa + ?Sized,
{
    type Target = V::Context;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

impl<K, V> DerefMut for KeyValuePairContext<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_inner_mut()
    }
}
