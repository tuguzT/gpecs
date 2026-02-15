use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::data::ErasedSlicePtr;

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutSlicePtr, ErasedComponentPtr, ErasedComponentSlice,
        error::{DowncastError, check_downcast},
    },
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

type Fields = ErasedSlicePtr<*const MaybeUninit<u8>>;

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentSlicePtr {
    component_id: ComponentId,
    fields: Fields,
}

impl ErasedComponentSlicePtr {
    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: *const [C],
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
    pub fn downcast<C>(
        self,
        registry: &ComponentRegistry,
    ) -> Result<*const [C], DowncastError<Self>>
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
    pub fn cast_mut(self) -> ErasedComponentMutSlicePtr {
        let Self {
            component_id,
            fields,
        } = self;

        let fields = fields.cast_mut();
        unsafe { ErasedComponentMutSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedComponentSlice<'a> {
        unsafe { ErasedComponentSlice::from_ptr(self) }
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
    pub fn component_ptr(self) -> ErasedComponentPtr {
        let Self {
            component_id,
            fields,
        } = self;

        let field = fields.field_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
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
    pub fn as_ptr(self) -> *const u8 {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [u8] {
        let Self { fields, .. } = self;
        fields.as_buffer()
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

impl PartialEq for ErasedComponentSlicePtr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentSlicePtr {}

impl PartialOrd for ErasedComponentSlicePtr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentSlicePtr {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentSlicePtr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentSlicePtr {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}
