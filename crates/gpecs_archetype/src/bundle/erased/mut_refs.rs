use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::{ErasedComponentMutRef, ErasedComponentRef},
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType, WithComponentId},
    },
};
use gpecs_soa_erased::{
    BufferOffsetsFrom, BufferOffsetsFromSelf, BufferOffsetsOf, CovariantFieldLayouts,
    ErasedSoaMutRefs, ErasedSoaMutRefsIter,
    ptr::slice::{CastConst, MutSliceItemPtr},
    soa::{
        field::{FieldLayouts, FieldLayoutsItem, FieldLayoutsOutput, FieldLayoutsOwned},
        traits::SoaContext,
    },
};

use crate::{
    bundle::{
        Bundle, BundleRefsMut,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleRefs, ErasedBundleRefsIter,
            error::DowncastError,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleMutRefs<'a, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutRefs<'a, D, P>,
}

impl<'a, D, P> ErasedBundleMutRefs<'a, D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaMutRefs<'a, D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutPtrs<D, P>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.as_mut_unchecked() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaMutRefs<'a, D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleMutPtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleMutRefs<'_, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [P::Item] {
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
    pub fn layouts(&self) -> &D {
        let Self { inner } = self;
        inner.layouts()
    }
}

impl<'a, D, P> ErasedBundleMutRefs<'_, D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf, OutputIter: ErasedArchetypeIterator>
        + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedBundleRefsIter<'a, D::OutputIter, CastConst<P>, BufferOffsetsOf<D::OutputItem>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleRefsIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(
        &'a mut self,
    ) -> ErasedBundleMutRefsIter<'a, D::OutputIter, P, BufferOffsetsOf<D::OutputItem>> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutRefsIter::from_inner(inner) }
    }
}

impl<'a, D, P> ErasedBundleMutRefs<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleRefsMut<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B>(components)
            .map_err(|error| error.map_value(into_self))?;

        let refs = unsafe { B::CONTEXT.mut_ptrs_to_mut_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, P> ErasedBundleMutRefs<'_, D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentRef<'_, CastConst<P>>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutRef<'_, P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleMutRefs<'_, D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleMutRefs")
            .field("inner", &inner)
            .finish()
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleMutRefs<'_, D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf, OutputIter: ErasedArchetypeIterator>
        + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentRef<'a, CastConst<P>>;
    type IntoIter =
        ErasedBundleRefsIter<'a, D::OutputIter, CastConst<P>, BufferOffsetsOf<D::OutputItem>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedBundleMutRefs<'_, D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf, OutputIter: ErasedArchetypeIterator>
        + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutRef<'a, P>;
    type IntoIter = ErasedBundleMutRefsIter<'a, D::OutputIter, P, BufferOffsetsOf<D::OutputItem>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D, P> IntoIterator for ErasedBundleMutRefs<'a, D, P>
where
    D: IntoErasedArchetypeIterator<Item: BufferOffsetsFromSelf>,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutRef<'a, P>;
    type IntoIter = ErasedBundleMutRefsIter<'a, D::IntoIter, P, BufferOffsetsOf<D::Item>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutRefsIter::from_inner(inner) }
    }
}

impl<'a, D, P> From<ErasedBundleMutRefs<'a, D, P>> for ErasedBundleRefs<'a, D, CastConst<P>>
where
    P: MutSliceItemPtr,
{
    #[inline]
    fn from(refs: ErasedBundleMutRefs<'a, D, P>) -> Self {
        let inner = refs.into_inner();
        let inner = inner.into();
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleMutRefs<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleMutRefs<'_, D, P>
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

pub struct ErasedBundleMutRefsIter<'a, D, P, F>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutRefsIter<'a, D, P, F>,
}

impl<'a, D, P, F> ErasedBundleMutRefsIter<'a, D, P, F>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaMutRefsIter<'a, D, P, F>) -> Self {
        Self { inner }
    }
}

impl<D, P, F> ErasedBundleMutRefsIter<'_, D, P, F>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [P::Item] {
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
    pub fn layouts(&self) -> &D {
        let Self { inner, .. } = self;
        inner.layouts()
    }
}

impl<'a, D, P, F> ErasedBundleMutRefsIter<'_, D, P, F>
where
    D: FieldLayouts<'a, OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutRefsIter<'a, D::OutputIter, P, F> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutRefsIter::from_inner(inner) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedBundleMutRefsIter<'_, D, P, F>
where
    D: FieldLayouts<'a, OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    type Item = ErasedComponentMutRef<'a, P>;
    type IntoIter = ErasedBundleMutRefsIter<'a, D::OutputIter, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedBundleMutRefsIter<'_, D, P, F>
where
    D: FieldLayoutsOwned<OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr<Item: Debug>,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<'a, D, P, F> Iterator for ErasedBundleMutRefsIter<'a, D, P, F>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedComponentMutRef<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutRef::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedBundleMutRefsIter<'_, D, P, F>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D, P, F> FusedIterator for ErasedBundleMutRefsIter<'_, D, P, F>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedBundleMutRefsIter<'_, D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P, F> CovariantFieldLayouts for ErasedBundleMutRefsIter<'_, D, P, F>
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
