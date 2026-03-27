use crate::registry::ComponentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentInfo<Meta>
where
    Meta: ?Sized,
{
    component_id: ComponentId,
    meta: Meta,
}

impl<Meta> ComponentInfo<Meta> {
    #[inline]
    pub(super) fn new(component_id: ComponentId, meta: Meta) -> Self {
        Self { component_id, meta }
    }

    #[inline]
    pub fn map_meta<F, N>(self, f: F) -> ComponentInfo<N>
    where
        F: FnOnce(Meta) -> N,
    {
        let Self { component_id, meta } = self;

        let meta = f(meta);
        ComponentInfo { component_id, meta }
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Meta) {
        let Self { component_id, meta } = self;
        (component_id, meta)
    }
}

impl<Meta> ComponentInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn as_meta(&self) -> &Meta {
        let Self { meta, .. } = self;
        meta
    }
}
