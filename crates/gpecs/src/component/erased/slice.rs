use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::data::ErasedSlice;

use crate::component::{
    Component,
    erased::{
        ErasedComponentPtr, ErasedComponentSlicePtr,
        error::{DowncastError, check_downcast},
    },
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

type Fields<'a> = ErasedSlice<'a, *const MaybeUninit<u8>>;

#[derive(Debug, Clone, Copy)]
pub struct ErasedComponentSlice<'a> {
    component_id: ComponentId,
    fields: Fields<'a>,
}

impl<'a> ErasedComponentSlice<'a> {
    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: &'a [C],
    ) -> Result<Self, NotRegisteredError>
    where
        C: Component,
    {
        let component_id = registry.component_id::<C>().ok_or(NotRegisteredError)?;
        let fields = Fields::try_from(component)
            .expect("alignment of bytes should be sufficient for any component");

        Ok(Self {
            component_id,
            fields,
        })
    }

    #[inline]
    pub unsafe fn from_parts(ptr: ErasedComponentPtr, len: usize) -> Self {
        let (component_id, field) = ptr.into_parts();
        let fields = unsafe { Fields::from_parts(field, len) };
        Self {
            component_id,
            fields,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentSlicePtr) -> Self {
        let (ptr, len) = ptr.into_parts();
        unsafe { Self::from_parts(ptr, len) }
    }

    #[inline]
    pub fn downcast<C>(self, registry: &ComponentRegistry) -> Result<&'a [C], DowncastError<Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = self;
        let Self { fields, .. } = check_downcast::<C, _>(registry, component_id, self)?;

        let component = unsafe { fields.downcast::<C>() }
            .expect("descriptors of input component and self should be equal");
        Ok(component)
    }

    #[inline]
    pub fn downcast_ref<C>(
        &self,
        registry: &ComponentRegistry,
    ) -> Result<&[C], DowncastError<&Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = *self;
        let Self { fields, .. } = check_downcast::<C, _>(registry, component_id, self)?;

        let component = unsafe { fields.downcast_ref::<C>() }
            .expect("descriptors of input component and self should be equal");
        Ok(component)
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn fields(&self) -> &Fields<'a> {
        let Self { fields, .. } = self;
        fields
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
    pub fn as_component_slice_ptr(&self) -> ErasedComponentSlicePtr {
        let Self {
            ref fields,
            component_id,
        } = *self;

        let field = fields.as_field_ptr();
        let ptr = unsafe { ErasedComponentPtr::from_parts(component_id, field) };

        let len = fields.len();
        unsafe { ErasedComponentSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub fn as_component_ptr(&self) -> ErasedComponentPtr {
        let Self {
            ref fields,
            component_id,
        } = *self;

        let field = fields.as_field_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[u8] {
        let Self { fields, .. } = self;
        fields.as_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedComponentPtr, usize) {
        let Self {
            component_id,
            fields,
        } = self;

        let (field, len) = fields.into_parts();
        let ptr = unsafe { ErasedComponentPtr::from_parts(component_id, field) };
        (ptr, len)
    }
}

impl PartialEq for ErasedComponentSlice<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentSlice<'_> {}

impl PartialOrd for ErasedComponentSlice<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentSlice<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentSlice<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentSlice<'_> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl AsRef<[u8]> for ErasedComponentSlice<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_buffer()
    }
}
