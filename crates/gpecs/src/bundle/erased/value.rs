use std::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::{FusedIterator, chain},
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaIntoFields,
    error::FromFieldsDescriptorsError,
    ptr::slice::CoreSliceItemPtrs,
    storage::{AllocError, BoxedAlignedUninitStorage},
};
use itertools::{equal, zip_eq};

use crate::{
    archetype::{
        collect::try_collect_components,
        erased::{ComponentIds, ErasedArchetype, FromComponentInfo},
        error::{AlreadyHasComponentError, MissingComponentError},
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundleMutRefsIter, ErasedBundlePtrs,
            ErasedBundleRefs, ErasedBundleRefsIter,
            error::{
                DowncastError, FromBundleError, FromComponentsError, InsertError, RemoveError,
                RemoveErrorKind, ReplaceError, ShuffleError,
            },
        },
    },
    component::{
        erased::{ErasedComponent, ErasedComponentMutRef, ErasedComponentRef, WithErasedDrop},
        registry::{
            ComponentId, ComponentRegistry,
            traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType},
        },
    },
    hash::IndexSet,
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
        traits::{RawSoaContext, ReadSoaContext},
    },
};

pub trait ErasedArchetypeKind:
    private::Sealed
    + for<'a> FieldDescriptors<'a, Output = &'a ErasedArchetype<Self::Meta>>
    + for<'a> IntoIterator<
        Item: AsRef<FieldDescriptor>,
        IntoIter: FieldDescriptors<
            'a,
            Output: IntoIterator<
                Item: AsRef<FieldDescriptor> + WithErasedDrop + Into<ComponentId>,
            > + Clone,
        >,
    >
{
    type Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static;
}

impl<Meta> ErasedArchetypeKind for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    type Meta = Meta;
}

impl<Meta> ErasedArchetypeKind for &ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    type Meta = Meta;
}

mod private {
    use super::ErasedArchetype;

    pub trait Sealed {}

    impl<Meta> Sealed for ErasedArchetype<Meta> {}
    impl<Meta> Sealed for &ErasedArchetype<Meta> {}
}

pub type ErasedBundle<Meta> = ErasedBundleKind<ErasedArchetype<Meta>>;
pub type ErasedBorrowedBundle<'a, Meta> = ErasedBundleKind<&'a ErasedArchetype<Meta>>;

pub struct ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    inner: Inner<T>,
}

type Inner<T> = ErasedSoa<BoxedAlignedUninitStorage, T, CoreSliceItemPtrs<MaybeUninit<u8>>>;

impl<Meta> ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + 'static,
{
    #[inline]
    pub fn try_from<B, M, T>(
        components: &mut ComponentRegistry<M, T>,
        bundle: B,
    ) -> Result<Self, FromBundleError<B>>
    where
        B: Bundle,
        Meta: FromComponentInfo<M>,
        M: FromComponentType,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let archetype = match ErasedArchetype::register::<B, M, T>(components) {
            Ok(archetype) => archetype,
            Err(reason) => return Err(FromBundleError::new(bundle, reason.into())),
        };
        let inner = Inner::try_from_descriptors_value::<B, _>(archetype, B::CONTEXT, bundle)
            .map_err(|error| {
                use gpecs_soa_erased::error::{
                    FromDescriptorsValueError, FromDescriptorsValueErrorKind::FromLayout,
                };

                let FromDescriptorsValueError { value, reason, .. } = error;
                match reason {
                    FromLayout(reason) => FromBundleError::new(value, reason.into()),
                    _ => unreachable!("{reason}"),
                }
            })?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

pub trait FromErasedComponent: Sized {
    fn from_erased_component(component: &ErasedComponent) -> Self;
}

impl<Meta> ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + FromErasedComponent + 'static,
{
    #[inline]
    pub fn from_components<I>(components: I) -> Result<Self, FromComponentsError>
    where
        I: IntoIterator<Item = ErasedComponent>,
    {
        let components =
            try_collect_components(components, IndexSet::insert, ErasedComponent::component_id)?;

        let iter = components.iter().map(|component| {
            let id = component.component_id();
            let meta = Meta::from_erased_component(component);
            (id, meta)
        });
        let archetype = unsafe { ErasedArchetype::with_meta_unchecked(iter) };

        let fields = components.into_iter().map(ErasedComponent::into_field);
        let inner = Inner::try_from_fields_descriptors(fields, archetype)
            .map_err::<FromComponentsError, _>(|error| match error {
                FromFieldsDescriptorsError::FromLayout(error) => error.into(),
                FromFieldsDescriptorsError::InvalidLayout(error) => error.into(),
                _ => unreachable!("failed to create erased bundle from components: {error}"),
            })?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

impl<T> ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    #[inline]
    pub unsafe fn from_inner(inner: Inner<T>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn downcast<B, U>(
        self,
        registry: &ComponentRegistry<impl Sized, U>,
    ) -> Result<B, DowncastError<Self>>
    where
        B: Bundle,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let src = match self.as_ptrs().downcast::<B, U>(registry) {
            Ok(src) => src,
            Err(reason) => return Err(DowncastError::new(self, reason)),
        };

        let bundle = unsafe { B::CONTEXT.read(src) };
        let _ = self.into_inner();
        Ok(bundle)
    }

    #[inline]
    pub fn downcast_ref<B, U>(
        &self,
        registry: &ComponentRegistry<impl Sized, U>,
    ) -> Result<BundleRefs<'_, B>, DowncastError<&Self>>
    where
        B: Bundle,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        self.as_refs()
            .downcast::<B, U>(registry)
            .map_err(|reason| DowncastError::new(self, reason))
    }

    #[inline]
    pub fn downcast_mut<B, U>(
        &mut self,
        registry: &ComponentRegistry<impl Sized, U>,
    ) -> Result<BundleRefsMut<'_, B>, DowncastError<&mut Self>>
    where
        B: Bundle,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        unsafe { self.as_mut_ptrs().deref_mut() }
            .downcast::<B, U>(registry)
            .map_err(|reason| DowncastError::new(self, reason))
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { inner } = self;
        inner.layout()
    }

    #[inline]
    pub fn archetype(&self) -> &ErasedArchetype<T::Meta> {
        let Self { inner } = self;
        inner.field_descriptors()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const MaybeUninit<u8> {
        let Self { inner } = self;
        inner.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut MaybeUninit<u8> {
        let Self { inner } = self;
        inner.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<u8>] {
        let Self { inner } = self;
        inner.as_mut_buffer()
    }

    #[inline]
    pub fn as_ptrs(&self) -> ErasedBundlePtrs<'_, T::Meta> {
        let (ptrs, _) = self.as_ptrs_with_archetype();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_archetype(
        &self,
    ) -> (ErasedBundlePtrs<'_, T::Meta>, &ErasedArchetype<T::Meta>) {
        let Self { inner } = self;

        let (inner, descriptors) = inner.as_ptrs_with_descriptors();
        let ptrs = unsafe { ErasedBundlePtrs::from_inner(inner) };
        (ptrs, descriptors.field_descriptors())
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> ErasedBundleMutPtrs<'_, T::Meta> {
        let (ptrs, _) = self.as_mut_ptrs_with_archetype();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_archetype(
        &mut self,
    ) -> (ErasedBundleMutPtrs<'_, T::Meta>, &ErasedArchetype<T::Meta>) {
        let Self { inner } = self;

        let (inner, descriptors) = inner.as_mut_ptrs_with_descriptors();
        let ptrs = unsafe { ErasedBundleMutPtrs::from_inner(inner) };
        (ptrs, descriptors.field_descriptors())
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedBundleRefs<'_, '_, T::Meta> {
        let (refs, _) = self.as_refs_with_archetype();
        refs
    }

    #[inline]
    pub fn as_refs_with_archetype(
        &self,
    ) -> (ErasedBundleRefs<'_, '_, T::Meta>, &ErasedArchetype<T::Meta>) {
        let (ptrs, descriptors) = self.as_ptrs_with_archetype();
        let refs = unsafe { ptrs.deref() };
        (refs, descriptors)
    }

    #[inline]
    pub fn as_mut_refs(&mut self) -> ErasedBundleMutRefs<'_, '_, T::Meta> {
        let (refs, _) = self.as_mut_refs_with_archetype();
        refs
    }

    #[inline]
    pub fn as_mut_refs_with_archetype(
        &mut self,
    ) -> (
        ErasedBundleMutRefs<'_, '_, T::Meta>,
        &ErasedArchetype<T::Meta>,
    ) {
        let (ptrs, descriptors) = self.as_mut_ptrs_with_archetype();
        let refs = unsafe { ptrs.deref_mut() };
        (refs, descriptors)
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleRefsIter<'_, '_, T::Meta> {
        self.as_refs().into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutRefsIter<'_, '_, T::Meta> {
        self.as_mut_refs().into_iter()
    }

    #[inline]
    pub fn into_inner(self) -> Inner<T> {
        let me = ManuallyDrop::new(self);
        unsafe { ptr::read(&raw const me.inner) }
    }
}

#[derive(Debug)]
pub enum ShuffledBundle<Original, Other>
where
    Original: ErasedArchetypeKind,
    Other: ErasedArchetypeKind<Meta = Original::Meta>,
{
    Original(ErasedBundleKind<Original>),
    Other(ErasedBundleKind<Other>),
}

impl<Original> ErasedBundleKind<Original>
where
    Original: ErasedArchetypeKind,
{
    #[inline]
    pub fn shuffle<Other>(
        self,
        archetype: Other,
    ) -> Result<ShuffledBundle<Original, Other>, ShuffleError<Self, Other>>
    where
        Other: ErasedArchetypeKind<Meta = Original::Meta>,
    {
        let this = self.archetype();
        let other = archetype.field_descriptors();
        if let Err(error) = this.check_exact_compatibility(other) {
            let error = ShuffleError {
                bundle: self,
                archetype,
                reason: error.into(),
            };
            return Err(error);
        }

        if equal(
            this.iter().map(ComponentId::from),
            other.iter().map(ComponentId::from),
        ) {
            let shuffled = ShuffledBundle::Original(self);
            return Ok(shuffled);
        }

        let refs = self.as_refs();
        let fields = other.iter().map(|component| {
            let component_id = component.into();
            refs.get(component_id).expect("component should be present")
        });
        let result = Inner::try_from_fields_descriptors(fields, other);

        let result = result.map_err(|error| match error {
            FromFieldsDescriptorsError::FromLayout(error) => error.into(),
            FromFieldsDescriptorsError::InvalidLayout(error) => error.into(),
            _ => unreachable!("failed to shuffle bundle: {error}"),
        });
        let inner = match result {
            Ok(inner) => inner,
            Err(reason) => {
                let error = ShuffleError {
                    bundle: self,
                    archetype,
                    reason,
                };
                return Err(error);
            }
        };
        let _ = self.into_inner();

        let (storage, _) = inner.into_parts();
        let inner = unsafe { ErasedSoa::from_parts(storage, archetype) };
        let other = unsafe { ErasedBundleKind::from_inner(inner) };

        let shuffled = ShuffledBundle::Other(other);
        Ok(shuffled)
    }
}

impl<T> ErasedBundleKind<T>
where
    T: ErasedArchetypeKind<Meta: Clone>,
{
    #[inline]
    // FIXME: can we optimize this?
    pub fn insert<ToInsert>(
        self,
        to_insert: ErasedBundleKind<ToInsert>,
    ) -> Result<ErasedBundle<T::Meta>, InsertError<Self, ErasedBundleKind<ToInsert>>>
    where
        ToInsert: ErasedArchetypeKind<Meta = T::Meta>,
    {
        if let Some(has_component_id) = to_insert
            .archetype()
            .component_ids()
            .find(|&id| self.archetype().contains(id))
        {
            let error = InsertError {
                reason: AlreadyHasComponentError::new(has_component_id).into(),
                bundle: self,
                to_insert,
            };
            return Err(error);
        }

        let refs = chain(self.as_refs(), to_insert.as_refs());
        let with_meta = chain(self.archetype(), to_insert.archetype())
            .map(|component| (component.id, component.meta.clone()));
        let archetype = unsafe { ErasedArchetype::with_meta_unchecked(with_meta) };
        let result = Inner::try_from_fields_descriptors(refs, archetype);

        let result = result.map_err(|error| match error {
            FromFieldsDescriptorsError::FromLayout(error) => error.into(),
            FromFieldsDescriptorsError::InvalidLayout(error) => error.into(),
            _ => unreachable!("failed to insert some components into bundle: {error}"),
        });
        let inner = match result {
            Ok(inner) => inner,
            Err(reason) => {
                let error = InsertError {
                    reason,
                    bundle: self,
                    to_insert,
                };
                return Err(error);
            }
        };

        let _ = (self.into_inner(), to_insert.into_inner());
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }

    #[inline]
    // FIXME: can we optimize this?
    pub fn replace<ToReplace>(
        mut self,
        to_replace: ErasedBundleKind<ToReplace>,
    ) -> Result<ErasedBundle<T::Meta>, ReplaceError<Self, ErasedBundleKind<ToReplace>>>
    where
        ToReplace: ErasedArchetypeKind<Meta = T::Meta>,
    {
        let (ptrs, archetype) = self.as_mut_ptrs_with_archetype();

        let ptrs = zip_eq(ptrs, archetype).map(|(dst, component)| {
            if to_replace.archetype().contains(dst.component_id()) {
                if let Some(erased_drop) = component.meta.erased_drop() {
                    unsafe { erased_drop.drop_in_place(dst) }
                }
                let src = to_replace
                    .as_ptrs()
                    .get(dst.component_id())
                    .expect("to replace archetype should contain component");
                unsafe { dst.copy_from_nonoverlapping(src, 1) }
            }
            dst.cast_const()
        });
        let ptrs_to_append = to_replace
            .as_ptrs()
            .into_iter()
            .filter(|ptr| !archetype.contains(ptr.component_id()));
        let refs = chain(ptrs, ptrs_to_append).map(|ptr| unsafe { ptr.deref() });

        let metas_to_append = to_replace
            .archetype()
            .iter()
            .filter(|component| !archetype.contains(component.id));
        let with_metas = chain(archetype, metas_to_append)
            .map(|component| (component.id, component.meta.clone()));
        let archetype = unsafe { ErasedArchetype::with_meta_unchecked(with_metas) };

        let result = Inner::try_from_fields_descriptors(refs, archetype);
        let result = result.map_err(|error| match error {
            FromFieldsDescriptorsError::FromLayout(error) => error.into(),
            FromFieldsDescriptorsError::InvalidLayout(error) => error.into(),
            _ => unreachable!("failed to replace some components in bundle: {error}"),
        });
        let inner = match result {
            Ok(inner) => inner,
            Err(reason) => {
                let error = ReplaceError {
                    reason,
                    bundle: self,
                    to_replace,
                };
                return Err(error);
            }
        };

        let _ = (self.into_inner(), to_replace.into_inner());
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }
}

pub struct RemovePair<ToRemove>
where
    ToRemove: ErasedArchetypeKind,
{
    pub retained: ErasedBundle<ToRemove::Meta>,
    pub removed: ErasedBundleKind<ToRemove>,
}

impl<T> ErasedBundleKind<T>
where
    T: ErasedArchetypeKind<Meta: Clone>,
{
    #[inline]
    // FIXME: can we optimize this?
    pub fn remove<ToRemove>(
        self,
        to_remove: ToRemove,
    ) -> Result<RemovePair<ToRemove>, RemoveError<Self>>
    where
        ToRemove: ErasedArchetypeKind<Meta = T::Meta>,
    {
        let archetype_to_remove = to_remove.field_descriptors();
        let bundle = self.check_remove(archetype_to_remove.component_ids())?;

        let retained_refs = bundle
            .as_refs()
            .into_iter()
            .filter(|component| !archetype_to_remove.contains(component.component_id()));
        let retained_meta = bundle
            .archetype()
            .iter()
            .filter(|component| !archetype_to_remove.contains(component.id))
            .map(|component| (component.id, component.meta.clone()));
        let retained_archetype = unsafe { ErasedArchetype::with_meta_unchecked(retained_meta) };
        let result = Inner::try_from_fields_descriptors(retained_refs, retained_archetype);
        let retained_inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(reason) => {
                let error = RemoveError { reason, bundle };
                return Err(error);
            }
        };

        let removed_refs = bundle
            .as_refs()
            .into_iter()
            .filter(|component| archetype_to_remove.contains(component.component_id()));
        let result = Inner::try_from_fields_descriptors(removed_refs, archetype_to_remove);
        let removed_inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(reason) => {
                let error = RemoveError { reason, bundle };
                return Err(error);
            }
        };
        let (removed_storage, _) = removed_inner.into_parts();
        let removed_inner = unsafe { Inner::from_parts(removed_storage, to_remove) };

        let _ = bundle.into_inner();
        let pair = RemovePair {
            retained: unsafe { ErasedBundleKind::from_inner(retained_inner) },
            removed: unsafe { ErasedBundleKind::from_inner(removed_inner) },
        };
        Ok(pair)
    }

    #[inline]
    // FIXME: can we optimize this?
    pub fn destroy(
        self,
        to_destroy: &ErasedArchetype<impl Sized>,
    ) -> Result<ErasedBundle<T::Meta>, RemoveError<Self>> {
        let mut bundle = self.check_remove(to_destroy.component_ids())?;

        let (refs, archetype) = bundle.as_mut_refs_with_archetype();
        let fields = zip_eq(refs, archetype).filter_map(|(mut field, component)| {
            if to_destroy.contains(component.id) {
                let to_drop = field.as_mut_component_ptr();
                if let Some(erased_drop) = component.meta.erased_drop() {
                    unsafe { erased_drop.drop_in_place(to_drop) }
                }
                return None;
            }
            Some(field)
        });
        let with_meta = archetype.iter().filter_map(|component| {
            if to_destroy.contains(component.id) {
                return None;
            }
            Some((component.id, component.meta.clone()))
        });
        let archetype = unsafe { ErasedArchetype::with_meta_unchecked(with_meta) };
        let result = Inner::try_from_fields_descriptors(fields, archetype);
        let inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(reason) => {
                let error = RemoveError { reason, bundle };
                return Err(error);
            }
        };

        let _ = bundle.into_inner();
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }

    #[inline]
    fn check_remove(self, mut to_remove: ComponentIds<'_>) -> Result<Self, RemoveError<Self>> {
        if let Some(missing_component_id) = to_remove.find(|&id| !self.archetype().contains(id)) {
            let error = RemoveError {
                reason: MissingComponentError::new(missing_component_id).into(),
                bundle: self,
            };
            return Err(error);
        }
        Ok(self)
    }
}

#[inline]
#[expect(clippy::needless_pass_by_value)]
fn into_remove_error_kind(error: FromFieldsDescriptorsError<AllocError>) -> RemoveErrorKind {
    match error {
        FromFieldsDescriptorsError::FromLayout(error) => error.into(),
        _ => unreachable!("failed to remove some components of bundle: {error}"),
    }
}

impl<'a, Meta> From<ErasedBorrowedBundle<'a, Meta>> for ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop + Clone + 'static,
{
    #[inline]
    fn from(bundle: ErasedBorrowedBundle<'a, Meta>) -> Self {
        let (storage, archetype) = bundle.into_inner().into_parts();
        let archetype = archetype.clone();

        let inner = unsafe { Inner::from_parts(storage, archetype) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<T> Debug for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let components = &self.into_iter();
        f.debug_struct("ErasedBundle")
            .field("components", components)
            .finish()
    }
}

impl<T> AsRef<[MaybeUninit<u8>]> for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<u8>] {
        self.as_buffer()
    }
}

impl<T> AsRef<ErasedArchetype<T::Meta>> for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    #[inline]
    fn as_ref(&self) -> &ErasedArchetype<T::Meta> {
        self.archetype()
    }
}

impl<T> Drop for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    fn drop(&mut self) {
        let (ptrs, archetype) = self.as_mut_ptrs_with_archetype();
        unsafe { archetype.ptrs_drop_in_place(ptrs) }
    }
}

impl<'a, T> IntoIterator for &'a ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    type Item = ErasedComponentRef<'a>;
    type IntoIter = ErasedBundleRefsIter<'a, 'a, T::Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    type Item = ErasedComponentMutRef<'a>;
    type IntoIter = ErasedBundleMutRefsIter<'a, 'a, T::Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    type Item = Result<ErasedComponent, AllocError>;
    type IntoIter = ErasedBundleIntoIterKind<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let inner = self.into_inner();

        let inner = inner.into_iter();
        ErasedBundleIntoIterKind { inner }
    }
}

impl<'a, T> FieldDescriptors<'a> for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    type Output = &'a ErasedArchetype<T::Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.archetype()
    }
}

impl<T> CovariantFieldDescriptors for ErasedBundleKind<T>
where
    T: ErasedArchetypeKind,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

pub type ErasedBundleIntoIter<Meta> = ErasedBundleIntoIterKind<ErasedArchetype<Meta>>;
pub type ErasedBorrowedBundleIntoIter<'a, Meta> =
    ErasedBundleIntoIterKind<&'a ErasedArchetype<Meta>>;

pub struct ErasedBundleIntoIterKind<T>
where
    T: ErasedArchetypeKind,
{
    inner: InnerIter<T::IntoIter>,
}

type InnerIter<T> = ErasedSoaIntoFields<
    BoxedAlignedUninitStorage,
    T,
    BoxedAlignedUninitStorage,
    CoreSliceItemPtrs<MaybeUninit<u8>>,
>;

impl<T> Iterator for ErasedBundleIntoIterKind<T>
where
    T: ErasedArchetypeKind,
{
    type Item = Result<ErasedComponent, AllocError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        use gpecs_soa_erased::data::error::FromLayoutDataError::FromLayout;

        let Self { inner } = self;

        let component = inner.field_descriptors().clone().into_iter().next()?;
        let erased_drop = component.erased_drop();
        let id = component.into();

        let item = match inner.next()? {
            Ok(field) => {
                let component = unsafe { ErasedComponent::from_parts(id, field, erased_drop) };
                Ok(component)
            }
            Err(error) => match error {
                FromLayout(error) => Err(error),
                _ => unreachable!("failed to create erased data: {error}"),
            },
        };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<T> ExactSizeIterator for ErasedBundleIntoIterKind<T>
where
    T: ErasedArchetypeKind,
    T::IntoIter: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<T> FusedIterator for ErasedBundleIntoIterKind<T>
where
    T: ErasedArchetypeKind,
    T::IntoIter: FusedIterator,
{
}
