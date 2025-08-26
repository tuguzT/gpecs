use gpecs_sparse::key::Epoch;
use gpecs_types::{
    entity::{Entity, EntityEpoch},
    world::WorldId,
};

mod epoch {
    use super::*;

    #[test]
    fn default() {
        let default = EntityEpoch::default();
        assert_eq!(u16::from(default), 0);
        assert_eq!(u32::from(default), 0);

        let new = EntityEpoch::new();
        assert_eq!(u16::from(new), 0);
        assert_eq!(u32::from(new), 0);

        assert_eq!(new, default);
    }

    #[test]
    fn u16() {
        let min = EntityEpoch::from(u16::MIN);
        assert_eq!(u16::from(min), u16::MIN);
        assert_eq!(u32::from(min), u32::MIN);

        let id = EntityEpoch::from(42);
        assert_eq!(u16::from(id), 42);
        assert_eq!(u32::from(id), 42);

        let max = EntityEpoch::from(u16::MAX);
        assert_eq!(u16::from(max), u16::MAX);
        assert_eq!(u32::from(max), u16::MAX.into());
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic = "`EntityEpoch` should fit into `u16`")]
    fn u32() {
        let min = EntityEpoch::try_from(u32::MIN).unwrap();
        assert_eq!(u16::from(min), u16::MIN);
        assert_eq!(u32::from(min), u32::MIN);

        let id = EntityEpoch::try_from_u32(42).unwrap();
        assert_eq!(u16::from(id), 42);
        assert_eq!(u32::from(id), 42);

        let max = EntityEpoch::try_from_u32(u16::MAX.into()).unwrap();
        assert_eq!(u16::from(max), u16::MAX);
        assert_eq!(u32::from(max), u16::MAX.into());

        let error = EntityEpoch::try_from(u32::MAX).unwrap_err();
        assert_eq!(error.to_string(), "`EntityEpoch` should fit into `u16`");

        let overflow = unsafe { EntityEpoch::from_u32(u32::MAX) };
        assert_eq!(u16::from(overflow), u16::MAX);
        assert_eq!(u32::from(overflow), u32::MAX);
    }

    #[test]
    fn next() {
        let min = EntityEpoch::default();
        assert_eq!(min.next(), EntityEpoch::from(1));

        let id = EntityEpoch::from(42);
        assert_eq!(id.next(), EntityEpoch::from(43));

        let max = EntityEpoch::from(u16::MAX);
        assert_eq!(max.next(), EntityEpoch::default());
    }
}

#[test]
fn new() {
    let epoch = EntityEpoch::default();
    let entity = Entity::new(0, epoch, WorldId::default());
    assert_eq!(entity.index(), 0);
    assert_eq!(entity.epoch(), epoch);
    assert_eq!(entity.world(), WorldId::default());
}

#[test]
fn set() {
    let mut entity = Entity::new(0, EntityEpoch::default(), WorldId::default());

    let index = 42;
    entity.set_index(index);
    assert_eq!(entity.index(), index);
    assert_eq!(entity.epoch(), EntityEpoch::default());
    assert_eq!(entity.world(), WorldId::default());

    let epoch = EntityEpoch::from(7);
    entity.set_epoch(epoch);
    assert_eq!(entity.index(), index);
    assert_eq!(entity.epoch(), epoch);
    assert_eq!(entity.world(), WorldId::default());

    let world = unsafe { WorldId::from_u16(3) };
    entity.set_world(world);
    assert_eq!(entity.index(), index);
    assert_eq!(entity.epoch(), epoch);
    assert_eq!(entity.world(), world);
}

#[test]
fn fmt() {
    let entity = Entity::new(0, EntityEpoch::default(), WorldId::default());
    assert_eq!(format!("{entity}"), "entity{i0e0w0}");
    assert_eq!(
        format!("{entity:?}"),
        "Entity { index: 0, epoch: EntityEpoch(0), world: WorldId(0) }",
    );

    let entity = Entity::new(42, EntityEpoch::from(7), unsafe { WorldId::from_u16(3) });
    assert_eq!(format!("{entity}"), "entity{i42e7w3}");
    assert_eq!(
        format!("{entity:?}"),
        "Entity { index: 42, epoch: EntityEpoch(7), world: WorldId(3) }",
    );
}
