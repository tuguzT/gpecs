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

    #[inline]
    fn new(sparse_index: usize, _: Self::Epoch) -> Self {
        sparse_index
    }

    #[inline]
    fn sparse_index(self) -> usize {
        self
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {
        Default::default()
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct EpochKey<E = usize> {
    pub sparse_index: usize,
    pub epoch: E,
}

impl<E> EpochKey<E> {
    #[inline]
    pub const fn new(sparse_index: usize, epoch: E) -> Self {
        Self {
            sparse_index,
            epoch,
        }
    }

    #[inline]
    pub const fn sparse_index(&self) -> usize {
        let Self { sparse_index, .. } = self;
        *sparse_index
    }

    #[inline]
    pub fn sparse_index_mut(&mut self) -> &mut usize {
        let Self { sparse_index, .. } = self;
        sparse_index
    }

    #[inline]
    pub const fn epoch(&self) -> &E {
        let Self { epoch, .. } = self;
        epoch
    }

    #[inline]
    pub fn epoch_mut(&mut self) -> &mut E {
        let Self { epoch, .. } = self;
        epoch
    }
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

    #[inline]
    fn new(sparse_index: usize, epoch: Self::Epoch) -> Self {
        EpochKey::new(sparse_index, epoch)
    }

    #[inline]
    fn sparse_index(self) -> usize {
        EpochKey::sparse_index(&self)
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {
        *EpochKey::epoch(&self)
    }
}

impl From<usize> for EpochKey<NoEpoch> {
    #[inline]
    fn from(sparse_index: usize) -> Self {
        EpochKey {
            sparse_index,
            epoch: NoEpoch::default(),
        }
    }
}

impl From<EpochKey<NoEpoch>> for usize {
    #[inline]
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
    #[inline]
    fn next(self) -> Self {
        Default::default()
    }
}

impl Epoch for u8 {
    #[inline]
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for u16 {
    #[inline]
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for u32 {
    #[inline]
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for u64 {
    #[inline]
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for u128 {
    #[inline]
    fn next(self) -> Self {
        self + 1
    }
}

impl Epoch for usize {
    #[inline]
    fn next(self) -> Self {
        self + 1
    }
}

impl<T> Epoch for Wrapping<T>
where
    T: Epoch + From<u8>,
    Self: Add<Output = Self>,
{
    #[inline]
    fn next(self) -> Self {
        let one = 1.into();
        self + Self(one)
    }
}
