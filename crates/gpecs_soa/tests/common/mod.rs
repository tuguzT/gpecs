#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
#[repr(align(16))]
pub struct ZST1;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct ZST2(pub ());

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
#[repr(align(4))]
pub struct ZST3 {
    pub empty: (),
}
