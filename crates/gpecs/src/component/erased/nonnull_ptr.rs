use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
    ptr::NonNull,
};

use gpecs_soa_erased::data::ErasedNonNullPtr;

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutPtr,
        error::{DowncastError, NotRegisteredError, check_downcast},
    },
    registry::{ComponentId, ComponentRegistry},
};

type Field = ErasedNonNullPtr<NonNull<MaybeUninit<u8>>>;

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentNonNullPtr {
    component_id: ComponentId,
    field: Field,
}

impl ErasedComponentNonNullPtr {
    #[inline]
    pub fn new(ptr: ErasedComponentMutPtr) -> Option<Self> {
        let (component_id, field) = ptr.into_parts();
        let field = Field::new(field)?;

        let me = unsafe { Self::from_parts(component_id, field) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: ErasedComponentMutPtr) -> Self {
        let (component_id, field) = ptr.into_parts();
        let field = unsafe { Field::new_unchecked(field) };

        unsafe { Self::from_parts(component_id, field) }
    }

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
        component: NonNull<C>,
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
    pub fn downcast<C>(
        self,
        registry: &ComponentRegistry,
    ) -> Result<NonNull<C>, DowncastError<Self>>
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
    pub unsafe fn swap(self, with: Self) {
        let Self { field, .. } = self;

        let with = with.field();
        unsafe { field.swap(with) }
    }

    #[inline]
    pub unsafe fn copy_from(self, src: Self, count: usize) {
        let Self { field, .. } = self;

        let src = src.field();
        unsafe { field.copy_from(src, count) }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(self, src: Self, count: usize) {
        let Self { field, .. } = self;

        let src = src.field();
        unsafe { field.copy_from_nonoverlapping(src, count) }
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
    pub fn as_ptr(self) -> NonNull<MaybeUninit<u8>> {
        let Self { field, .. } = self;
        field.as_ptr()
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

impl PartialEq for ErasedComponentNonNullPtr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentNonNullPtr {}

impl PartialOrd for ErasedComponentNonNullPtr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentNonNullPtr {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentNonNullPtr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentNonNullPtr {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl From<ErasedComponentNonNullPtr> for ErasedComponentMutPtr {
    #[inline]
    fn from(ptr: ErasedComponentNonNullPtr) -> Self {
        let (component_id, field) = ptr.into_parts();
        let field = field.into();
        unsafe { Self::from_parts(component_id, field) }
    }
}
