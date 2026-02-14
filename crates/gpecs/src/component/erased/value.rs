use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::{field::BoxedErasedField, slice_item_ptr::CoreSliceItemPtrs};

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentPtr, ErasedComponentRef, ErasedComponentRefMut,
        error::{DowncastError, FromComponentError, check_downcast},
    },
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

type Field = BoxedErasedField<CoreSliceItemPtrs<MaybeUninit<u8>>>;

#[derive(Debug)]
pub struct ErasedComponent {
    component_id: ComponentId,
    field: Field,
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
            use gpecs_soa_erased::field::error::{
                FromValueError,
                FromValueErrorKind::{FromLayout, InsufficientAlign},
            };

            let FromValueError { value, reason, .. } = error;
            match reason {
                FromLayout(error) => FromComponentError::new(value, error.into()),
                InsufficientAlign(error) => {
                    panic!("alignment of bytes should be sufficient for any component: {error:?}")
                }
            }
        })?;

        let me = unsafe { Self::from_parts(component_id, field) };
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
    pub fn downcast<C>(self, registry: &ComponentRegistry) -> Result<C, DowncastError<Self>>
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
    pub fn drop_in_place_consume(
        mut self,
        registry: &ComponentRegistry,
    ) -> Result<(), (Self, NotRegisteredError)> {
        if let Err(error) = unsafe { self.drop_in_place(registry) } {
            return Err((self, error));
        }
        Ok(())
    }

    #[inline]
    pub unsafe fn drop_in_place(
        &mut self,
        registry: &ComponentRegistry,
    ) -> Result<(), NotRegisteredError> {
        let ptr = self.as_mut_erased_component_ptr();
        unsafe { ptr.drop_in_place(registry) }
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn field(&self) -> &Field {
        let Self { field, .. } = self;
        field
    }

    #[inline]
    pub fn as_erased_component_ptr(&self) -> ErasedComponentPtr {
        let Self {
            ref field,
            component_id,
        } = *self;

        let field = field.as_field_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_mut_erased_component_ptr(&mut self) -> ErasedComponentMutPtr {
        let Self {
            ref mut field,
            component_id,
        } = *self;

        let field = field.as_mut_field_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_erased_component(&self) -> ErasedComponentRef<'_> {
        unsafe { self.as_erased_component_ptr().deref() }
    }

    #[inline]
    pub fn as_mut_erased_component(&mut self) -> ErasedComponentRefMut<'_> {
        unsafe { self.as_mut_erased_component_ptr().deref_mut() }
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
    pub fn as_slice(&self) -> &[u8] {
        let Self { field, .. } = self;
        field.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        let Self { field, .. } = self;
        field.as_mut_slice()
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

impl AsRef<[u8]> for ErasedComponent {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for ErasedComponent {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}
