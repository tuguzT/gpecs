use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::Index,
    ptr,
};

use gpecs_component::registry::{
    ComponentId, ComponentInfo, ComponentRegistryView,
    traits::{ComponentIdFrom, FromComponentType},
};
use gpecs_soa_erased::CovariantFieldLayouts;
use gpecs_sparse::{
    error::FromPartsError,
    item::{KeyValueSlices, SparseItem},
    soa::{
        field::{FieldLayouts, FieldLayoutsOutput},
        identity::{AsIdentitySlice, Identity, IdentitySlice},
        layout::WithLayout,
        slice::SoaSlices,
    },
    view::{EpochSparseView, EpochSparseViewPtr},
};

use crate::{
    bundle::Bundle,
    erased::{
        ComponentIdOrderedIter, ComponentIds, Iter,
        error::{
            AlreadyHasComponentError, IncompatibleArchetypeError, IncompatibleArchetypeExactError,
            IncompatibleArchetypeViewExactError, MissingComponentError, TooFewComponentsError,
        },
    },
};

type Inner<'a, Meta> = EpochSparseViewPtr<'a, u32, Identity<Meta>>;

#[repr(transparent)]
pub struct ErasedArchetypeView<'a, Meta>
where
    Meta: 'a,
{
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> ErasedArchetypeView<'a, Meta> {
    const CONTEXT: &'a () = &();

    #[inline]
    pub fn new(
        component_ids: &'a [ComponentId],
        metas: &'a [Meta],
        sparse: &'a [SparseItem<u32>],
    ) -> Result<Self, FromPartsError<u32>> {
        let context = Self::CONTEXT;
        let keys = component_ids_to_u32s(component_ids);
        let values = metas.as_identity_slice();
        let dense = SoaSlices::new(
            Identity::from_inner_ref(context),
            KeyValueSlices::new(context, keys, values),
        );

        let inner = EpochSparseView::new(dense, sparse)?.into_view_ptr();
        let me = Self::from_inner(inner);
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        component_ids: &'a [ComponentId],
        metas: &'a [Meta],
        sparse: &'a [SparseItem<u32>],
    ) -> Self {
        let context = Self::CONTEXT;
        let keys = component_ids_to_u32s(component_ids);
        let values = metas.as_identity_slice();
        let dense = unsafe {
            SoaSlices::new(
                Identity::from_inner_ref(context),
                KeyValueSlices::new_unchecked(keys, values),
            )
        };

        let inner = unsafe { EpochSparseView::from_parts(dense, sparse) }.into_view_ptr();
        Self::from_inner(inner)
    }

    #[inline]
    pub(crate) fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(crate) fn into_inner(self) -> Inner<'a, Meta> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_parts(self) -> (&'a [ComponentId], &'a [Meta], &'a [SparseItem<u32>]) {
        let Self { inner } = self;

        let (dense, sparse) = inner.into_slice_ptrs();
        let sparse = unsafe { sparse.as_ref_unchecked() };

        let context = Self::CONTEXT;
        let (keys, values) = unsafe { dense.as_ref_unchecked(context).into_parts() };

        let component_ids = unsafe { u32s_to_component_ids(keys) };
        let metas = values.as_inner();
        (component_ids, metas, sparse)
    }

    #[inline]
    pub fn as_view(&self) -> ErasedArchetypeView<'_, Meta> {
        let Self { inner } = *self;
        ErasedArchetypeView::from_inner(inner)
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
    pub fn as_slices(&self) -> (&[ComponentId], &[Meta], &[SparseItem<u32>]) {
        let Self { inner } = self;

        let (dense, sparse) = inner.as_slice_ptrs();
        let sparse = unsafe { sparse.as_ref_unchecked() };

        let context = Self::CONTEXT;
        let (keys, values) = unsafe { dense.as_ref_unchecked(context).into_parts() };

        let component_ids = unsafe { u32s_to_component_ids(keys) };
        let metas = values.as_inner();
        (component_ids, metas, sparse)
    }

    #[inline]
    pub fn as_component_ids(&self) -> &[ComponentId] {
        let (component_ids, _, _) = self.as_slices();
        component_ids
    }

    #[inline]
    pub fn as_metas(&self) -> &[Meta] {
        let (_, metas, _) = self.as_slices();
        metas
    }

    #[inline]
    pub fn as_sparse(&self) -> &[SparseItem<u32>] {
        let (_, _, sparse) = self.as_slices();
        sparse
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const ComponentId, *const Meta, *const SparseItem<u32>) {
        let (component_ids, metas, sparse) = self.as_slices();
        (component_ids.as_ptr(), metas.as_ptr(), sparse.as_ptr())
    }

    #[inline]
    pub fn contains(&self, component_id: ComponentId) -> bool {
        let Self { inner } = self;

        let key = component_id.into_u32();
        unsafe { inner.as_ref_unchecked() }.contains_key(key)
    }

    #[inline]
    pub fn has_components(
        &self,
        of: ErasedArchetypeView<impl Sized>,
    ) -> Result<(), MissingComponentError> {
        let of = of.component_ids();
        self.has_components_trusted(of)
    }

    #[inline]
    fn has_components_trusted(
        &self,
        of: impl IntoIterator<Item = ComponentId>,
    ) -> Result<(), MissingComponentError> {
        if let Some(id) = of.into_iter().find(|&id| !self.contains(id)) {
            let error = MissingComponentError::new(id);
            return Err(error);
        }
        Ok(())
    }

    #[inline]
    pub fn has_no_components(
        &self,
        of: ErasedArchetypeView<impl Sized>,
    ) -> Result<(), AlreadyHasComponentError> {
        let of = of.component_ids();
        self.has_no_components_trusted(of)
    }

    #[inline]
    fn has_no_components_trusted(
        &self,
        of: impl IntoIterator<Item = ComponentId>,
    ) -> Result<(), AlreadyHasComponentError> {
        if let Some(id) = of.into_iter().find(|&id| self.contains(id)) {
            let error = AlreadyHasComponentError::new(id);
            return Err(error);
        }
        Ok(())
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<&Meta> {
        self.into_get(component_id)
    }

    #[inline]
    pub fn into_get(self, component_id: ComponentId) -> Option<&'a Meta> {
        let Self { inner } = self;

        let key = component_id.into_u32();
        unsafe { inner.as_ref_unchecked() }
            .into_get(key)
            .map(Identity::as_inner)
    }

    #[inline]
    pub fn into_index(self, component_id: ComponentId) -> &'a Meta {
        let Self { inner } = self;

        let key = component_id.into_u32();
        unsafe { inner.as_ref_unchecked() }.into_index(key)
    }

    #[inline]
    pub fn get_index_of(&self, component_id: ComponentId) -> Option<usize> {
        let Self { inner } = self;

        let index: usize = component_id.into_u32().try_into().ok()?;
        let sparse_item = unsafe { inner.as_ref_unchecked() }
            .into_sparse_slice()
            .get(index)?;
        let index_of = sparse_item.dense_index().copied()?;
        index_of.try_into().ok()
    }

    #[inline]
    pub fn get_by_index(&self, index: usize) -> Option<(ComponentId, &Meta)> {
        self.into_get_by_index(index)
    }

    #[inline]
    pub fn into_get_by_index(self, index: usize) -> Option<(ComponentId, &'a Meta)> {
        let Self { inner } = self;

        let index = index.try_into().ok()?;
        let (id, meta) = unsafe { inner.as_ref_unchecked() }.into_get_with_key(index)?;

        let id = unsafe { ComponentId::from_u32(id) };
        Some((id, meta))
    }

    #[inline]
    pub fn check_compatibility(
        &self,
        other: ErasedArchetypeView<impl Sized>,
    ) -> Result<(), MissingComponentError> {
        self.has_components(other)
    }

    #[inline]
    pub fn check_compatibility_of<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        B: Bundle,
    {
        let of = B::get_components(components)?;
        self.has_components_trusted(of)?;
        Ok(())
    }

    #[inline]
    pub fn check_exact_compatibility(
        &self,
        other: ErasedArchetypeView<impl Sized>,
    ) -> Result<(), IncompatibleArchetypeViewExactError> {
        self.check_compatibility(other)?;

        if self.len() != other.len() {
            return Err(TooFewComponentsError.into());
        }
        Ok(())
    }

    #[inline]
    pub fn check_exact_compatibility_of<B>(
        &self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        B: Bundle,
    {
        let of = B::get_components(components)?.into_iter();
        let len = of.len();
        self.has_components_trusted(of)?;

        if self.len() != len {
            return Err(TooFewComponentsError.into());
        }
        Ok(())
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Meta> {
        (*self).into_iter()
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds<'_> {
        self.into_component_ids()
    }

    #[inline]
    pub fn component_id_ordered_iter(&self) -> ComponentIdOrderedIter<'_, Meta> {
        self.into_component_id_ordered_iter()
    }

    #[inline]
    pub fn into_component_ids(self) -> ComponentIds<'a> {
        let Self { inner } = self;

        let inner = unsafe { inner.as_ref_unchecked() }.into_key_slice().iter();
        ComponentIds::from_inner(inner)
    }

    #[inline]
    pub fn into_component_id_ordered_iter(self) -> ComponentIdOrderedIter<'a, Meta> {
        let (_, dense, sparse) = self.into_parts();
        ComponentIdOrderedIter::from_inner(dense, sparse)
    }
}

impl<Meta> Debug for ErasedArchetypeView<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (component_ids, metas, sparse) = self.as_slices();
        f.debug_struct("ErasedArchetypeView")
            .field("component_ids", &component_ids)
            .field("metas", &metas)
            .field("sparse", &sparse)
            .finish()
    }
}

impl<Meta> Default for ErasedArchetypeView<'_, Meta> {
    fn default() -> Self {
        let inner = Inner::from(Self::CONTEXT);
        Self::from_inner(inner)
    }
}

impl<Meta> Clone for ErasedArchetypeView<'_, Meta> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Meta> Copy for ErasedArchetypeView<'_, Meta> {}

impl<Meta> PartialEq for ErasedArchetypeView<'_, Meta>
where
    Meta: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
}

impl<Meta> Eq for ErasedArchetypeView<'_, Meta> where Meta: Eq {}

impl<Meta> PartialOrd for ErasedArchetypeView<'_, Meta>
where
    Meta: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let slices = self.as_slices();
        let other = other.as_slices();
        slices.partial_cmp(&other)
    }
}

impl<Meta> Ord for ErasedArchetypeView<'_, Meta>
where
    Meta: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let slices = self.as_slices();
        let other = other.as_slices();
        slices.cmp(&other)
    }
}

impl<Meta> Hash for ErasedArchetypeView<'_, Meta>
where
    Meta: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slices().hash(state);
    }
}

impl<Meta> AsRef<[ComponentId]> for ErasedArchetypeView<'_, Meta> {
    #[inline]
    fn as_ref(&self) -> &[ComponentId] {
        self.as_component_ids()
    }
}

impl<Meta> Index<ComponentId> for ErasedArchetypeView<'_, Meta> {
    type Output = Meta;

    #[inline]
    fn index(&self, component_id: ComponentId) -> &Self::Output {
        self.into_index(component_id)
    }
}

impl<'a, Meta> IntoIterator for &'a ErasedArchetypeView<'_, Meta> {
    type Item = ComponentInfo<&'a Meta>;
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta> IntoIterator for ErasedArchetypeView<'a, Meta> {
    type Item = ComponentInfo<&'a Meta>;
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        Iter::from_inner(inner)
    }
}

unsafe impl<Meta> Send for ErasedArchetypeView<'_, Meta> where Meta: Sync {}
unsafe impl<Meta> Sync for ErasedArchetypeView<'_, Meta> where Meta: Sync {}

impl<'a, Meta> FieldLayouts<'a> for ErasedArchetypeView<'_, Meta>
where
    Meta: WithLayout + 'a,
{
    type Output = ErasedArchetypeView<'a, Meta>;
    type OutputIter = Iter<'a, Meta>;
    type OutputItem = ComponentInfo<&'a Meta>;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        *self
    }
}

impl<Meta> CovariantFieldLayouts for ErasedArchetypeView<'_, Meta>
where
    Meta: WithLayout + 'static,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}

#[inline]
fn component_ids_to_u32s(component_ids: &[ComponentId]) -> &[u32] {
    let u32s = ptr::from_ref(component_ids) as *const [_];
    unsafe { u32s.as_ref_unchecked() }
}

#[inline]
unsafe fn u32s_to_component_ids(keys: &[u32]) -> &[ComponentId] {
    let component_ids = ptr::from_ref(keys) as *const [_];
    unsafe { component_ids.as_ref_unchecked() }
}
