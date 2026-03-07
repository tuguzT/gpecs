use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::data::ErasedMutSlicePtr;

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentMutSlice, ErasedComponentSlice,
        ErasedComponentSlicePtr,
        error::{DowncastError, check_downcast},
    },
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

type Fields = ErasedMutSlicePtr<*mut MaybeUninit<u8>>;

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentMutSlicePtr {
    component_id: ComponentId,
    fields: Fields,
}

impl ErasedComponentMutSlicePtr {
    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: *mut [C],
    ) -> Result<Self, NotRegisteredError>
    where
        C: Component,
    {
        let component_id = registry.component_id::<C>().ok_or(NotRegisteredError)?;
        let fields = Fields::try_from(component)
            .expect("alignment of bytes should be sufficient for any component");

        let me = unsafe { Self::from_parts(component_id, fields) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, fields: Fields) -> Self {
        Self {
            component_id,
            fields,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentMutPtr, len: usize) -> Self {
        let (component_id, field) = ptr.into_parts();
        let fields = unsafe { Fields::from_parts(field, len) };
        unsafe { Self::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn downcast<C>(self, registry: &ComponentRegistry) -> Result<*mut [C], DowncastError<Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = self;
        let Self { fields, .. } = check_downcast::<C, _>(registry, component_id, self)?;

        let component = fields
            .downcast()
            .expect("descriptors of input component and self should be equal");
        Ok(component)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedComponentSlicePtr {
        let Self {
            component_id,
            fields,
        } = self;

        let fields = fields.cast_const();
        unsafe { ErasedComponentSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedComponentSlice<'a> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedComponentMutSlice<'a> {
        unsafe { ErasedComponentMutSlice::from_ptr(self) }
    }

    #[inline]
    pub unsafe fn drop_in_place(
        self,
        registry: &ComponentRegistry,
    ) -> Result<(), NotRegisteredError> {
        let Self {
            component_id,
            fields,
        } = self;

        let component_info = registry
            .get_component_info(component_id)
            .ok_or(NotRegisteredError)?;
        let Some(drop_fn) = component_info.drop_fn() else {
            return Ok(());
        };

        for i in 0..fields.len() {
            let field = unsafe { fields.field_ptr().add(i) };
            let ptr = field.as_mut_ptr().cast();
            unsafe { drop_fn(ptr) }
        }
        Ok(())
    }

    #[inline]
    pub fn component_id(self) -> ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }

    #[inline]
    pub fn fields(self) -> Fields {
        let Self { fields, .. } = self;
        fields
    }

    #[inline]
    pub fn component_ptr(self) -> ErasedComponentMutPtr {
        let Self {
            component_id,
            fields,
        } = self;

        let field = fields.field_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn len(self) -> usize {
        let Self { fields, .. } = self;
        fields.len()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_ptr(self) -> *const MaybeUninit<u8> {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(self) -> *mut MaybeUninit<u8> {
        let Self { fields, .. } = self;
        fields.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [MaybeUninit<u8>] {
        let Self { fields, .. } = self;
        fields.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(self) -> *mut [MaybeUninit<u8>] {
        let Self { fields, .. } = self;
        fields.as_mut_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Fields) {
        let Self {
            component_id,
            fields,
        } = self;
        (component_id, fields)
    }
}

impl PartialEq for ErasedComponentMutSlicePtr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentMutSlicePtr {}

impl PartialOrd for ErasedComponentMutSlicePtr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentMutSlicePtr {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentMutSlicePtr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentMutSlicePtr {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
