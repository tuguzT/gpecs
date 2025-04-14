use crate::{
    component::registry::{ComponentId, ComponentRegistry},
    soa::traits::{DefaultContext, Soa},
};

mod impls;

/// Non-empty collection of [components](crate::component::Component).
#[allow(unsafe_code)]
pub unsafe trait Bundle: Soa<Context = DefaultContext> + 'static {
    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](Soa::FieldDescriptors).
    type MaybeComponentIds: IntoIterator<Item = Option<ComponentId>>;

    fn get_components(components: &ComponentRegistry) -> Self::MaybeComponentIds;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](Soa::FieldDescriptors).
    type ComponentIds: IntoIterator<Item = ComponentId>;

    fn register_components(components: &mut ComponentRegistry) -> Self::ComponentIds;
}
