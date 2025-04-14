use crate::{
    component::registry::{ComponentId, ComponentRegistry},
    soa::traits::Soa,
};

mod impls;

/// Non-empty collection of [components](crate::component::Component).
#[allow(unsafe_code)]
pub unsafe trait Bundle: Soa + 'static {
    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](Soa::FieldDescriptors).
    type MaybeComponentIds: IntoIterator<Item = Option<ComponentId>>;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](Soa::FieldDescriptors).
    type ComponentIds: IntoIterator<Item = ComponentId>;

    fn get_components(
        context: &Self::Context,
        components: &ComponentRegistry,
    ) -> Self::MaybeComponentIds;

    fn register_components(
        context: &Self::Context,
        components: &mut ComponentRegistry,
    ) -> Self::ComponentIds;
}
