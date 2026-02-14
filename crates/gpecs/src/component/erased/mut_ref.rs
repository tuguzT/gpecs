use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::field::ErasedFieldRefMut;

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentPtr, ErasedComponentRef,
        error::{DowncastError, check_downcast},
    },
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

type Field<'a> = ErasedFieldRefMut<'a, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedComponentRefMut<'a> {
    component_id: ComponentId,
    field: Field<'a>,
}

impl<'a> ErasedComponentRefMut<'a> {
    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: &'a mut C,
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
    pub unsafe fn from_ptr(ptr: ErasedComponentMutPtr) -> Self {
        let (component_id, field) = ptr.into_parts();
        let field = unsafe { Field::from_ptr(field) };
        unsafe { Self::from_parts(component_id, field) }
    }

    #[inline]
    pub fn downcast<C>(self, registry: &ComponentRegistry) -> Result<&'a mut C, DowncastError<Self>>
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
    pub fn downcast_mut<C>(
        &mut self,
        registry: &ComponentRegistry,
    ) -> Result<&mut C, DowncastError<&mut Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = *self;
        let Self { field, .. } = check_downcast::<C, _>(registry, component_id, self)?;

        let component = unsafe { field.downcast_mut::<C>() }
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
    pub fn as_mut_component_ptr(&mut self) -> ErasedComponentMutPtr {
        let Self {
            ref mut field,
            component_id,
        } = *self;

        let field = field.as_mut_field_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { field, .. } = self;
        field.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[u8] {
        let Self { field, .. } = self;
        field.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [u8] {
        let Self { field, .. } = self;
        field.as_mut_buffer()
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

impl PartialEq for ErasedComponentRefMut<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentRefMut<'_> {}

impl PartialOrd for ErasedComponentRefMut<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentRefMut<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentRefMut<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentRefMut<'_> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl AsRef<[u8]> for ErasedComponentRefMut<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_buffer()
    }
}

impl AsMut<[u8]> for ErasedComponentRefMut<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_buffer()
    }
}

impl<'a> From<ErasedComponentRefMut<'a>> for ErasedComponentRef<'a> {
    #[inline]
    fn from(value: ErasedComponentRefMut<'a>) -> Self {
        let (component_id, field) = value.into_parts();
        let field = field.into();
        unsafe { Self::from_parts(component_id, field) }
    }
}
