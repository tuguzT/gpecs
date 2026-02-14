use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::field::ErasedFieldRef;

use crate::component::{
    Component,
    erased::{
        ErasedComponentPtr,
        error::{DowncastError, check_downcast},
    },
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

type Field<'a> = ErasedFieldRef<'a, *const MaybeUninit<u8>>;

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentRef<'a> {
    component_id: ComponentId,
    field: Field<'a>,
}

impl<'a> ErasedComponentRef<'a> {
    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: &'a C,
    ) -> Result<Self, NotRegisteredError>
    where
        C: Component,
    {
        let component_id = registry.component_id::<C>().ok_or(NotRegisteredError)?;
        let field = Field::try_from(component)
            .expect("alignment of bytes should be sufficient for any component");

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, field: Field<'a>) -> Self {
        Self {
            component_id,
            field,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentPtr) -> Self {
        let (component_id, field) = ptr.into_parts();
        let field = unsafe { Field::from_ptr(field) };
        unsafe { Self::from_parts(component_id, field) }
    }

    #[inline]
    pub fn downcast<C>(self, registry: &ComponentRegistry) -> Result<&'a C, DowncastError<Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = self;
        let Self { field, .. } = check_downcast::<C, _>(registry, component_id, self)?;

        let component = unsafe { field.downcast::<C>() }
            .expect("descriptors of input component and self should be equal");
        Ok(component)
    }

    #[inline]
    pub fn downcast_ref<C>(&self, registry: &ComponentRegistry) -> Result<&C, DowncastError<&Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = *self;
        let Self { field, .. } = check_downcast::<C, _>(registry, component_id, self)?;

        let component = unsafe { field.downcast_ref::<C>() }
            .expect("descriptors of input component and self should be equal");
        Ok(component)
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn field(&self) -> &Field<'a> {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn as_component_ptr(&self) -> ErasedComponentPtr {
        let Self {
            ref field,
            component_id,
        } = *self;

        let field = field.as_field_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[u8] {
        let Self { field, .. } = self;
        field.as_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Field<'a>) {
        let Self {
            component_id,
            field,
        } = self;
        (component_id, field)
    }
}

impl PartialEq for ErasedComponentRef<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentRef<'_> {}

impl PartialOrd for ErasedComponentRef<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentRef<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentRef<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentRef<'_> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl AsRef<[u8]> for ErasedComponentRef<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_buffer()
    }
}
