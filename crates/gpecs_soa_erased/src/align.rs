pub struct Unaligned(());

pub struct Aligned<Fields>(Fields);

pub trait Align: private::Sealed {
    const IS_ALIGNED: bool;
}

impl Align for Unaligned {
    const IS_ALIGNED: bool = false;
}

impl<Fields> Align for Aligned<Fields> {
    const IS_ALIGNED: bool = true;
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::Unaligned {}
    impl<Fields> Sealed for super::Aligned<Fields> {}
}
