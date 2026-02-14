use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::field::ErasedFieldSliceMut;

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentPtr, ErasedComponentSliceMutPtr,
        ErasedComponentSlicePtr,
        error::{DowncastError, check_downcast},
    },
    error::NotRegisteredError,
    registry::{ComponentId, ComponentRegistry},
};

type Fields<'a> = ErasedFieldSliceMut<'a, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedComponentSliceMut<'a> {
    component_id: ComponentId,
    fields: Fields<'a>,
}

impl<'a> ErasedComponentSliceMut<'a> {
    #[inline]
    pub fn try_from<C>(
        registry: &ComponentRegistry,
        component: &'a mut [C],
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
    pub unsafe fn from_parts(ptr: ErasedComponentMutPtr, len: usize) -> Self {
        let (component_id, field) = ptr.into_parts();
        let fields = unsafe { Fields::from_parts(field, len) };
        Self {
            component_id,
            fields,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentSliceMutPtr) -> Self {
        let (ptr, len) = ptr.into_parts();
        unsafe { Self::from_parts(ptr, len) }
    }

    #[inline]
    pub fn downcast<C>(
        self,
        registry: &ComponentRegistry,
    ) -> Result<&'a mut [C], DowncastError<Self>>
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
    pub fn downcast_mut<C>(
        &mut self,
        registry: &ComponentRegistry,
    ) -> Result<&mut [C], DowncastError<&mut Self>>
    where
        C: Component,
    {
        let Self { component_id, .. } = *self;
        let Self { fields, .. } = check_downcast::<C, _>(registry, component_id, self)?;

        let component = unsafe { fields.downcast_mut::<C>() }
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
    pub fn as_mut_component_slice_ptr(&mut self) -> ErasedComponentSliceMutPtr {
        let Self {
            ref mut fields,
            component_id,
        } = *self;

        let field = fields.as_mut_field_ptr();
        let ptr = unsafe { ErasedComponentMutPtr::from_parts(component_id, field) };

        let len = fields.len();
        unsafe { ErasedComponentSliceMutPtr::from_parts(ptr, len) }
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
    pub fn as_mut_component_ptr(&mut self) -> ErasedComponentMutPtr {
        let Self {
            ref mut fields,
            component_id,
        } = *self;

        let field = fields.as_mut_field_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { fields, .. } = self;
        fields.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[u8] {
        let Self { fields, .. } = self;
        fields.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [u8] {
        let Self { fields, .. } = self;
        fields.as_mut_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedComponentMutPtr, usize) {
        let Self {
            component_id,
            fields,
        } = self;

        let (field, len) = fields.into_parts();
        let ptr = unsafe { ErasedComponentMutPtr::from_parts(component_id, field) };
        (ptr, len)
    }
}

impl PartialEq for ErasedComponentSliceMut<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentSliceMut<'_> {}

impl PartialOrd for ErasedComponentSliceMut<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentSliceMut<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentSliceMut<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentSliceMut<'_> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl AsRef<[u8]> for ErasedComponentSliceMut<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_buffer()
    }
}

impl AsMut<[u8]> for ErasedComponentSliceMut<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_buffer()
    }
}
