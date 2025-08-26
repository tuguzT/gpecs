use gpecs_types::world::WorldId;

#[test]
fn default() {
    let default = WorldId::default();
    assert_eq!(u16::from(default), 0);
    assert_eq!(u32::from(default), 0);

    let new = WorldId::new();
    assert_eq!(u16::from(new), 0);
    assert_eq!(u32::from(new), 0);

    assert_eq!(new, default);
}

#[test]
fn u16() {
    let min = unsafe { WorldId::from_u16(u16::MIN) };
    assert_eq!(u16::from(min), u16::MIN);
    assert_eq!(u32::from(min), u32::MIN);

    let id = unsafe { WorldId::from_u16(42) };
    assert_eq!(u16::from(id), 42);
    assert_eq!(u32::from(id), 42);

    let max = unsafe { WorldId::from_u16(u16::MAX) };
    assert_eq!(u16::from(max), u16::MAX);
    assert_eq!(u32::from(max), u16::MAX.into());
}

#[test]
#[cfg_attr(debug_assertions, should_panic = "`WorldId` should fit into `u16`")]
fn u32() {
    let min = unsafe { WorldId::try_from_u32(u32::MIN).unwrap() };
    assert_eq!(u16::from(min), u16::MIN);
    assert_eq!(u32::from(min), u32::MIN);

    let id = unsafe { WorldId::try_from_u32(42).unwrap() };
    assert_eq!(u16::from(id), 42);
    assert_eq!(u32::from(id), 42);

    let max = unsafe { WorldId::try_from_u32(u16::MAX.into()).unwrap() };
    assert_eq!(u16::from(max), u16::MAX);
    assert_eq!(u32::from(max), u16::MAX.into());

    let error = unsafe { WorldId::try_from_u32(u32::MAX).unwrap_err() };
    assert_eq!(error.to_string(), "`WorldId` should fit into `u16`");

    let overflow = unsafe { WorldId::from_u32(u32::MAX) };
    assert_eq!(u16::from(overflow), u16::MAX);
    assert_eq!(u32::from(overflow), u32::MAX);
}
