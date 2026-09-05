#![expect(clippy::identity_op)]

use core::alloc::Layout;

use crate::{
    buffer::{
        BufferPrefix, buffer_align, buffer_layout_dangling, buffer_layout_inner, capacity_from,
    },
    traits::AllocSoa,
};

fn buffer_size<T>(context: &T::Context, capacity: usize) -> usize
where
    T: AllocSoa + ?Sized,
{
    let layout = buffer_layout_inner::<T>(&context, capacity).unwrap();
    layout.size()
}

fn prefix_size<T>(context: &T::Context) -> usize
where
    T: AllocSoa + ?Sized,
{
    let next = buffer_layout_dangling::<T>(context);
    let (_, size) = Layout::new::<BufferPrefix<T>>().extend(next).unwrap();
    size
}

fn capacity_from_size<T>(context: &T::Context, buffer_size: usize) -> usize
where
    T: AllocSoa + ?Sized,
{
    let buffer_align = buffer_align::<T>(&context);
    let buffer_layout = Layout::from_size_align(buffer_size, buffer_align).unwrap();
    capacity_from::<T>(&context, buffer_layout)
}

#[test]
#[cfg_attr(miri, ignore)]
fn u8_u8_u8_buffer_size() {
    type SoA = (u8, u8, u8);

    let context = ();
    let buffer_size = |capacity| buffer_size::<SoA>(&context, capacity);

    let u8 = size_of::<u8>();
    let prefix = prefix_size::<SoA>(&context);

    assert_eq!(buffer_size(0), 0);
    assert_eq!(buffer_size(1), prefix + 3 * u8 * 1);
    assert_eq!(buffer_size(2), prefix + 3 * u8 * 2);
    assert_eq!(buffer_size(3), prefix + 3 * u8 * 3);
    assert_eq!(buffer_size(4), prefix + 3 * u8 * 4);
    assert_eq!(buffer_size(5), prefix + 3 * u8 * 5);
    assert_eq!(buffer_size(6), prefix + 3 * u8 * 6);
    assert_eq!(buffer_size(7), prefix + 3 * u8 * 7);
    assert_eq!(buffer_size(8), prefix + 3 * u8 * 8);
    assert_eq!(buffer_size(9), prefix + 3 * u8 * 9);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u8_u8_u8_capacity_from() {
    type SoA = (u8, u8, u8);

    let context = ();
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&context, buffer_size);

    let u8 = size_of::<u8>();
    let prefix = prefix_size::<SoA>(&context);

    for buffer_size in 0..(prefix + 3 * u8 * 1) {
        assert_eq!(0, capacity_from(buffer_size));
    }

    assert_eq!(1, capacity_from(prefix + 3 * u8 * 1));
    assert_eq!(1, capacity_from(prefix + 3 * u8 * 1 + 1));
    assert_eq!(1, capacity_from(prefix + 3 * u8 * 2 - 1));

    assert_eq!(2, capacity_from(prefix + 3 * u8 * 2));
    assert_eq!(2, capacity_from(prefix + 3 * u8 * 2 + 1));
    assert_eq!(2, capacity_from(prefix + 3 * u8 * 3 - 1));

    assert_eq!(3, capacity_from(prefix + 3 * u8 * 3));
    assert_eq!(3, capacity_from(prefix + 3 * u8 * 3 + 1));
    assert_eq!(3, capacity_from(prefix + 3 * u8 * 4 - 1));

    assert_eq!(4, capacity_from(prefix + 3 * u8 * 4));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u16_u16_buffer_size() {
    type SoA = (u16, u16, u16);

    let context = ();
    let buffer_size = |capacity| buffer_size::<SoA>(&context, capacity);

    let u16 = size_of::<u16>();
    let prefix = prefix_size::<SoA>(&context);

    assert_eq!(buffer_size(0), 0);
    assert_eq!(buffer_size(1), prefix + 3 * u16 * 1);
    assert_eq!(buffer_size(2), prefix + 3 * u16 * 2);
    assert_eq!(buffer_size(3), prefix + 3 * u16 * 3);
    assert_eq!(buffer_size(4), prefix + 3 * u16 * 4);
    assert_eq!(buffer_size(5), prefix + 3 * u16 * 5);
    assert_eq!(buffer_size(6), prefix + 3 * u16 * 6);
    assert_eq!(buffer_size(7), prefix + 3 * u16 * 7);
    assert_eq!(buffer_size(8), prefix + 3 * u16 * 8);
    assert_eq!(buffer_size(9), prefix + 3 * u16 * 9);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u16_u16_capacity_from() {
    type SoA = (u16, u16, u16);

    let context = ();
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&context, buffer_size);

    let u16 = size_of::<u16>();
    let prefix = prefix_size::<SoA>(&context);

    for buffer_size in 0..(prefix + 3 * u16 * 1) {
        assert_eq!(0, capacity_from(buffer_size));
    }

    assert_eq!(1, capacity_from(prefix + 3 * u16 * 1));
    assert_eq!(1, capacity_from(prefix + 3 * u16 * 1 + 1));
    assert_eq!(1, capacity_from(prefix + 3 * u16 * 2 - 1));

    assert_eq!(2, capacity_from(prefix + 3 * u16 * 2));
    assert_eq!(2, capacity_from(prefix + 3 * u16 * 2 + 1));
    assert_eq!(2, capacity_from(prefix + 3 * u16 * 3 - 1));

    assert_eq!(3, capacity_from(prefix + 3 * u16 * 3));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u32_u32_u32_buffer_size() {
    type SoA = (u32, u32, u32);

    let context = ();
    let buffer_size = |capacity| buffer_size::<SoA>(&context, capacity);

    let u32 = size_of::<u32>();
    let prefix = prefix_size::<SoA>(&context);

    assert_eq!(buffer_size(0), 0);
    assert_eq!(buffer_size(1), prefix + 3 * u32 * 1);
    assert_eq!(buffer_size(2), prefix + 3 * u32 * 2);
    assert_eq!(buffer_size(3), prefix + 3 * u32 * 3);
    assert_eq!(buffer_size(4), prefix + 3 * u32 * 4);
    assert_eq!(buffer_size(5), prefix + 3 * u32 * 5);
    assert_eq!(buffer_size(6), prefix + 3 * u32 * 6);
    assert_eq!(buffer_size(7), prefix + 3 * u32 * 7);
    assert_eq!(buffer_size(8), prefix + 3 * u32 * 8);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u32_u32_u32_capacity_from() {
    type SoA = (u32, u32, u32);

    let context = ();
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&context, buffer_size);

    let u32 = size_of::<u32>();
    let prefix = prefix_size::<SoA>(&context);

    for buffer_size in 0..(prefix + 3 * u32 * 1) {
        assert_eq!(0, capacity_from(buffer_size));
    }

    assert_eq!(1, capacity_from(prefix + 3 * u32 * 1));
    assert_eq!(1, capacity_from(prefix + 3 * u32 * 1 + 1));
    assert_eq!(1, capacity_from(prefix + 3 * u32 * 2 - 1));

    assert_eq!(2, capacity_from(prefix + 3 * u32 * 2));
    assert_eq!(2, capacity_from(prefix + 3 * u32 * 2 + 1));
    assert_eq!(2, capacity_from(prefix + 3 * u32 * 3 - 1));

    assert_eq!(3, capacity_from(prefix + 3 * u32 * 3));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u64_u64_u64_buffer_size() {
    type SoA = (u64, u64, u64);

    let context = ();
    let buffer_size = |capacity| buffer_size::<SoA>(&context, capacity);

    let u64 = size_of::<u64>();
    let prefix = prefix_size::<SoA>(&context);

    assert_eq!(buffer_size(0), 0);
    assert_eq!(buffer_size(1), prefix + 3 * u64 * 1);
    assert_eq!(buffer_size(2), prefix + 3 * u64 * 2);
    assert_eq!(buffer_size(3), prefix + 3 * u64 * 3);
    assert_eq!(buffer_size(4), prefix + 3 * u64 * 4);
    assert_eq!(buffer_size(5), prefix + 3 * u64 * 5);
    assert_eq!(buffer_size(6), prefix + 3 * u64 * 6);
    assert_eq!(buffer_size(7), prefix + 3 * u64 * 7);
    assert_eq!(buffer_size(8), prefix + 3 * u64 * 8);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u64_u64_u64_capacity_from() {
    type SoA = (u64, u64, u64);

    let context = ();
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&context, buffer_size);

    let u64 = size_of::<u64>();
    let prefix = prefix_size::<SoA>(&context);

    for buffer_size in 0..(prefix + 3 * u64 * 1) {
        assert_eq!(0, capacity_from(buffer_size));
    }

    assert_eq!(1, capacity_from(prefix + 3 * u64 * 1));
    assert_eq!(1, capacity_from(prefix + 3 * u64 * 1 + 1));
    assert_eq!(1, capacity_from(prefix + 3 * u64 * 2 - 1));

    assert_eq!(2, capacity_from(prefix + 3 * u64 * 2));
    assert_eq!(2, capacity_from(prefix + 3 * u64 * 2 + 1));
    assert_eq!(2, capacity_from(prefix + 3 * u64 * 3 - 1));

    assert_eq!(3, capacity_from(prefix + 3 * u64 * 3));
}

#[test]
#[cfg_attr(miri, ignore)]
#[rustfmt::skip::macros(assert_eq)]
fn u8_u16_u32_buffer_size() {
    type SoA = (u8, u16, u32);

    let context = ();
    let buffer_size = |capacity| buffer_size::<SoA>(&context, capacity);

    let u8 = size_of::<u8>();
    let u16 = size_of::<u16>();
    let u32 = size_of::<u32>();
    let prefix = prefix_size::<SoA>(&context);

    assert_eq!(buffer_size(0), 0);
    assert_eq!(buffer_size(1), prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1));
    assert_eq!(buffer_size(2), prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2));
    assert_eq!(buffer_size(3), prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3));
    assert_eq!(buffer_size(4), prefix + (u8 * 4) + 0 + (u16 * 4) + 0 + (u32 * 4));
    assert_eq!(buffer_size(5), prefix + (u8 * 5) + 1 + (u16 * 5) + 0 + (u32 * 5));
    assert_eq!(buffer_size(6), prefix + (u8 * 6) + 0 + (u16 * 6) + 2 + (u32 * 6));
    assert_eq!(buffer_size(7), prefix + (u8 * 7) + 1 + (u16 * 7) + 2 + (u32 * 7));
    assert_eq!(buffer_size(8), prefix + (u8 * 8) + 0 + (u16 * 8) + 0 + (u32 * 8));
}

#[test]
#[cfg_attr(miri, ignore)]
#[rustfmt::skip::macros(assert_eq)]
fn u8_u16_u32_capacity_from() {
    type SoA = (u8, u16, u32);

    let context = ();
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&context, buffer_size);

    let u8 = size_of::<u8>();
    let u16 = size_of::<u16>();
    let u32 = size_of::<u32>();
    let prefix = prefix_size::<SoA>(&context);

    for buffer_size in 0..(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)) {
        assert_eq!(0, capacity_from(buffer_size));
    }

    assert_eq!(1, capacity_from(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)));
    assert_eq!(1, capacity_from(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1) + 1));
    assert_eq!(1, capacity_from(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) - 1));

    assert_eq!(2, capacity_from(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2)));
    assert_eq!(2, capacity_from(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) + 1));
    assert_eq!(2, capacity_from(prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3) - 1));

    assert_eq!(3, capacity_from(prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3)));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u32_u16_u8_buffer_size() {
    type SoA = (u32, u16, u8);
    type Ref = (u8, u16, u32);

    let ref_buffer_size = |capacity| buffer_size::<Ref>(&(), capacity);
    let buffer_size = |capacity| buffer_size::<SoA>(&(), capacity);

    assert_eq!(buffer_size(0), ref_buffer_size(0));
    assert_eq!(buffer_size(1), ref_buffer_size(1));
    assert_eq!(buffer_size(2), ref_buffer_size(2));
    assert_eq!(buffer_size(3), ref_buffer_size(3));
    assert_eq!(buffer_size(4), ref_buffer_size(4));
    assert_eq!(buffer_size(5), ref_buffer_size(5));
    assert_eq!(buffer_size(6), ref_buffer_size(6));
    assert_eq!(buffer_size(7), ref_buffer_size(7));
    assert_eq!(buffer_size(8), ref_buffer_size(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u32_u16_u8_capacity_from() {
    type SoA = (u32, u16, u8);
    type Ref = (u8, u16, u32);

    let ref_capacity_from = |buffer_size| capacity_from_size::<Ref>(&(), buffer_size);
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&(), buffer_size);

    for buffer_size in 0..128 {
        assert_eq!(capacity_from(buffer_size), ref_capacity_from(buffer_size));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u8_u16_u8_buffer_size() {
    type SoA = (u8, u16, u8);
    type Ref = (u8, u8, u16);

    let ref_buffer_size = |capacity| buffer_size::<Ref>(&(), capacity);
    let buffer_size = |capacity| buffer_size::<SoA>(&(), capacity);

    assert_eq!(buffer_size(0), ref_buffer_size(0));
    assert_eq!(buffer_size(1), ref_buffer_size(1));
    assert_eq!(buffer_size(2), ref_buffer_size(2));
    assert_eq!(buffer_size(3), ref_buffer_size(3));
    assert_eq!(buffer_size(4), ref_buffer_size(4));
    assert_eq!(buffer_size(5), ref_buffer_size(5));
    assert_eq!(buffer_size(6), ref_buffer_size(6));
    assert_eq!(buffer_size(7), ref_buffer_size(7));
    assert_eq!(buffer_size(8), ref_buffer_size(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u8_u16_u8_capacity_from() {
    type SoA = (u8, u16, u8);
    type Ref = (u8, u8, u16);

    let ref_capacity_from = |buffer_size| capacity_from_size::<Ref>(&(), buffer_size);
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&(), buffer_size);

    for buffer_size in 0..128 {
        assert_eq!(capacity_from(buffer_size), ref_capacity_from(buffer_size));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u8_u16_buffer_size() {
    type SoA = (u16, u8, u16);
    type Ref = (u8, u16, u16);

    let ref_buffer_size = |capacity| buffer_size::<Ref>(&(), capacity);
    let buffer_size = |capacity| buffer_size::<SoA>(&(), capacity);

    assert_eq!(buffer_size(0), ref_buffer_size(0));
    assert_eq!(buffer_size(1), ref_buffer_size(1));
    assert_eq!(buffer_size(2), ref_buffer_size(2));
    assert_eq!(buffer_size(3), ref_buffer_size(3));
    assert_eq!(buffer_size(4), ref_buffer_size(4));
    assert_eq!(buffer_size(5), ref_buffer_size(5));
    assert_eq!(buffer_size(6), ref_buffer_size(6));
    assert_eq!(buffer_size(7), ref_buffer_size(7));
    assert_eq!(buffer_size(8), ref_buffer_size(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u8_u16_capacity_from() {
    type SoA = (u16, u8, u16);
    type Ref = (u8, u16, u16);

    let ref_capacity_from = |buffer_size| capacity_from_size::<Ref>(&(), buffer_size);
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&(), buffer_size);

    for buffer_size in 0..128 {
        assert_eq!(capacity_from(buffer_size), ref_capacity_from(buffer_size));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u8_u32_buffer_size() {
    type SoA = (u16, u8, u32);
    type Ref = (u8, u16, u32);

    let ref_buffer_size = |capacity| buffer_size::<Ref>(&(), capacity);
    let buffer_size = |capacity| buffer_size::<SoA>(&(), capacity);

    assert_eq!(buffer_size(0), ref_buffer_size(0));
    assert_eq!(buffer_size(1), ref_buffer_size(1));
    assert_eq!(buffer_size(2), ref_buffer_size(2));
    assert_eq!(buffer_size(3), ref_buffer_size(3));
    assert_eq!(buffer_size(4), ref_buffer_size(4));
    assert_eq!(buffer_size(5), ref_buffer_size(5));
    assert_eq!(buffer_size(6), ref_buffer_size(6));
    assert_eq!(buffer_size(7), ref_buffer_size(7));
    assert_eq!(buffer_size(8), ref_buffer_size(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u8_u32_capacity_from() {
    type SoA = (u16, u8, u32);
    type Ref = (u8, u16, u32);

    let ref_capacity_from = |buffer_size| capacity_from_size::<Ref>(&(), buffer_size);
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&(), buffer_size);

    for buffer_size in 0..128 {
        assert_eq!(capacity_from(buffer_size), ref_capacity_from(buffer_size));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u32_u16_buffer_size() {
    type SoA = (u16, u32, u16);
    type Ref = (u16, u16, u32);

    let ref_buffer_size = |capacity| buffer_size::<Ref>(&(), capacity);
    let buffer_size = |capacity| buffer_size::<SoA>(&(), capacity);

    assert_eq!(buffer_size(0), ref_buffer_size(0));
    assert_eq!(buffer_size(1), ref_buffer_size(1));
    assert_eq!(buffer_size(2), ref_buffer_size(2));
    assert_eq!(buffer_size(3), ref_buffer_size(3));
    assert_eq!(buffer_size(4), ref_buffer_size(4));
    assert_eq!(buffer_size(5), ref_buffer_size(5));
    assert_eq!(buffer_size(6), ref_buffer_size(6));
    assert_eq!(buffer_size(7), ref_buffer_size(7));
    assert_eq!(buffer_size(8), ref_buffer_size(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u32_u16_to_capacity() {
    type SoA = (u16, u32, u16);
    type Ref = (u16, u16, u32);

    let ref_capacity_from = |buffer_size| capacity_from_size::<Ref>(&(), buffer_size);
    let capacity_from = |buffer_size| capacity_from_size::<SoA>(&(), buffer_size);

    for buffer_size in 0..128 {
        assert_eq!(capacity_from(buffer_size), ref_capacity_from(buffer_size));
    }
}
