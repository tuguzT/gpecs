use core::{
    fmt::{self, Display},
    num::Wrapping,
    ops::Add,
};

pub trait Key: Copy + Ord {
    type Epoch: Epoch;

    fn new(sparse_index: usize, epoch: Self::Epoch) -> Self;

    fn sparse_index(self) -> usize;

    fn epoch(self) -> Self::Epoch;
}

impl Key for usize {
    type Epoch = NoEpoch;

    fn new(sparse_index: usize, _: Self::Epoch) -> Self {
        sparse_index
    }

    fn sparse_index(self) -> usize {
        self
    }

    fn epoch(self) -> Self::Epoch {
        Default::default()
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct EpochKey<E = usize> {
    pub sparse_index: usize,
    pub epoch: E,
}

impl<E> Display for EpochKey<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            sparse_index,
            epoch,
        } = self;

        write!(f, "{sparse_index}v{epoch}")
    }
}

impl<E> Key for EpochKey<E>
where
    E: Epoch,
{
    type Epoch = E;

    fn new(sparse_index: usize, epoch: Self::Epoch) -> Self {
        EpochKey {
            sparse_index,
            epoch,
        }
    }

    fn sparse_index(self) -> usize {
        let Self { sparse_index, .. } = self;
        sparse_index
    }

    fn epoch(self) -> Self::Epoch {
        let Self { epoch, .. } = self;
        epoch
    }
}

impl From<usize> for EpochKey<NoEpoch> {
    fn from(sparse_index: usize) -> Self {
        EpochKey {
            sparse_index,
            epoch: NoEpoch::default(),
        }
    }
}

impl From<EpochKey<NoEpoch>> for usize {
    fn from(value: EpochKey<NoEpoch>) -> Self {
        let EpochKey { sparse_index, .. } = value;
        sparse_index
    }
}

pub trait Epoch: Copy + Ord + Default {
    fn next(self) -> Self;
}

pub type NoEpoch = ();

impl Epoch for NoEpoch {
    fn next(self) -> Self {
        Default::default()
    }
}

impl Epoch for u8 {
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for u16 {
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for u32 {
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for u64 {
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for u128 {
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for usize {
    fn next(self) -> Self {
        self + 1
    }
}

impl<T> Epoch for Wrapping<T>
where
    T: Epoch + From<u8>,
    Self: Add<Output = Self>,
{
    fn next(self) -> Self {
        self + Self(1.into())
    }
}
