use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::{ErasedComponentMutSlicePtr, ErasedComponentSlicePtr, error::NotRegisteredError},
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType, WithComponentId},
    },
};
use gpecs_soa_erased::{
    CovariantFieldLayouts, ErasedSoaMutSlicePtrs, ErasedSoaMutSlicePtrsIter,
    ptr::slice::{CastConst, MutSliceItemPtr},
    soa::{
        field::{FieldLayouts, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned},
        traits::RawSoaContext,
    },
};

use crate::{
    bundle::{
        Bundle, BundleSliceMutPtrs,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleMutSlices, ErasedBundleSlicePtrs,
            ErasedBundleSlicePtrsIter, ErasedBundleSlices,
            error::DowncastError,
            traits::{
                ErasedArchetypeIterator, ErasedArchetypeKind, ErasedBundleDrop,
                IntoErasedArchetypeIterator,
            },
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleMutSlicePtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutSlicePtrs<D, P>,
}

impl<D, P> ErasedBundleMutSlicePtrs<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaMutSlicePtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutPtrs<D, P>, len: usize) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { ErasedSoaMutSlicePtrs::from_ptrs(inner, len) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaMutSlicePtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleMutPtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedBundleSlicePtrs<D, CastConst<P>> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        unsafe { ErasedBundleSlicePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedBundleSlices<'a, D, CastConst<P>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedBundleMutSlices<'a, D, P> {
        unsafe { ErasedBundleMutSlices::from_ptrs(self) }
    }
}

impl<D, P> ErasedBundleMutSlicePtrs<D, P>
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
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner } = self;
        inner.layouts()
    }
}

impl<'a, D, P> ErasedBundleMutSlicePtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicePtrsIter<FieldLayoutsIter<'a, D>, CastConst<P>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedBundleMutSlicePtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutSlicePtrsIter::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleMutSlicePtrs<D, P>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn drop_in_place<M, K>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
    ) -> Result<(), NotRegisteredError>
    where
        K: ErasedBundleDrop<M>,
    {
        self.iter()
            .map(ErasedComponentSlicePtr::component_id)
            .try_for_each(|id| {
                components
                    .get_component_descriptor(id)
                    .map(drop)
                    .ok_or_else(NotRegisteredError::new)
            })?;

        self.iter_mut().for_each(|to_drop| {
            let component_id = to_drop.component_id();
            let info = components
                .get_component_descriptor(component_id)
                .expect("component info should exist");
            unsafe { K::drop_in_place_slice_with(to_drop, info) }
        });
        Ok(())
    }
}

impl<D, P> ErasedBundleMutSlicePtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSliceMutPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let len = self.len();
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = B::CONTEXT.mut_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D, P> ErasedBundleMutSlicePtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlicePtr<CastConst<P>>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutSlicePtr<P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleMutSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleMutSlicePtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleMutSlicePtrs<D, P>
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

impl<D, P> Copy for ErasedBundleMutSlicePtrs<D, P>
where
    D: Copy,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleMutSlicePtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentSlicePtr<CastConst<P>>;
    type IntoIter = ErasedBundleSlicePtrsIter<FieldLayoutsIter<'a, D>, CastConst<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedBundleMutSlicePtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutSlicePtr<P>;
    type IntoIter = ErasedBundleMutSlicePtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, P> IntoIterator for ErasedBundleMutSlicePtrs<D, P>
where
    D: IntoErasedArchetypeIterator,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutSlicePtr<P>;
    type IntoIter = ErasedBundleMutSlicePtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleMutSlicePtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleMutSlicePtrs<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedBundleMutSlicePtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutSlicePtrsIter<D, P>,
}

impl<D, P> ErasedBundleMutSlicePtrsIter<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaMutSlicePtrsIter<D, P>) -> Self {
        Self { inner }
    }
}

impl<D, P> ErasedBundleMutSlicePtrsIter<D, P>
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
    pub fn slice_len(&self) -> usize {
        let Self { inner } = self;
        inner.slice_len()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner, .. } = self;
        inner.layouts()
    }
}

impl<'a, D, P> ErasedBundleMutSlicePtrsIter<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutSlicePtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleMutSlicePtrsIter<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutSlicePtr<P>;
    type IntoIter = ErasedBundleMutSlicePtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedBundleMutSlicePtrsIter<D, P>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedBundleMutSlicePtrsIter<D, P>
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

impl<D, P> Iterator for ErasedBundleMutSlicePtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutSlicePtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutSlicePtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedBundleMutSlicePtrsIter<D, P>
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

impl<D, P> FusedIterator for ErasedBundleMutSlicePtrsIter<D, P>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleMutSlicePtrsIter<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleMutSlicePtrsIter<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
