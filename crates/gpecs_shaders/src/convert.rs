pub fn u32_to_usize(num: u32) -> usize {
    let Ok(num) = num.try_into() else {
        u32_to_usize_fail()
    };
    num
}

fn u32_to_usize_fail() -> ! {
    unreachable!("`u32` and `usize` are the same thing in Rust-GPU")
}
