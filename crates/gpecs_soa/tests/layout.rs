use gpecs_soa::vec::SoaVec;

#[test]
fn null_opt() {
    type Vec = SoaVec<u32, u16, u8>;
    assert_eq!(size_of::<Option<Vec>>(), size_of::<Vec>());
}
