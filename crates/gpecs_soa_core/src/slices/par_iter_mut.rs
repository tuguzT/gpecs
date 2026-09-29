use core::fmt::{self, Debug};

use rayon::iter::{
    IndexedParallelIterator, ParallelIterator,
    plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};

use crate::{
    slices::{IterMut, SoaView, SoaViewMut},
    traits::{RawSoa, RefsMut, Slices, Soa, SoaOwned},
};

#[repr(transparent)]
pub struct ParIterMut<'ctx, 'a, T>
where
    T: RawSoa + ?Sized,
{
    view: SoaViewMut<'ctx, 'a, T>,
}

impl<'ctx, 'a, T> ParIterMut<'ctx, 'a, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(view: SoaViewMut<'ctx, 'a, T>) -> Self {
        Self { view }
    }

    #[inline]
    pub fn as_view(&self) -> SoaView<'_, '_, T> {
        let (_, view) = self.as_view_with_context();
        view
    }

    #[inline]
    pub fn as_view_with_context(&self) -> (&T::Context, SoaView<'_, '_, T>) {
        let Self { view } = self;
        view.as_view_with_context()
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> SoaViewMut<'_, '_, T> {
        let (_, view) = self.as_mut_view_with_context();
        view
    }

    #[inline]
    pub fn as_mut_view_with_context(&mut self) -> (&T::Context, SoaViewMut<'_, '_, T>) {
        let Self { view } = self;
        view.as_mut_view_with_context()
    }

    #[inline]
    pub fn into_view(self) -> SoaViewMut<'ctx, 'a, T> {
        let Self { view } = self;
        view
    }
}

impl<T> Debug for ParIterMut<'_, '_, T>
where
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { view } = self;

        let slices = view.as_slices();
        f.debug_tuple("ParIterMut").field(&slices).finish()
    }
}

impl<'ctx, 'a, T> ParallelIterator for ParIterMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    T::Context: Sync,
    T::Fields: Send,
    RefsMut<'ctx, 'a, T>: Send,
{
    type Item = RefsMut<'ctx, 'a, T>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'ctx, 'a, T> IndexedParallelIterator for ParIterMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    T::Context: Sync,
    T::Fields: Send,
    RefsMut<'ctx, 'a, T>: Send,
{
    fn len(&self) -> usize {
        let Self { view } = self;
        view.len()
    }

    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.callback(self)
    }
}

impl<'ctx, 'a, T> Producer for ParIterMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    T::Context: Sync,
    T::Fields: Send,
{
    type Item = RefsMut<'ctx, 'a, T>;
    type IntoIter = IterMut<'ctx, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { view } = self;
        view.into_iter()
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let Self { view } = self;

        let (left, right) = view.split_at_mut(index);
        (Self::new(left), Self::new(right))
    }
}
