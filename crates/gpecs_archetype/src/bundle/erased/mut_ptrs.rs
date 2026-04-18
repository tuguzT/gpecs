use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::{
        ErasedComponentMutPtr, ErasedComponentPtr, WithErasedDrop, error::NotRegisteredError,
    },
    registry::{ComponentId, ComponentRegistryView, traits::WithComponentId},
};
use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter,
    ptr::slice::{CastConst, MutSliceItemPtr},
    soa::field::{
        FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput, FieldDescriptorsOwned,
    },
    storage::AlignedStorage,
};

use crate::{
    bundle::erased::{
        ErasedBundleKind, ErasedBundleMutRefs, ErasedBundlePtrs, ErasedBundlePtrsIter,
        ErasedBundleRefs,
        traits::{
            ErasedArchetypeIterator, ErasedArchetypeKind, ErasedBundleDrop,
            IntoErasedArchetypeIterator,
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleMutPtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutPtrs<D, P>,
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaMutPtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaMutPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn cast_const(self) -> ErasedBundlePtrs<D, CastConst<P>> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedBundleRefs<'a, D, CastConst<P>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedBundleMutRefs<'a, D, P> {
        unsafe { ErasedBundleMutRefs::from_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { inner } = self;

        let inner = unsafe { inner.add(count) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn as_inner(&self) -> &ErasedSoaMutPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub unsafe fn as_mut_inner(&mut self) -> &mut ErasedSoaMutPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> *mut [P::Item] {
        let Self { inner } = self;
        inner.as_mut_buffer()
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

impl<'a, D, P> ErasedBundleMutPtrs<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'n, N>(
        &'a self,
        origin: &'n ErasedBundlePtrs<N, CastConst<P>>,
    ) -> isize
    where
        N: FieldDescriptors<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let origin = unsafe { origin.as_inner() };
        unsafe { inner.offset_from(origin) }
    }

    #[inline]
    pub fn iter(&'a self) -> ErasedBundlePtrsIter<FieldDescriptorsIter<'a, D>, CastConst<P>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundlePtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedBundleMutPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<'n, N>(&'a mut self, with: &'n mut ErasedBundleMutPtrs<N, P>)
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
        src: &'n ErasedBundlePtrs<N, CastConst<P>>,
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
        src: &'n ErasedBundlePtrs<N, CastConst<P>>,
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
        src: &'n ErasedBundlePtrs<N, CastConst<P>>,
        count: usize,
    ) where
        N: FieldDescriptors<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_nonoverlapping(src, count) }
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn drop_in_place(
        &mut self,
        components: &ComponentRegistryView<impl WithErasedDrop, impl ?Sized>,
    ) -> Result<(), NotRegisteredError> {
        self.iter()
            .map(ErasedComponentPtr::component_id)
            .try_for_each(|id| {
                components
                    .get_component_info(id)
                    .map(drop)
                    .ok_or_else(NotRegisteredError::new)
            })?;

        self.iter_mut().for_each(|ptr| {
            if let Err(error) = unsafe { ptr.drop_in_place(components) } {
                unreachable!("{error}, but it was checked earlier to be registered")
            }
        });
        Ok(())
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_descriptors()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentPtr<CastConst<P>>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutPtr<P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    // TODO: shuffle components if needed?..
    pub unsafe fn write<T, K, S>(&mut self, value: ErasedBundleKind<T, K, S, P::Ptrs>)
    where
        T: ErasedArchetypeKind,
        K: ErasedBundleDrop<T::Meta>,
        S: AlignedStorage<Item = P::Item>,
    {
        let Self { inner } = self;

        let value = value.into_inner();
        unsafe { inner.write(value) }
    }
}

impl<D, P> Debug for ErasedBundleMutPtrs<D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleMutPtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleMutPtrs<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> Copy for ErasedBundleMutPtrs<D, P>
where
    D: Copy,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleMutPtrs<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentPtr<CastConst<P>>;
    type IntoIter = ErasedBundlePtrsIter<FieldDescriptorsIter<'a, D>, CastConst<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedBundleMutPtrs<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutPtr<P>;
    type IntoIter = ErasedBundleMutPtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, P> IntoIterator for ErasedBundleMutPtrs<D, P>
where
    D: IntoErasedArchetypeIterator,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutPtr<P>;
    type IntoIter = ErasedBundleMutPtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedBundleMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedBundleMutPtrs<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedBundleMutPtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutPtrsIter<D, P>,
}

impl<D, P> ErasedBundleMutPtrsIter<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaMutPtrsIter<D, P>) -> Self {
        Self { inner }
    }
}

impl<D, P> ErasedBundleMutPtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> *mut [P::Item] {
        let Self { inner } = self;
        inner.as_mut_buffer()
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

impl<'a, D, P> ErasedBundleMutPtrsIter<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleMutPtrsIter<D, P>
where
    D: FieldDescriptors<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutPtr<P>;
    type IntoIter = ErasedBundleMutPtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedBundleMutPtrsIter<D, P>
where
    D: FieldDescriptorsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedBundleMutPtrsIter<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<D, P> Iterator for ErasedBundleMutPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_descriptors().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutPtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedBundleMutPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D, P> FusedIterator for ErasedBundleMutPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedBundleMutPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.descriptors().field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedBundleMutPtrsIter<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}
