use std::{
    alloc::Layout,
    any::{self, TypeId},
    borrow::Cow,
};

use crate::{context::ComponentDescriptor, executor::gpu::component::GpuComponent};

#[derive(Debug, Clone)]
pub struct GpuComponentDescriptor {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    layout: Layout,
}

impl GpuComponentDescriptor {
    #[inline]
    pub fn new<N, I>(name: N, type_id: I, layout: Layout) -> Self
    where
        N: Into<Cow<'static, str>>,
        I: Into<Option<TypeId>>,
    {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            layout,
        }
    }

    #[inline]
    pub fn of<T>() -> Self
    where
        T: GpuComponent,
    {
        Self {
            name: any::type_name::<T>().into(),
            type_id: Some(TypeId::of::<T>()),
            layout: Layout::new::<T>(),
        }
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
}

impl From<GpuComponentDescriptor> for ComponentDescriptor {
    fn from(value: GpuComponentDescriptor) -> Self {
        let GpuComponentDescriptor {
            name,
            type_id,
            layout,
        } = value;
        Self::new(name, type_id, layout, None)
    }
}
