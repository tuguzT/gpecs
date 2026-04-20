use std::alloc::Layout;

use gpecs_soa_erased::{ErasedSoaContext, ptr::slice::CoreSliceItemPtrs};

use crate::common::ArrayLayouts;

type Ptrs = CoreSliceItemPtrs<u8>;

#[test]
#[cfg_attr(miri, ignore)]
fn context() {
    let layouts = [Layout::new::<u8>(), Layout::new::<i16>()];
    let _context = ErasedSoaContext::<_, Ptrs>::new(layouts).unwrap();
}

#[test]
#[cfg_attr(miri, ignore)]
fn context_of() {
    type Layouts = ArrayLayouts<Layout, 3>;

    let context = Default::default();
    let context = ErasedSoaContext::<Layouts, Ptrs>::of::<(u32, u16, u8)>(&context).unwrap();

    let layouts = [
        Layout::new::<u8>(),
        Layout::new::<u16>(),
        Layout::new::<u32>(),
    ];
    itertools::assert_equal(context.field_layouts().iter().copied(), layouts);
}

#[test]
#[cfg_attr(miri, ignore)]
fn context_of_zst() {
    type Layouts = ArrayLayouts<Layout, 1>;

    let context = Default::default();
    let context = ErasedSoaContext::<Layouts, Ptrs>::of::<()>(&context).unwrap();

    let layouts = [];
    itertools::assert_equal(context.field_layouts().iter().copied(), layouts);
}
