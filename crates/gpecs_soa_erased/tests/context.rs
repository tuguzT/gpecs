use arrayvec::ArrayVec;
use gpecs_soa_erased::{erased::ErasedSoaContext, soa::field::FieldDescriptor};

type ArrayDescriptors<const CAP: usize> = ArrayVec<FieldDescriptor, CAP>;

#[test]
#[cfg_attr(miri, ignore)]
fn context() {
    let descriptors = [FieldDescriptor::of::<u8>(), FieldDescriptor::of::<i16>()];
    let _context = ErasedSoaContext::<_, u8>::new(descriptors).unwrap();
}

#[test]
#[cfg_attr(miri, ignore)]
fn context_of() {
    let context = Default::default();
    let context =
        ErasedSoaContext::<ArrayDescriptors<3>, u8>::of::<(u32, u16, u8)>(&context).unwrap();

    let descriptors = [
        FieldDescriptor::of::<u8>(),
        FieldDescriptor::of::<u16>(),
        FieldDescriptor::of::<u32>(),
    ];
    itertools::assert_equal(
        context
            .field_descriptors()
            .iter()
            .copied()
            .map(FieldDescriptor::layout),
        descriptors.map(FieldDescriptor::layout),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn context_of_zst() {
    let context = Default::default();
    let context = ErasedSoaContext::<ArrayDescriptors<1>, u8>::of::<()>(&context).unwrap();

    let descriptors = [FieldDescriptor::of::<()>()];
    itertools::assert_equal(
        context
            .field_descriptors()
            .iter()
            .copied()
            .map(FieldDescriptor::layout),
        descriptors.map(FieldDescriptor::layout),
    );
}
