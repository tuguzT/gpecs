use std::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

use gpecs_soa_erased::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaIntoFields,
    ptr::slice::CoreSliceItemPtrs,
    storage::{AllocError, BoxedAlignedUninitStorage},
};
use itertools::{equal, zip_eq};

use crate::{
    archetype::{
        collect::try_collect_components,
        erased::{ErasedArchetype, FromComponentInfo},
    },
    bundle::{
        Bundle, BundleRefs, BundleRefsMut,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundleMutRefsIter, ErasedBundlePtrs,
            ErasedBundleRefs, ErasedBundleRefsIter,
            error::{DowncastError, FromBundleError, FromComponentsError, ShuffleError},
        },
    },
    component::{
        erased::{ErasedComponent, ErasedComponentMutRef, ErasedComponentRef},
        registry::{ComponentId, ComponentRegistry, DropFn},
    },
    hash::IndexSet,
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        traits::ReadSoaContext,
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
                Item: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + Into<ComponentId>,
            > + Clone,
        >,
    >
{
    type Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>;
}

impl<Meta> ErasedArchetypeKind for ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    type Meta = Meta;
}

impl<Meta> ErasedArchetypeKind for &ErasedArchetype<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
{
    type Meta = Meta;
}

mod private {
    use super::ErasedArchetype;

    pub trait Sealed {}

    impl<Meta> Sealed for ErasedArchetype<Meta> {}
    impl<Meta> Sealed for &ErasedArchetype<Meta> {}
}

pub type ErasedBorrowedBundle<'a, Meta> = ErasedBundle<Meta, &'a ErasedArchetype<Meta>>;

pub struct ErasedBundle<Meta, Archetype = ErasedArchetype<Meta>>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    inner: Inner<Archetype>,
}

type Inner<Archetype> =
    ErasedSoa<BoxedAlignedUninitStorage, Archetype, CoreSliceItemPtrs<MaybeUninit<u8>>>;

impl<Meta> ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + FromComponentInfo + 'static,
{
    #[inline]
    pub fn try_from<B>(
        registry: &mut ComponentRegistry,
        bundle: B,
    ) -> Result<Self, FromBundleError<B>>
    where
        B: Bundle,
    {
        let archetype = match ErasedArchetype::of::<B>(registry) {
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

impl<Meta> ErasedBundle<Meta>
where
    Meta: AsRef<FieldDescriptor>
        + AsRef<Option<DropFn>>
        + for<'a> From<&'a ErasedComponent>
        + 'static,
{
    #[inline]
    pub fn from_components<I>(components: I) -> Result<Self, FromComponentsError>
    where
        I: IntoIterator<Item = ErasedComponent>,
    {
        let components =
            try_collect_components(components, IndexSet::insert, ErasedComponent::component_id)?;

        let iter = components
            .iter()
            .map(|component| (component.component_id(), Meta::from(component)));
        let archetype = unsafe { ErasedArchetype::with_meta_unchecked(iter) };

        let fields = components.into_iter().map(ErasedComponent::into_field);
        let inner = Inner::try_from_fields_descriptors(fields, archetype).map_err(|error| {
            use gpecs_soa_erased::error::FromFieldsDescriptorsError::{FromLayout, InvalidLayout};

            match error {
                InvalidLayout(error) => FromComponentsError::from(error),
                FromLayout(error) => FromComponentsError::from(error),
                _ => unreachable!("failed to create erased bundle from components: {error}"),
            }
        })?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

impl<Meta, Archetype> ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    #[inline]
    pub unsafe fn from_inner(inner: Inner<Archetype>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn downcast<B>(self, registry: &ComponentRegistry) -> Result<B, DowncastError<Self>>
    where
        B: Bundle,
    {
        let src = match self.as_ptrs().downcast::<B>(registry) {
            Ok(src) => src,
            Err(reason) => return Err(DowncastError::new(self, reason)),
        };

        let bundle = unsafe { B::CONTEXT.read(src) };
        let _ = self.into_inner();
        Ok(bundle)
    }

    #[inline]
    pub fn downcast_ref<B>(
        &self,
        registry: &ComponentRegistry,
    ) -> Result<BundleRefs<'_, B>, DowncastError<&Self>>
    where
        B: Bundle,
    {
        self.as_refs()
            .downcast::<B>(registry)
            .map_err(|reason| DowncastError::new(self, reason))
    }

    #[inline]
    pub fn downcast_mut<B>(
        &mut self,
        registry: &ComponentRegistry,
    ) -> Result<BundleRefsMut<'_, B>, DowncastError<&mut Self>>
    where
        B: Bundle,
    {
        unsafe { self.as_mut_ptrs().deref_mut() }
            .downcast::<B>(registry)
            .map_err(|reason| DowncastError::new(self, reason))
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { inner } = self;
        inner.layout()
    }

    #[inline]
    pub fn archetype(&self) -> &ErasedArchetype<Meta> {
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
    pub fn as_ptrs(&self) -> ErasedBundlePtrs<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.as_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> ErasedBundleMutPtrs<'_, Meta> {
        let Self { inner } = self;

        let inner = inner.as_mut_ptrs();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedBundleRefs<'_, '_, Meta> {
        unsafe { self.as_ptrs().deref() }
    }

    #[inline]
    pub fn as_mut_refs(&mut self) -> ErasedBundleMutRefs<'_, '_, Meta> {
        unsafe { self.as_mut_ptrs().deref_mut() }
    }

    #[inline]
    pub fn iter(&self) -> ErasedBundleRefsIter<'_, '_, Meta> {
        self.as_refs().into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedBundleMutRefsIter<'_, '_, Meta> {
        self.as_mut_refs().into_iter()
    }

    #[inline]
    pub fn into_inner(self) -> Inner<Archetype> {
        let me = ManuallyDrop::new(self);
        unsafe { ptr::read(&raw const me.inner) }
    }
}

#[derive(Debug)]
pub enum ShuffledBundle<Meta, Original, Other>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Original: ErasedArchetypeKind<Meta = Meta>,
    Other: ErasedArchetypeKind<Meta = Meta>,
{
    Original(ErasedBundle<Meta, Original>),
    Other(ErasedBundle<Meta, Other>),
}

impl<Meta, Original> ErasedBundle<Meta, Original>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
    Original: ErasedArchetypeKind<Meta = Meta>,
{
    #[inline]
    pub fn shuffle<Other>(
        self,
        archetype: Other,
    ) -> Result<ShuffledBundle<Meta, Original, Other>, ShuffleError<Self, Other>>
    where
        Other: ErasedArchetypeKind<Meta = Meta>,
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

        let result = result.map_err(|error| {
            use gpecs_soa_erased::error::FromFieldsDescriptorsError::{FromLayout, InvalidLayout};

            match error {
                FromLayout(error) => error.into(),
                InvalidLayout(error) => error.into(),
                _ => unreachable!("failed to shuffle bundle: {error}"),
            }
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
        let other = unsafe { ErasedBundle::from_inner(inner) };

        let shuffled = ShuffledBundle::Other(other);
        Ok(shuffled)
    }
}

impl<Meta, Archetype> Debug for ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let components = &self.into_iter();
        f.debug_struct("ErasedBundle")
            .field("components", components)
            .finish()
    }
}

impl<Meta, Archetype> AsRef<[MaybeUninit<u8>]> for ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<u8>] {
        self.as_buffer()
    }
}

impl<Meta, Archetype> Drop for ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    fn drop(&mut self) {
        let Self { inner } = self;

        let ptrs = inner.as_mut_ptrs().into_iter();
        let components = ptrs.descriptors().clone();
        for (ptr, component) in zip_eq(ptrs, components) {
            let Some(drop_fn) = component.as_ref() else {
                continue;
            };
            unsafe { drop_fn(ptr.as_mut_ptr()) }
        }
    }
}

impl<'a, Meta, Archetype> IntoIterator for &'a ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    type Item = ErasedComponentRef<'a>;
    type IntoIter = ErasedBundleRefsIter<'a, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Meta, Archetype> IntoIterator for &'a mut ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    type Item = ErasedComponentMutRef<'a>;
    type IntoIter = ErasedBundleMutRefsIter<'a, 'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<Meta, Archetype> IntoIterator for ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>>,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    type Item = Result<ErasedComponent, AllocError>;
    type IntoIter = ErasedBundleIntoIter<Meta, Archetype>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let inner = self.into_inner();

        let inner = inner.into_iter();
        ErasedBundleIntoIter { inner }
    }
}

impl<'a, Meta, Archetype> FieldDescriptors<'a> for ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'a,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    type Output = &'a ErasedArchetype<Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.archetype()
    }
}

impl<Meta, Archetype> CovariantFieldDescriptors for ErasedBundle<Meta, Archetype>
where
    Meta: AsRef<FieldDescriptor> + AsRef<Option<DropFn>> + 'static,
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}

pub type ErasedBorrowedBundleIntoIter<'a, Meta> =
    ErasedBundleIntoIter<Meta, &'a ErasedArchetype<Meta>>;

pub struct ErasedBundleIntoIter<Meta, Archetype = ErasedArchetype<Meta>>
where
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    inner: InnerIter<Archetype::IntoIter>,
}

type InnerIter<Archetype> = ErasedSoaIntoFields<
    BoxedAlignedUninitStorage,
    Archetype,
    BoxedAlignedUninitStorage,
    CoreSliceItemPtrs<MaybeUninit<u8>>,
>;

impl<Meta, Archetype> Iterator for ErasedBundleIntoIter<Meta, Archetype>
where
    Archetype: ErasedArchetypeKind<Meta = Meta>,
{
    type Item = Result<ErasedComponent, AllocError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        use gpecs_soa_erased::data::error::FromLayoutDataError::FromLayout;

        let Self { inner } = self;

        let component = inner.field_descriptors().clone().into_iter().next()?;
        let &drop_fn = component.as_ref();
        let id = component.into();

        let item = match inner.next()? {
            Ok(field) => {
                let component = unsafe { ErasedComponent::from_parts(id, field, drop_fn) };
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

impl<Meta, Archetype> ExactSizeIterator for ErasedBundleIntoIter<Meta, Archetype>
where
    Archetype: ErasedArchetypeKind<Meta = Meta>,
    Archetype::IntoIter: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta, Archetype> FusedIterator for ErasedBundleIntoIter<Meta, Archetype>
where
    Archetype: ErasedArchetypeKind<Meta = Meta>,
    Archetype::IntoIter: FusedIterator,
{
}
