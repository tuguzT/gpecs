use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
    ptr::NonNull,
};

use gpecs_soa_erased::{CovariantFieldDescriptors, ErasedSoaNonNullPtrs, ErasedSoaNonNullPtrsIter};

use crate::{
    archetype::erased::{ErasedArchetypeView, error::IncompatibleArchetypeError},
    bundle::{
        Bundle, BundleNonNullPtrs,
        erased::{
            ErasedBundleMutPtrs,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    component::{
        erased::ErasedComponentNonNullPtr,
        registry::{
            ComponentId, ComponentRegistryView,
            traits::{ComponentIdFrom, FromComponentType, WithComponentId},
        },
    },
    soa::{
        field::{
            FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput, FieldDescriptorsOwned,
        },
        traits::RawSoaContext,
    },
};

type Inner<D> = ErasedSoaNonNullPtrs<D, NonNull<MaybeUninit<u8>>>;

#[derive(Debug)]
pub struct ErasedBundleNonNullPtrs<D>
where
    D: ?Sized,
{
    inner: Inner<D>,
}

impl<D> ErasedBundleNonNullPtrs<D> {
    #[inline]
    pub fn new(ptrs: ErasedBundleMutPtrs<D>) -> Option<Self> {
        let ptrs = ptrs.into_inner();
        let inner = Inner::new(ptrs)?;

        let me = unsafe { Self::from_inner(inner) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptrs: ErasedBundleMutPtrs<D>) -> Self {
        let ptrs = ptrs.into_inner();
        let inner = unsafe { Inner::new_unchecked(ptrs) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn from_inner(inner: Inner<D>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> Inner<D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { inner } = self;

        let inner = unsafe { inner.add(count) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<D> ErasedBundleNonNullPtrs<D>
where
    D: ?Sized,
{
    #[inline]
    pub unsafe fn as_inner(&self) -> &Inner<D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub unsafe fn as_mut_inner(&mut self) -> &mut Inner<D> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn as_buffer(&self) -> NonNull<[MaybeUninit<u8>]> {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner } = self;
        inner.offset()
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { inner } = self;
        inner.descriptors()
    }
}

impl<'a, D> ErasedBundleNonNullPtrs<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'n, N>(&'a self, origin: &'n ErasedBundleNonNullPtrs<N>) -> isize
    where
        N: FieldDescriptors<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let origin = unsafe { origin.as_inner() };
        unsafe { inner.offset_from(origin) }
    }

    #[inline]
    pub fn iter(&'a self) -> ErasedBundleNonNullPtrsIter<FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleNonNullPtrsIter::from_inner(inner) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<'n, N>(&'a mut self, with: &'n mut ErasedBundleNonNullPtrs<N>)
    where
        N: FieldDescriptors<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let with = unsafe { with.as_mut_inner() };
        unsafe { inner.swap(with) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward<'n, N>(
        &'a mut self,
        src: &'n ErasedBundleNonNullPtrs<N>,
        count: usize,
    ) where
        N: FieldDescriptors<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_forward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward<'n, N>(
        &'a mut self,
        src: &'n ErasedBundleNonNullPtrs<N>,
        count: usize,
    ) where
        N: FieldDescriptors<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_backward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'n, N>(
        &'a mut self,
        src: &'n ErasedBundleNonNullPtrs<N>,
        count: usize,
    ) where
        N: FieldDescriptors<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_nonoverlapping(src, count) }
    }
}

impl<D> ErasedBundleNonNullPtrs<D>
where
    D: ErasedArchetypeKind,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleNonNullPtrs<B>, IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let ptrs = ErasedBundleMutPtrs::from(self).downcast::<B, T>(components)?;
        let ptrs = unsafe { B::CONTEXT.ptrs_to_nonnull(ptrs) };
        Ok(ptrs)
    }
}

impl<D> ErasedBundleNonNullPtrs<D>
where
    D: ErasedArchetypeKind + ?Sized,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentNonNullPtr> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D> Clone for ErasedBundleNonNullPtrs<D>
where
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D> Copy for ErasedBundleNonNullPtrs<D> where D: Copy {}

impl<'a, D> IntoIterator for &'a ErasedBundleNonNullPtrs<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentNonNullPtr;
    type IntoIter = ErasedBundleNonNullPtrsIter<FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> IntoIterator for ErasedBundleNonNullPtrs<D>
where
    D: IntoErasedArchetypeIterator,
{
    type Item = ErasedComponentNonNullPtr;
    type IntoIter = ErasedBundleNonNullPtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleNonNullPtrsIter::from_inner(inner) }
    }
}

impl<D> From<ErasedBundleNonNullPtrs<D>> for ErasedBundleMutPtrs<D> {
    #[inline]
    fn from(ptrs: ErasedBundleNonNullPtrs<D>) -> Self {
        let inner = ptrs.into_inner();
        let inner = inner.into();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleNonNullPtrs<D>
where
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_descriptors()
    }
}

impl<D> CovariantFieldDescriptors for ErasedBundleNonNullPtrs<D>
where
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}

type InnerIter<D> = ErasedSoaNonNullPtrsIter<D, NonNull<MaybeUninit<u8>>>;

pub struct ErasedBundleNonNullPtrsIter<D>
where
    D: ?Sized,
{
    inner: InnerIter<D>,
}

impl<D> ErasedBundleNonNullPtrsIter<D> {
    #[inline]
    pub(super) unsafe fn from_inner(inner: InnerIter<D>) -> Self {
        Self { inner }
    }
}

impl<D> ErasedBundleNonNullPtrsIter<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[MaybeUninit<u8>]> {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner } = self;
        inner.offset()
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { inner, .. } = self;
        inner.descriptors()
    }
}

impl<'a, D> ErasedBundleNonNullPtrsIter<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleNonNullPtrsIter<FieldDescriptorsIter<'a, D>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleNonNullPtrsIter::from_inner(inner) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedBundleNonNullPtrsIter<D>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
{
    type Item = ErasedComponentNonNullPtr;
    type IntoIter = ErasedBundleNonNullPtrsIter<FieldDescriptorsIter<'a, D>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> Debug for ErasedBundleNonNullPtrsIter<D>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D> Clone for ErasedBundleNonNullPtrsIter<D>
where
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<D> Iterator for ErasedBundleNonNullPtrsIter<D>
where
    D: ErasedArchetypeIterator + ?Sized,
{
    type Item = ErasedComponentNonNullPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentNonNullPtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedBundleNonNullPtrsIter<D>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedBundleNonNullPtrsIter<D> where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized
{
}

impl<'a, D> FieldDescriptors<'a> for ErasedBundleNonNullPtrsIter<D>
where
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_descriptors()
    }
}

impl<D> CovariantFieldDescriptors for ErasedBundleNonNullPtrsIter<D>
where
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}
