use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::data::ErasedMutPtr;

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutRef, ErasedComponentPtr, ErasedComponentRef,
        error::{DowncastError, NotRegisteredError, check_downcast},
    },
    registry::{ComponentId, ComponentRegistry},
};

type Field = ErasedMutPtr<*mut MaybeUninit<u8>>;

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentMutPtr {
    component_id: ComponentId,
    field: Field,
}

impl ErasedComponentMutPtr {
    #[inline]
    pub fn dangling(
        registry: &ComponentRegistry,
        component_id: ComponentId,
    ) -> Result<Self, NotRegisteredError> {
        let component_info = registry
            .get_component_info(component_id)
            .ok_or(NotRegisteredError)?;

        let layout = component_info.descriptor().layout();
        let field = Field::dangling(layout)
            .expect("alignment of bytes should be sufficient for any component");

        let me = unsafe { Self::from_parts(component_id, field) };
        Ok(me)
    }

    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: *mut C,
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
    pub fn downcast<C>(self, registry: &ComponentRegistry) -> Result<*mut C, DowncastError<Self>>
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
    pub fn cast_const(self) -> ErasedComponentPtr {
        let Self {
            component_id,
            field,
        } = self;

        let field = field.cast_const();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedComponentRef<'a> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedComponentMutRef<'a> {
        unsafe { ErasedComponentMutRef::from_ptr(self) }
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
    pub unsafe fn offset_from(self, origin: ErasedComponentPtr) -> isize {
        let Self { field, .. } = self;

        let origin = origin.field();
        unsafe { field.offset_from(origin) }
    }

    #[inline]
    pub unsafe fn swap(self, with: Self) {
        let Self { field, .. } = self;

        let with = with.field();
        unsafe { field.swap(with) }
    }

    #[inline]
    pub unsafe fn copy_from(self, src: ErasedComponentPtr, count: usize) {
        let Self { field, .. } = self;

        let src = src.field();
        unsafe { field.copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(self, src: ErasedComponentPtr, count: usize) {
        let Self { field, .. } = self;

        let src = src.field();
        unsafe { field.copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub unsafe fn drop_in_place(
        self,
        registry: &ComponentRegistry,
    ) -> Result<(), NotRegisteredError> {
        let component_info = registry
            .get_component_info(self.component_id())
            .ok_or(NotRegisteredError)?;
        let Some(erased_drop) = component_info.erased_drop() else {
            return Ok(());
        };

        unsafe { erased_drop.drop_in_place(self) }
        Ok(())
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
    pub fn as_ptr(self) -> *const MaybeUninit<u8> {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(self) -> *mut MaybeUninit<u8> {
        let Self { field, .. } = self;
        field.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [MaybeUninit<u8>] {
        let Self { field, .. } = self;
        field.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(self) -> *mut [MaybeUninit<u8>] {
        let Self { field, .. } = self;
        field.as_mut_buffer()
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

impl PartialEq for ErasedComponentMutPtr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentMutPtr {}

impl PartialOrd for ErasedComponentMutPtr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentMutPtr {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentMutPtr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentMutPtr {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
