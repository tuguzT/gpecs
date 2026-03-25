use core::ptr;
use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::{ManuallyDrop, MaybeUninit},
};

use gpecs_soa_erased::{data::BoxedErased, ptr::slice::CoreSliceItemPtrs};

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentMutRef, ErasedComponentPtr, ErasedComponentRef,
        ErasedDrop,
        error::{DowncastError, FromComponentError, NotRegisteredError, check_downcast},
    },
    registry::{ComponentId, ComponentRegistry},
};

type Field = BoxedErased<CoreSliceItemPtrs<MaybeUninit<u8>>>;

#[derive(Debug)]
pub struct ErasedComponent {
    component_id: ComponentId,
    field: Field,
    erased_drop: Option<ErasedDrop>,
}

impl ErasedComponent {
    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: C,
    ) -> Result<Self, FromComponentError<C>>
    where
        C: Component,
    {
        let Some(component_id) = registry.component_id::<C>() else {
            let reason = NotRegisteredError.into();
            return Err(FromComponentError::new(component, reason));
        };

        let field = Field::try_from(component).map_err(|error| {
            use gpecs_soa_erased::data::error::{
                FromValueError,
                FromValueErrorKind::{FromLayout, InsufficientAlign},
            };

            let FromValueError { value, reason, .. } = error;
            match reason {
                FromLayout(error) => FromComponentError::new(value, error.into()),
                InsufficientAlign(error) => {
                    unreachable!("byte alignment should be sufficient for any component: {error:?}")
                }
            }
        })?;

        let Some(component_info) = registry.get_component_info(component_id) else {
            unreachable!("{component_id} should be registered")
        };
        let erased_drop = component_info.erased_drop();

        let me = unsafe { Self::from_parts(component_id, field, erased_drop) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        component_id: ComponentId,
        field: Field,
        erased_drop: Option<ErasedDrop>,
    ) -> Self {
        Self {
            component_id,
            field,
            erased_drop,
        }
    }

    #[inline]
    pub fn downcast<C>(self, registry: &ComponentRegistry) -> Result<C, DowncastError<Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = self;
        let (_, field, _) = check_downcast::<C, _>(registry, component_id, self)?.into_parts();

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
    pub fn as_field(&self) -> &Field {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub unsafe fn as_mut_field(&mut self) -> &mut Field {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn erased_drop(&self) -> Option<ErasedDrop> {
        let Self { erased_drop, .. } = *self;
        erased_drop
    }

    #[inline]
    pub fn as_erased_component_ptr(&self) -> ErasedComponentPtr {
        let Self {
            ref field,
            component_id,
            ..
        } = *self;

        let field = field.as_erased_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_mut_erased_component_ptr(&mut self) -> ErasedComponentMutPtr {
        let Self {
            ref mut field,
            component_id,
            ..
        } = *self;

        let field = field.as_mut_erased_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_erased_component(&self) -> ErasedComponentRef<'_> {
        unsafe { self.as_erased_component_ptr().deref() }
    }

    #[inline]
    pub fn as_mut_erased_component(&mut self) -> ErasedComponentMutRef<'_> {
        unsafe { self.as_mut_erased_component_ptr().deref_mut() }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { field, .. } = self;
        field.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { field, .. } = self;
        field.as_mut_ptr()
    }

    #[inline]
    pub fn as_slice(&self) -> &[MaybeUninit<u8>] {
        let Self { field, .. } = self;
        field.as_slice()
    }

    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [MaybeUninit<u8>] {
        let Self { field, .. } = self;
        field.as_mut_slice()
    }

    #[inline]
    pub fn into_field(self) -> Field {
        let me = ManuallyDrop::new(self);
        unsafe { ptr::read(&raw const me.field) }
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Field, Option<ErasedDrop>) {
        let Self {
            component_id,
            erased_drop,
            ..
        } = self;

        let field = self.into_field();
        (component_id, field, erased_drop)
    }
}

impl PartialEq for ErasedComponent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponent {}

impl PartialOrd for ErasedComponent {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponent {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponent {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponent {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl AsRef<[MaybeUninit<u8>]> for ErasedComponent {
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<u8>] {
        self.as_slice()
    }
}

impl Drop for ErasedComponent {
    #[inline]
    fn drop(&mut self) {
        let Some(drop) = self.erased_drop() else {
            return;
        };

        let to_drop = self.as_mut_erased_component_ptr();
        unsafe { drop.drop_in_place(to_drop) }
    }
}
