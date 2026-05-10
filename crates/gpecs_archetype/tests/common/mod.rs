use std::{alloc::Layout, any::TypeId};

use bytemuck::{Pod, Zeroable};
use gpecs_archetype::{bundle::erased::FromErasedComponent, erased::FromComponentDescriptor};
use gpecs_component::{
    Component,
    erased::{ErasedComponent, ErasedDrop, WithErasedDrop},
    registry::{ComponentIdMap, ComponentRegistry, traits::FromComponentType},
};
use gpecs_soa_erased::{
    BufferOffsetsFromLayout, BufferOffsetsFromSelf, layout::WithLayout, ptr::slice::SliceItemPtrs,
    storage::AlignedStorage,
};

#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub padding: u32,
}

impl Component for Position {}

#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct Tag;

impl Component for Tag {}

#[derive(Debug, PartialEq, Clone)]
pub struct Name {
    pub value: String,
}

impl Component for Name {}

#[derive(Debug, Clone, Copy)]
pub struct ComponentDescriptor {
    layout: Layout,
    erased_drop: Option<ErasedDrop>,
}

impl WithLayout for ComponentDescriptor {
    fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }
}

impl WithErasedDrop for ComponentDescriptor {
    fn erased_drop(&self) -> Option<ErasedDrop> {
        let Self { erased_drop, .. } = *self;
        erased_drop
    }
}

unsafe impl FromComponentType for ComponentDescriptor {
    fn from_component<T: Component>() -> Self {
        Self {
            layout: Layout::new::<T>(),
            erased_drop: ErasedDrop::of::<T>(),
        }
    }
}

pub type Components = ComponentRegistry<Vec<ComponentDescriptor>, ComponentIdMap<TypeId>>;

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

unsafe impl BufferOffsetsFromSelf for ErasedDropMeta {
    type BufferOffsets = BufferOffsetsFromLayout;
}

impl<Meta> FromComponentDescriptor<'_, Meta> for ErasedDropMeta
where
    Meta: WithLayout + WithErasedDrop,
{
    #[inline]
    fn from_component_descriptor(info: &Meta) -> Self {
        Self {
            layout: info.layout(),
            erased_drop: FromComponentDescriptor::from_component_descriptor(info),
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
