use std::{
    alloc::Layout,
    any::{self, TypeId},
    borrow::Cow,
};

use crate::{
    component::{
        Component,
        erased::{ErasedDrop, WithErasedDrop},
        registry::{self, ComponentIdMap, ComponentRegistry, traits::FromComponentType},
    },
    hash::BuildHasher,
    soa::layout::WithLayout,
};

pub type ComponentTypeIdMap = ComponentIdMap<TypeId, BuildHasher>;
pub type Components = ComponentRegistry<Vec<ComponentDescriptor>, ComponentTypeIdMap>;
pub type ComponentInfo<'a> = registry::ComponentInfo<&'a ComponentDescriptor>;

#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    layout: Layout,
    erased_drop: Option<ErasedDrop>,
}

impl ComponentDescriptor {
    #[inline]
    pub fn new<N>(
        name: N,
        type_id: Option<TypeId>,
        layout: Layout,
        erased_drop: Option<ErasedDrop>,
    ) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        Self {
            name: name.into(),
            type_id,
            layout,
            erased_drop,
        }
    }

    #[inline]
    pub fn of<T>() -> Self
    where
        T: Component,
    {
        let name = any::type_name::<T>();
        let type_id = Some(TypeId::of::<T>());
        let desc = Layout::new::<T>();
        let erased_drop = ErasedDrop::of::<T>();
        Self::new(name, type_id, desc, erased_drop)
    }

    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        let Self { type_id, .. } = *self;
        type_id
    }

    #[inline]
    pub fn name(&self) -> &str {
        let Self { name, .. } = self;
        name.as_ref()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn erased_drop(&self) -> Option<ErasedDrop> {
        let Self { erased_drop, .. } = *self;
        erased_drop
    }
}

impl AsRef<str> for ComponentDescriptor {
    #[inline]
    fn as_ref(&self) -> &str {
        self.name()
    }
}

unsafe impl FromComponentType for ComponentDescriptor {
    #[inline]
    fn from_component<T: Component>() -> Self {
        Self::of::<T>()
    }
}

impl WithLayout for ComponentDescriptor {
    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(self)
    }
}

impl WithErasedDrop for ComponentDescriptor {
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        Self::erased_drop(self)
    }
}
