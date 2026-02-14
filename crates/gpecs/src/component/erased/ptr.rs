use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::field::ErasedFieldPtr;

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentRef,
        error::{DowncastError, check_downcast},
    },
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

type Field = ErasedFieldPtr<*const MaybeUninit<u8>>;

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentPtr {
    component_id: ComponentId,
    field: Field,
}

impl ErasedComponentPtr {
    #[inline]
    pub fn dangling(
        registry: &ComponentRegistry,
        component_id: ComponentId,
    ) -> Result<Self, NotRegisteredError> {
        let component_info = registry
            .get_component_info(component_id)
            .ok_or(NotRegisteredError)?;

        let desc = component_info.descriptor();
        let field = Field::dangling(desc)
            .expect("alignment of bytes should be sufficient for any component");

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: *const C,
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
    pub fn dangling_of<C>(registry: &ComponentRegistry) -> Result<Self, NotRegisteredError>
    where
        C: Component,
    {
        let component_id = registry.component_id::<C>().ok_or(NotRegisteredError)?;

        let me = Self::dangling(registry, component_id).expect("component should be registered");
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, field: Field) -> Self {
        Self {
            component_id,
            field,
        }
    }

    #[inline]
    pub fn downcast<C>(self, registry: &ComponentRegistry) -> Result<*const C, DowncastError<Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = self;
        let Self { field, .. } = check_downcast::<C, _>(registry, component_id, self)?;

        let component = field
            .downcast()
            .expect("descriptors of input component and self should be equal");
        Ok(component)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedComponentMutPtr {
        let Self {
            component_id,
            field,
        } = self;

        let field = field.cast_mut();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedComponentRef<'a> {
        unsafe { ErasedComponentRef::from_ptr(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self {
            component_id,
            field,
        } = self;

        let field = unsafe { field.add(count) };
        unsafe { Self::from_parts(component_id, field) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { field, .. } = self;

        let origin = origin.field();
        unsafe { field.offset_from(origin) }
    }

    #[inline]
    pub fn component_id(self) -> ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }

    #[inline]
    pub fn field(self) -> Field {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn as_ptr(self) -> *const u8 {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [u8] {
        let Self { field, .. } = self;
        field.as_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Field) {
        let Self {
            component_id,
            field,
        } = self;
        (component_id, field)
    }
}

impl PartialEq for ErasedComponentPtr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentPtr {}

impl PartialOrd for ErasedComponentPtr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentPtr {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentPtr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentPtr {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
