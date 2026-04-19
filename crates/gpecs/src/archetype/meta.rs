use gpecs_soa_erased::{ptr::slice::SliceItemPtrs, storage::AlignedStorage};

use crate::{
    archetype::erased::FromComponentInfo,
    bundle::erased::FromErasedComponent,
    component::{
        erased::{ErasedComponent, ErasedDrop, WithErasedDrop},
        registry::ComponentInfo,
    },
    soa::field::FieldDescriptor,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedDropMeta {
    desc: FieldDescriptor,
    erased_drop: Option<ErasedDrop>,
}

impl AsRef<FieldDescriptor> for ErasedDropMeta {
    #[inline]
    fn as_ref(&self) -> &FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }
}

impl WithErasedDrop for ErasedDropMeta {
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        let Self { erased_drop, .. } = *self;
        erased_drop
    }
}

impl<Meta> FromComponentInfo<'_, Meta> for ErasedDropMeta
where
    Meta: AsRef<FieldDescriptor> + WithErasedDrop,
{
    #[inline]
    fn from_component_info(info: ComponentInfo<&Meta>) -> Self {
        let desc = FromComponentInfo::from_component_info(info);
        let erased_drop = FromComponentInfo::from_component_info(info);
        Self { desc, erased_drop }
    }
}

impl<S, P> FromErasedComponent<S, P> for ErasedDropMeta
where
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    fn from_erased_component(component: &ErasedComponent<S, P>) -> Self {
        let desc = FieldDescriptor::new(component.as_field().layout());
        let erased_drop = component.erased_drop();
        Self { desc, erased_drop }
    }
}
