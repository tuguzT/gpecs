use crate::{
    archetype::error::{ComponentNotRegisteredError, DuplicateComponentError, GetComponentsError},
    component::registry::ComponentId,
};

#[inline]
pub fn try_collect_component_ids<S, I, F>(
    component_ids: I,
    mut insert_fn: F,
) -> Result<S, DuplicateComponentError>
where
    S: Default,
    I: IntoIterator,
    I::Item: Into<ComponentId> + Copy,
    F: FnMut(&mut S, I::Item) -> bool,
{
    let mut set = S::default();
    component_ids.into_iter().try_for_each(|component_id| {
        let is_unique = insert_fn(&mut set, component_id);
        is_unique
            .then(Default::default)
            .ok_or_else(|| DuplicateComponentError::new(component_id.into()))
    })?;
    Ok(set)
}

#[inline]
pub fn try_collect_maybe_component_ids<S, I, T, F>(
    component_ids: I,
    mut insert_fn: F,
) -> Result<S, GetComponentsError>
where
    S: Default,
    I: IntoIterator<Item = Option<T>>,
    T: Into<ComponentId> + Copy,
    F: FnMut(&mut S, T) -> bool,
{
    let mut set = S::default();
    component_ids
        .into_iter()
        .try_for_each::<_, Result<_, GetComponentsError>>(|component_id| {
            let Some(component_id) = component_id else {
                return Err(ComponentNotRegisteredError.into());
            };
            let is_unique = insert_fn(&mut set, component_id);
            is_unique
                .then(Default::default)
                .ok_or_else(|| DuplicateComponentError::new(component_id.into()).into())
        })?;
    Ok(set)
}
