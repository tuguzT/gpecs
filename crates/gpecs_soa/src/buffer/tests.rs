#![expect(clippy::identity_op)]

use core::alloc::Layout;

use super::{BufferData, BufferPrefix, buffer_layout_inner, capacity_from};

#[test]
#[cfg_attr(miri, ignore)]
fn u8_u8_u8_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u8, u8, u8)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let u8 = size_of::<u8>();
    let prefix = Layout::new::<BufferPrefix<(u8, u8, u8)>>()
        .align_to(align_of::<u8>())
        .unwrap()
        .pad_to_align()
        .size();

    assert_eq!(to_capacity_in_bytes(0), 0);
    assert_eq!(to_capacity_in_bytes(1), prefix + 3 * u8 * 1);
    assert_eq!(to_capacity_in_bytes(2), prefix + 3 * u8 * 2);
    assert_eq!(to_capacity_in_bytes(3), prefix + 3 * u8 * 3);
    assert_eq!(to_capacity_in_bytes(4), prefix + 3 * u8 * 4);
    assert_eq!(to_capacity_in_bytes(5), prefix + 3 * u8 * 5);
    assert_eq!(to_capacity_in_bytes(6), prefix + 3 * u8 * 6);
    assert_eq!(to_capacity_in_bytes(7), prefix + 3 * u8 * 7);
    assert_eq!(to_capacity_in_bytes(8), prefix + 3 * u8 * 8);
    assert_eq!(to_capacity_in_bytes(9), prefix + 3 * u8 * 9);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u8_u8_u8_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u8, u8, u8)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u8, u8, u8)>(&Default::default(), buffer_layout)
    };
    let u8 = size_of::<u8>();
    let prefix = Layout::new::<BufferPrefix<(u8, u8, u8)>>()
        .align_to(align_of::<u8>())
        .unwrap()
        .pad_to_align()
        .size();

    for capacity_in_bytes in 0..(prefix + 3 * u8 * 1) {
        assert_eq!(to_capacity(capacity_in_bytes), 0);
    }

    assert_eq!(1, to_capacity(prefix + 3 * u8 * 1));
    assert_eq!(1, to_capacity(prefix + 3 * u8 * 1 + 1));
    assert_eq!(1, to_capacity(prefix + 3 * u8 * 2 - 1));

    assert_eq!(2, to_capacity(prefix + 3 * u8 * 2));
    assert_eq!(2, to_capacity(prefix + 3 * u8 * 2 + 1));
    assert_eq!(2, to_capacity(prefix + 3 * u8 * 3 - 1));

    assert_eq!(3, to_capacity(prefix + 3 * u8 * 3));
    assert_eq!(3, to_capacity(prefix + 3 * u8 * 3 + 1));
    assert_eq!(3, to_capacity(prefix + 3 * u8 * 4 - 1));

    assert_eq!(4, to_capacity(prefix + 3 * u8 * 4));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u16_u16_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u16, u16, u16)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let u16 = size_of::<u16>();
    let prefix = Layout::new::<BufferPrefix<(u16, u16, u16)>>()
        .align_to(align_of::<u16>())
        .unwrap()
        .pad_to_align()
        .size();

    assert_eq!(to_capacity_in_bytes(0), 0);
    assert_eq!(to_capacity_in_bytes(1), prefix + 3 * u16 * 1);
    assert_eq!(to_capacity_in_bytes(2), prefix + 3 * u16 * 2);
    assert_eq!(to_capacity_in_bytes(3), prefix + 3 * u16 * 3);
    assert_eq!(to_capacity_in_bytes(4), prefix + 3 * u16 * 4);
    assert_eq!(to_capacity_in_bytes(5), prefix + 3 * u16 * 5);
    assert_eq!(to_capacity_in_bytes(6), prefix + 3 * u16 * 6);
    assert_eq!(to_capacity_in_bytes(7), prefix + 3 * u16 * 7);
    assert_eq!(to_capacity_in_bytes(8), prefix + 3 * u16 * 8);
    assert_eq!(to_capacity_in_bytes(9), prefix + 3 * u16 * 9);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u16_u16_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u16, u16, u16)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u16, u16, u16)>(&Default::default(), buffer_layout)
    };
    let u16 = size_of::<u16>();
    let prefix = Layout::new::<BufferPrefix<(u16, u16, u16)>>()
        .align_to(align_of::<u16>())
        .unwrap()
        .pad_to_align()
        .size();

    for capacity_in_bytes in 0..(prefix + 3 * u16 * 1) {
        assert_eq!(to_capacity(capacity_in_bytes), 0);
    }

    assert_eq!(1, to_capacity(prefix + 3 * u16 * 1));
    assert_eq!(1, to_capacity(prefix + 3 * u16 * 1 + 1));
    assert_eq!(1, to_capacity(prefix + 3 * u16 * 2 - 1));

    assert_eq!(2, to_capacity(prefix + 3 * u16 * 2));
    assert_eq!(2, to_capacity(prefix + 3 * u16 * 2 + 1));
    assert_eq!(2, to_capacity(prefix + 3 * u16 * 3 - 1));

    assert_eq!(3, to_capacity(prefix + 3 * u16 * 3));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u32_u32_u32_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u32, u32, u32)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let u32 = size_of::<u32>();
    let prefix = Layout::new::<BufferPrefix<(u32, u32, u32)>>()
        .align_to(align_of::<u32>())
        .unwrap()
        .pad_to_align()
        .size();

    assert_eq!(to_capacity_in_bytes(0), 0);
    assert_eq!(to_capacity_in_bytes(1), prefix + 3 * u32 * 1);
    assert_eq!(to_capacity_in_bytes(2), prefix + 3 * u32 * 2);
    assert_eq!(to_capacity_in_bytes(3), prefix + 3 * u32 * 3);
    assert_eq!(to_capacity_in_bytes(4), prefix + 3 * u32 * 4);
    assert_eq!(to_capacity_in_bytes(5), prefix + 3 * u32 * 5);
    assert_eq!(to_capacity_in_bytes(6), prefix + 3 * u32 * 6);
    assert_eq!(to_capacity_in_bytes(7), prefix + 3 * u32 * 7);
    assert_eq!(to_capacity_in_bytes(8), prefix + 3 * u32 * 8);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u32_u32_u32_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u32, u32, u32)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u32, u32, u32)>(&Default::default(), buffer_layout)
    };
    let u32 = size_of::<u32>();
    let prefix = Layout::new::<BufferPrefix<(u32, u32, u32)>>()
        .align_to(align_of::<u32>())
        .unwrap()
        .pad_to_align()
        .size();

    for capacity_in_bytes in 0..(prefix + 3 * u32 * 1) {
        assert_eq!(to_capacity(capacity_in_bytes), 0);
    }

    assert_eq!(1, to_capacity(prefix + 3 * u32 * 1));
    assert_eq!(1, to_capacity(prefix + 3 * u32 * 1 + 1));
    assert_eq!(1, to_capacity(prefix + 3 * u32 * 2 - 1));

    assert_eq!(2, to_capacity(prefix + 3 * u32 * 2));
    assert_eq!(2, to_capacity(prefix + 3 * u32 * 2 + 1));
    assert_eq!(2, to_capacity(prefix + 3 * u32 * 3 - 1));

    assert_eq!(3, to_capacity(prefix + 3 * u32 * 3));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u64_u64_u64_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u64, u64, u64)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let u64 = size_of::<u64>();
    let prefix = Layout::new::<BufferPrefix<(u64, u64, u64)>>()
        .align_to(align_of::<u64>())
        .unwrap()
        .pad_to_align()
        .size();

    assert_eq!(to_capacity_in_bytes(0), 0);
    assert_eq!(to_capacity_in_bytes(1), prefix + 3 * u64 * 1);
    assert_eq!(to_capacity_in_bytes(2), prefix + 3 * u64 * 2);
    assert_eq!(to_capacity_in_bytes(3), prefix + 3 * u64 * 3);
    assert_eq!(to_capacity_in_bytes(4), prefix + 3 * u64 * 4);
    assert_eq!(to_capacity_in_bytes(5), prefix + 3 * u64 * 5);
    assert_eq!(to_capacity_in_bytes(6), prefix + 3 * u64 * 6);
    assert_eq!(to_capacity_in_bytes(7), prefix + 3 * u64 * 7);
    assert_eq!(to_capacity_in_bytes(8), prefix + 3 * u64 * 8);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u64_u64_u64_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u64, u64, u64)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u64, u64, u64)>(&Default::default(), buffer_layout)
    };
    let u64 = size_of::<u64>();
    let prefix = Layout::new::<BufferPrefix<(u64, u64, u64)>>()
        .align_to(align_of::<u64>())
        .unwrap()
        .pad_to_align()
        .size();

    for capacity_in_bytes in 0..(prefix + 3 * u64 * 1) {
        assert_eq!(to_capacity(capacity_in_bytes), 0);
    }

    assert_eq!(1, to_capacity(prefix + 3 * u64 * 1));
    assert_eq!(1, to_capacity(prefix + 3 * u64 * 1 + 1));
    assert_eq!(1, to_capacity(prefix + 3 * u64 * 2 - 1));

    assert_eq!(2, to_capacity(prefix + 3 * u64 * 2));
    assert_eq!(2, to_capacity(prefix + 3 * u64 * 2 + 1));
    assert_eq!(2, to_capacity(prefix + 3 * u64 * 3 - 1));

    assert_eq!(3, to_capacity(prefix + 3 * u64 * 3));
}

#[test]
#[cfg_attr(miri, ignore)]
#[rustfmt::skip::macros(assert_eq)]
fn u8_u16_u32_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u8, u16, u32)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let u8 = size_of::<u8>();
    let u16 = size_of::<u16>();
    let u32 = size_of::<u32>();
    let prefix = Layout::new::<BufferPrefix<(u8, u16, u32)>>()
        .align_to(align_of::<u32>())
        .unwrap()
        .pad_to_align()
        .size();

    assert_eq!(to_capacity_in_bytes(0), 0);
    assert_eq!(to_capacity_in_bytes(1), prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1));
    assert_eq!(to_capacity_in_bytes(2), prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2));
    assert_eq!(to_capacity_in_bytes(3), prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3));
    assert_eq!(to_capacity_in_bytes(4), prefix + (u8 * 4) + 0 + (u16 * 4) + 0 + (u32 * 4));
    assert_eq!(to_capacity_in_bytes(5), prefix + (u8 * 5) + 1 + (u16 * 5) + 0 + (u32 * 5));
    assert_eq!(to_capacity_in_bytes(6), prefix + (u8 * 6) + 0 + (u16 * 6) + 2 + (u32 * 6));
    assert_eq!(to_capacity_in_bytes(7), prefix + (u8 * 7) + 1 + (u16 * 7) + 2 + (u32 * 7));
    assert_eq!(to_capacity_in_bytes(8), prefix + (u8 * 8) + 0 + (u16 * 8) + 0 + (u32 * 8));
}

#[test]
#[cfg_attr(miri, ignore)]
#[rustfmt::skip::macros(assert_eq)]
fn u8_u16_u32_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u8, u16, u32)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u8, u16, u32)>(&Default::default(), buffer_layout)
    };
    let u8 = size_of::<u8>();
    let u16 = size_of::<u16>();
    let u32 = size_of::<u32>();
    let prefix = Layout::new::<BufferPrefix<(u8, u16, u32)>>()
        .align_to(align_of::<u32>())
        .unwrap()
        .pad_to_align()
        .size();

    for capacity_in_bytes in 0..(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)) {
        dbg!(capacity_in_bytes);
        assert_eq!(to_capacity(capacity_in_bytes), 0);
    }

    assert_eq!(1, to_capacity(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)));
    assert_eq!(1, to_capacity(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1) + 1));
    assert_eq!(1, to_capacity(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) - 1));

    assert_eq!(2, to_capacity(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2)));
    assert_eq!(2, to_capacity(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) + 1));
    assert_eq!(2, to_capacity(prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3) - 1));

    assert_eq!(3, to_capacity(prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3)));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u32_u16_u8_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u32, u16, u8)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let efficient_to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u8, u16, u32)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };

    assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
    assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
    assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
    assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
    assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
    assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
    assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
    assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
    assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u32_u16_u8_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u32, u16, u8)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u32, u16, u8)>(&Default::default(), buffer_layout)
    };
    let efficient_to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u8, u16, u32)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u8, u16, u32)>(&Default::default(), buffer_layout)
    };

    for capacity_in_bytes in 0..128 {
        assert_eq!(
            to_capacity(capacity_in_bytes),
            efficient_to_capacity(capacity_in_bytes),
        );
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u8_u16_u8_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u8, u16, u8)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let efficient_to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u8, u8, u16)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };

    assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
    assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
    assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
    assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
    assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
    assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
    assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
    assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
    assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u8_u16_u8_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u8, u16, u8)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u8, u16, u8)>(&Default::default(), buffer_layout)
    };
    let efficient_to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u8, u8, u16)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u8, u8, u16)>(&Default::default(), buffer_layout)
    };

    for capacity_in_bytes in 0..128 {
        assert_eq!(
            to_capacity(capacity_in_bytes),
            efficient_to_capacity(capacity_in_bytes),
        );
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u8_u16_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u16, u8, u16)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let efficient_to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u8, u16, u16)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };

    assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
    assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
    assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
    assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
    assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
    assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
    assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
    assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
    assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u8_u16_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u16, u8, u16)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u16, u8, u16)>(&Default::default(), buffer_layout)
    };
    let efficient_to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u8, u16, u16)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u8, u16, u16)>(&Default::default(), buffer_layout)
    };

    for capacity_in_bytes in 0..128 {
        assert_eq!(
            to_capacity(capacity_in_bytes),
            efficient_to_capacity(capacity_in_bytes),
        );
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u8_u32_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u16, u8, u32)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let efficient_to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u8, u16, u32)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };

    assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
    assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
    assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
    assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
    assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
    assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
    assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
    assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
    assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u8_u32_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u16, u8, u32)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u16, u8, u32)>(&Default::default(), buffer_layout)
    };
    let efficient_to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u8, u16, u32)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u8, u16, u32)>(&Default::default(), buffer_layout)
    };

    for capacity_in_bytes in 0..128 {
        assert_eq!(
            to_capacity(capacity_in_bytes),
            efficient_to_capacity(capacity_in_bytes),
        );
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u32_u16_to_capacity_in_bytes() {
    let to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u16, u32, u16)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };
    let efficient_to_capacity_in_bytes = |capacity| {
        buffer_layout_inner::<(u16, u16, u32)>(&Default::default(), capacity)
            .unwrap()
            .size()
    };

    assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
    assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
    assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
    assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
    assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
    assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
    assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
    assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
    assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_u32_u16_to_capacity() {
    let to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u16, u32, u16)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u16, u32, u16)>(&Default::default(), buffer_layout)
    };
    let efficient_to_capacity = |capacity_in_bytes| {
        let align = align_of::<BufferData<(u16, u16, u32)>>();
        let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
        capacity_from::<(u16, u16, u32)>(&Default::default(), buffer_layout)
    };

    for capacity_in_bytes in 0..128 {
        assert_eq!(
            to_capacity(capacity_in_bytes),
            efficient_to_capacity(capacity_in_bytes),
        );
    }
}
