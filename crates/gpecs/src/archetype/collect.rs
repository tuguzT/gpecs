use crate::{
    archetype::error::{ArchetypeError, DuplicateComponentError},
    component::{error::NotRegisteredError, registry::ComponentId},
};

#[inline]
pub fn try_collect_components<S, I>(
    component_ids: I,
    mut insert_fn: impl FnMut(&mut S, I::Item) -> bool,
    mut component_id_fn: impl FnMut(&I::Item) -> ComponentId,
) -> Result<S, DuplicateComponentError>
where
    S: Default,
    I: IntoIterator,
{
    let mut set = S::default();
    component_ids.into_iter().try_for_each(|item| {
        let component_id = component_id_fn(&item);
        let is_unique = insert_fn(&mut set, item);
        is_unique
            .then(Default::default)
            .ok_or_else(|| DuplicateComponentError::new(component_id))
    })?;
    Ok(set)
}

#[inline]
pub fn try_collect_opt_components<S, I, T>(
    component_ids: I,
    mut insert_fn: impl FnMut(&mut S, T) -> bool,
    mut component_id_fn: impl FnMut(&T) -> ComponentId,
) -> Result<S, ArchetypeError>
where
    S: Default,
    I: IntoIterator<Item = Option<T>>,
{
    let mut set = S::default();
    component_ids
        .into_iter()
        .try_for_each::<_, Result<_, ArchetypeError>>(|item| {
            let Some(item) = item else {
                return Err(NotRegisteredError.into());
            };
            let component_id = component_id_fn(&item);
            let is_unique = insert_fn(&mut set, item);
            is_unique
                .then(Default::default)
                .ok_or_else(|| DuplicateComponentError::new(component_id).into())
        })?;
    Ok(set)
}
