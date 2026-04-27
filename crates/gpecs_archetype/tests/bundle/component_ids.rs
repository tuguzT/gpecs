#![cfg(feature = "alloc")]

use gpecs_archetype::bundle::Bundle;

use crate::common::{Components, Name, Position, Tag};

#[test]
fn get_components() {
    let mut components = Components::new();

    let ids = <(Position,)>::get_components(&components.as_view());
    assert_eq!(ids, [None]);

    let position_id = components.register_component::<Position>();
    let ids = <(Position,)>::get_components(&components.as_view());
    assert_eq!(ids, [Some(position_id)]);

    // TODO: this should return an error!
    let ids = <(Position, Position)>::get_components(&components.as_view());
    assert_eq!(ids, [Some(position_id), Some(position_id)]);

    let ids = <(Position, Tag)>::get_components(&components.as_view());
    assert_eq!(ids, [None, Some(position_id)]);

    let tag_id = components.register_component::<Tag>();
    let ids = <(Position, Tag)>::get_components(&components.as_view());
    assert_eq!(ids, [Some(tag_id), Some(position_id)]);

    let ids = <(Position, Name, Tag)>::get_components(&components.as_view());
    assert_eq!(ids, [Some(tag_id), None, Some(position_id)]);

    let name_id = components.register_component::<Name>();
    let ids = <(Name, Position, Tag)>::get_components(&components.as_view());
    assert_eq!(ids, [Some(tag_id), Some(name_id), Some(position_id)]);

    // TODO: this should return an error!
    let ids = <(Position, Name, Position, Tag, Tag)>::get_components(&components.as_view());
    assert_eq!(
        ids,
        [
            Some(tag_id),
            Some(tag_id),
            Some(name_id),
            Some(position_id),
            Some(position_id),
        ]
    );
}

#[test]
fn register_components() {
    let mut components = Components::new();

    let position_id = components.component_id::<Position>();
    assert_eq!(position_id, None);

    let ids = <(Position,)>::register_components(&mut components);
    let position_id = components
        .component_id::<Position>()
        .expect("`Position` component should be already registered");
    assert_eq!(ids, [position_id]);

    // TODO: this should return an error!
    let ids = <(Position, Position)>::register_components(&mut components);
    assert_eq!(ids, [position_id, position_id]);

    let ids = <(Position, Tag)>::register_components(&mut components);
    let tag_id = components
        .component_id::<Tag>()
        .expect("`Tag` component should be already registered");
    assert_eq!(ids, [tag_id, position_id]);

    let name_id = components.register_component::<Name>();
    let ids = <(Name, Position, Tag)>::register_components(&mut components);
    assert_eq!(ids, [tag_id, name_id, position_id]);

    // TODO: this should return an error!
    let ids = <(Position, Name, Position, Tag, Tag)>::register_components(&mut components);
    assert_eq!(ids, [tag_id, tag_id, name_id, position_id, position_id]);
}
