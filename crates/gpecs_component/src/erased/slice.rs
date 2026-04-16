use core::{
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use gpecs_erased::{data::ErasedSlice, ptr::slice::ConstSliceItemPtr};

use crate::{
    Component,
    erased::{
        ErasedComponentPtr, ErasedComponentSlicePtr,
        error::{DowncastError, NotRegisteredError, TryFromSlicePtrError, check_downcast},
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Clone, Copy)]
pub struct ErasedComponentSlice<'a, T>
where
    T: ConstSliceItemPtr,
{
    component_id: ComponentId,
    fields: ErasedSlice<'a, T>,
}

impl<'a, T> ErasedComponentSlice<'a, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn try_from<C, U>(
        components: &ComponentRegistryView<impl Sized, U>,
        component: &'a [C],
    ) -> Result<Self, TryFromSlicePtrError>
    where
        C: Component,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<C>()
            .ok_or_else(NotRegisteredError::of::<C>)?;
        let fields = ErasedSlice::try_from(component)?;

        let me = unsafe { Self::from_parts(component_id, fields) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(component_id: ComponentId, fields: ErasedSlice<'a, T>) -> Self {
        Self {
            component_id,
            fields,
        }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedComponentSlicePtr<T>) -> Self {
        let (component_id, fields) = ptr.into_parts();
        let fields = unsafe { fields.as_ref_unchecked() };
        unsafe { Self::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn downcast<C, U>(
        self,
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<&'a [C], DowncastError<Self>>
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
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn fields(&self) -> &ErasedSlice<'a, T> {
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
    pub fn as_component_slice_ptr(&self) -> ErasedComponentSlicePtr<T> {
        let Self {
            ref fields,
            component_id,
        } = *self;

        let fields = fields.as_field_slice_ptr();
        unsafe { ErasedComponentSlicePtr::from_parts(component_id, fields) }
    }

    #[inline]
    pub fn as_component_ptr(&self) -> ErasedComponentPtr<T> {
        let Self {
            ref fields,
            component_id,
        } = *self;

        let field = fields.as_field_ptr();
        unsafe { ErasedComponentPtr::from_parts(component_id, field) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { fields, .. } = self;
        fields.as_ptr()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[T::Item] {
        let Self { fields, .. } = self;
        fields.as_buffer()
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, ErasedSlice<'a, T>) {
        let Self {
            component_id,
            fields,
        } = self;
        (component_id, fields)
    }
}

impl<T> Debug for ErasedComponentSlice<'_, T>
where
    T: ConstSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            component_id,
            fields,
        } = self;

        f.debug_struct("ErasedComponentSlice")
            .field("component_id", component_id)
            .field("fields", fields)
            .finish()
    }
}

impl<T, U> PartialEq<ErasedComponentSlice<'_, U>> for ErasedComponentSlice<'_, T>
where
    T: ConstSliceItemPtr,
    U: ConstSliceItemPtr,
{
    #[inline]
    fn eq(&self, other: &ErasedComponentSlice<'_, U>) -> bool {
        let Self { component_id, .. } = self;
        component_id.eq(other.borrow())
    }
}

impl<T> Eq for ErasedComponentSlice<'_, T> where T: ConstSliceItemPtr {}

impl<T, U> PartialOrd<ErasedComponentSlice<'_, U>> for ErasedComponentSlice<'_, T>
where
    T: ConstSliceItemPtr,
    U: ConstSliceItemPtr,
{
    #[inline]
    fn partial_cmp(&self, other: &ErasedComponentSlice<'_, U>) -> Option<cmp::Ordering> {
        let Self { component_id, .. } = self;
        component_id.partial_cmp(other.borrow())
    }
}

impl<T> Ord for ErasedComponentSlice<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { component_id, .. } = self;
        component_id.cmp(other.borrow())
    }
}

impl<T> Hash for ErasedComponentSlice<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { component_id, .. } = self;
        component_id.hash(state);
    }
}

impl<T> Borrow<ComponentId> for ErasedComponentSlice<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn borrow(&self) -> &ComponentId {
        let Self { component_id, .. } = self;
        component_id
    }
}

impl<T> AsRef<[T::Item]> for ErasedComponentSlice<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}
