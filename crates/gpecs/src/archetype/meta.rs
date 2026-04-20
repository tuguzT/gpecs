use std::alloc::Layout;

use gpecs_soa_erased::{ptr::slice::SliceItemPtrs, storage::AlignedStorage};

use crate::{
    archetype::erased::FromComponentInfo,
    bundle::erased::FromErasedComponent,
    component::{
        erased::{ErasedComponent, ErasedDrop, WithErasedDrop},
        registry::ComponentInfo,
    },
    soa::layout::WithLayout,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedDropMeta {
    layout: Layout,
    erased_drop: Option<ErasedDrop>,
}

impl WithLayout for ErasedDropMeta {
    #[inline]
    fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
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
    Meta: WithLayout + WithErasedDrop,
{
    #[inline]
    fn from_component_info(info: ComponentInfo<&Meta>) -> Self {
        Self {
            layout: info.layout(),
            erased_drop: FromComponentInfo::from_component_info(info),
        }
    }
}

impl<S, P> FromErasedComponent<S, P> for ErasedDropMeta
where
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    fn from_erased_component(component: &ErasedComponent<S, P>) -> Self {
        Self {
            layout: component.as_field().layout(),
            erased_drop: component.erased_drop(),
        }
    }
}
