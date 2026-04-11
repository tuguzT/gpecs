use core::{
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use gpecs_erased::{
    data::ErasedMutSlice,
    ptr::slice::{CastConst, MutSliceItemPtr},
};
use polonius_the_crab::{polonius, polonius_return};

use crate::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentMutSlicePtr, ErasedComponentPtr,
        ErasedComponentSlice, ErasedComponentSlicePtr,
        error::{DowncastError, NotRegisteredError, TryFromSlicePtrError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

pub struct ErasedComponentMutSlice<'a, T>
where
    T: MutSliceItemPtr,
{
    component_id: ComponentId,
    fields: ErasedMutSlice<'a, T>,
}

impl<'a, T> ErasedComponentMutSlice<'a, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub fn try_from<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: &'a mut [C],
    ) -> Result<Self, TryFromSlicePtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let fields = ErasedMutSlice::try_from(component)?;

        let me = unsafe { Self::from_parts(component_id, fields) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, fields: ErasedMutSlice<'a, T>) -> Self {
        Self {
            component_id,
            fields,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentMutSlicePtr<T>) -> Self {
        let (component_id, fields) = ptr.into_parts();
        let fields = unsafe { fields.deref_mut() };
        unsafe { Self::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&'a mut [C], DowncastError<Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = self;
        let Self { fields, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let into_self = |fields| unsafe { Self::from_parts(component_id, fields) };
        let component = unsafe { fields.downcast() }.map_err(|err| err.map_value(into_self))?;
        Ok(component)
    }

    #[inline]
    pub fn downcast_ref<C, U>(
        &self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&[C], DowncastError<&Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = *self;
        let Self { fields, .. } = check_downcast::<C, U, _>(components, component_id, self)?;

        let component = unsafe { fields.downcast_ref() }.map_err(|err| err.map_value(|_| self))?;
        Ok(component)
    }

    #[inline]
    pub fn downcast_mut<C, U>(
        &mut self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&mut [C], DowncastError<&mut Self>>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let Self { component_id, .. } = *self;
        let mut this = check_downcast::<C, U, _>(components, component_id, self)?;

        let source = polonius!(|this| -> Result<&'polonius mut [C], _> {
            match unsafe { this.fields.downcast_mut() } {
                Ok(component) => polonius_return!(Ok(component)),
                Err(error) => error.source.into(),
            }
        });
        Err(DowncastError::new(this, source))
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn fields(&self) -> &ErasedMutSlice<'a, T> {
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
    pub fn as_component_slice_ptr(&self) -> ErasedComponentSlicePtr<CastConst<T>> {
        let Self {
            ref fields,
            component_id,
        } = *self;

        let fields = fields.as_field_slice_ptr();
        unsafe { ErasedComponentSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn as_mut_component_slice_ptr(&mut self) -> ErasedComponentMutSlicePtr<T> {
        let Self {
            ref mut fields,
            component_id,
        } = *self;

        let fields = fields.as_mut_field_slice_ptr();
        unsafe { ErasedComponentMutSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn as_component_ptr(&self) -> ErasedComponentPtr<CastConst<T>> {
        let Self {
            ref fields,
            component_id,
        } = *self;

        let field = fields.as_field_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_mut_component_ptr(&mut self) -> ErasedComponentMutPtr<T> {
        let Self {
            ref mut fields,
            component_id,
        } = *self;

        let field = fields.as_mut_field_ptr();
        unsafe { ErasedComponentMutPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T::Item {
        let Self { fields, .. } = self;
        fields.as_mut_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[T::Item] {
        let Self { fields, .. } = self;
        fields.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [T::Item] {
        let Self { fields, .. } = self;
        fields.as_mut_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedMutSlice<'a, T>) {
        let Self {
            component_id,
            fields,
        } = self;
        (component_id, fields)
    }
}

impl<T> Debug for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            component_id,
            fields,
        } = self;

        f.debug_struct("ErasedComponentMutSlice")
            .field("component_id", component_id)
            .field("fields", fields)
            .finish()
    }
}

impl<T, U> PartialEq<ErasedComponentMutSlice<'_, U>> for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr,
    U: MutSliceItemPtr,
{
    #[inline]
    fn eq(&self, other: &ErasedComponentMutSlice<'_, U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentMutSlice<'_, T> where T: MutSliceItemPtr {}

impl<T, U> PartialOrd<ErasedComponentMutSlice<'_, U>> for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr,
    U: MutSliceItemPtr,
{
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentMutSlice<'_, U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl<T> AsRef<[T::Item]> for ErasedComponentMutSlice<'_, T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}

impl<'a, T> From<ErasedComponentMutSlice<'a, T>> for ErasedComponentSlice<'a, CastConst<T>>
where
    T: MutSliceItemPtr,
{
    #[inline]
    fn from(slice: ErasedComponentMutSlice<'a, T>) -> Self {
        let (component_id, fields) = slice.into_parts();
        let fields = fields.into();
        unsafe { Self::from_parts(component_id, fields) }
    }
}
