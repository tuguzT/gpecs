use gpecs_soa_erased::{erased::ErasedSoaContext, soa::traits::FieldDescriptor};

#[test]
#[cfg_attr(miri, ignore)]
fn erased_context() {
    let descriptors = [FieldDescriptor::of::<u8>(), FieldDescriptor::of::<i16>()];
    let _context = ErasedSoaContext::new(descriptors);
}

#[test]
#[cfg_attr(miri, ignore)]
fn erased_context_of() {
    let context = ErasedSoaContext::of::<()>(&());
    let descriptors = [FieldDescriptor::of::<()>()];
    assert!(context
        .field_descriptors()
        .iter()
        .map(FieldDescriptor::layout)
        .eq(descriptors.iter().map(FieldDescriptor::layout)));

    let context = ErasedSoaContext::of::<(u32, u16, u8)>(&());
    let descriptors = [
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u32>(),
    ];
    assert!(context
        .field_descriptors()
        .iter()
        .map(FieldDescriptor::layout)
        .eq(descriptors.iter().map(FieldDescriptor::layout)));
}
