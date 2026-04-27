use gpecs_component::registry::{ComponentId, ComponentRegistryView};

use crate::erased::{
    ErasedArchetype, ErasedArchetypeView,
    error::{IncompatibleArchetypeError, IncompatibleArchetypeExactError},
};

impl<Meta> ErasedArchetypeView<'_, Meta> {
    #[inline]
    pub fn check_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let other = ErasedArchetype::<()>::new(components, component_ids)?;
        self.check_compatibility(other.as_view())?;
        Ok(())
    }

    #[inline]
    pub fn check_exact_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let other = ErasedArchetype::<()>::new(components, component_ids)?;
        self.check_exact_compatibility(other.as_view())?;
        Ok(())
    }
}
