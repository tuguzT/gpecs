use std::{
    borrow::Borrow,
    cmp,
    hash::{self, Hash},
    mem::MaybeUninit,
};

use gpecs_soa_erased::data::ErasedMutSlice;

use crate::component::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentMutSlicePtr, ErasedComponentPtr,
        ErasedComponentSlice, ErasedComponentSlicePtr,
        error::{DowncastError, NotRegisteredError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistry,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

type Fields<'a> = ErasedMutSlice<'a, *mut MaybeUninit<u8>>;

#[derive(Debug)]
pub struct ErasedComponentMutSlice<'a> {
    component_id: ComponentId,
    fields: Fields<'a>,
}

impl<'a> ErasedComponentMutSlice<'a> {
    #[inline]
    pub fn try_from<C, T>(
        components: &ComponentRegistry<impl Sized, T>,
        component: &'a mut [C],
    ) -> Result<Self, NotRegisteredError>
    where
        C: Component,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components.component_id::<C>().ok_or(NotRegisteredError)?;
        let fields = Fields::try_from(component)
            .expect("alignment of bytes should be sufficient for any component");

        let me = unsafe { Self::from_parts(component_id, fields) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, fields: Fields<'a>) -> Self {
        Self {
            component_id,
            fields,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentMutSlicePtr) -> Self {
        let (component_id, fields) = ptr.into_parts();
        let fields = unsafe { fields.deref_mut() };
        unsafe { Self::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn downcast<C, T>(
        self,
        components: &ComponentRegistry<impl Sized, T>,
    ) -> Result<&'a mut [C], DowncastError<Self>>
    where
        C: Component,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let Self { fields, .. } = check_downcast::<C, T, _>(components, component_id, self)?;

        let component = unsafe { fields.downcast::<C>() }
            .expect("descriptors of input component and self should be equal");
        Ok(component)
    }

    #[inline]
    pub fn downcast_ref<C, T>(
        &self,
        components: &ComponentRegistry<impl Sized, T>,
    ) -> Result<&[C], DowncastError<&Self>>
    where
        C: Component,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = *self;
        let Self { fields, .. } = check_downcast::<C, T, _>(components, component_id, self)?;

        let component = unsafe { fields.downcast_ref::<C>() }
            .expect("descriptors of input component and self should be equal");
        Ok(component)
    }

    #[inline]
    pub fn downcast_mut<C, T>(
        &mut self,
        components: &ComponentRegistry<impl Sized, T>,
    ) -> Result<&mut [C], DowncastError<&mut Self>>
    where
        C: Component,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = *self;
        let Self { fields, .. } = check_downcast::<C, T, _>(components, component_id, self)?;

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

        let fields = fields.as_field_slice_ptr();
        unsafe { ErasedComponentSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn as_mut_component_slice_ptr(&mut self) -> ErasedComponentMutSlicePtr {
        let Self {
            ref mut fields,
            component_id,
        } = *self;

        let fields = fields.as_mut_field_slice_ptr();
        unsafe { ErasedComponentMutSlicePtr::from_parts(component_id, fields) }
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
    pub fn as_ptr(&self) -> *const MaybeUninit<u8> {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut MaybeUninit<u8> {
        let Self { fields, .. } = self;
        fields.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<u8>] {
        let Self { fields, .. } = self;
        fields.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<u8>] {
        let Self { fields, .. } = self;
        fields.as_mut_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Fields<'a>) {
        let Self {
            component_id,
            fields,
        } = self;
        (component_id, fields)
    }
}

impl PartialEq for ErasedComponentMutSlice<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl Eq for ErasedComponentMutSlice<'_> {}

impl PartialOrd for ErasedComponentMutSlice<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErasedComponentMutSlice<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl Hash for ErasedComponentMutSlice<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl Borrow<ComponentId> for ErasedComponentMutSlice<'_> {
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl AsRef<[MaybeUninit<u8>]> for ErasedComponentMutSlice<'_> {
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<u8>] {
        self.as_buffer()
    }
}

impl<'a> From<ErasedComponentMutSlice<'a>> for ErasedComponentSlice<'a> {
    #[inline]
    fn from(slice: ErasedComponentMutSlice<'a>) -> Self {
        let (component_id, fields) = slice.into_parts();
        let fields = fields.into();
        unsafe { Self::from_parts(component_id, fields) }
    }
}
