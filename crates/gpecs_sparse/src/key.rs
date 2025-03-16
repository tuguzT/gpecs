use core::{
    fmt::{self, Display},
    num::Wrapping,
    ops::Add,
};

pub trait Key: Copy + Ord {
    type SparseIndex: SparseIndex;
    type Epoch: Epoch;

    fn new(sparse_index: Self::SparseIndex, epoch: Self::Epoch) -> Self;

    fn sparse_index(self) -> Self::SparseIndex;

    fn epoch(self) -> Self::Epoch;
}

impl<I> Key for I
where
    I: SparseIndex,
{
    type SparseIndex = I;
    type Epoch = NoEpoch;

    #[inline]
    fn new(sparse_index: Self::SparseIndex, _: Self::Epoch) -> Self {
        sparse_index
    }

    #[inline]
    fn sparse_index(self) -> Self::SparseIndex {
        self
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {
        Default::default()
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct EpochKey<I = usize, E = usize> {
    pub sparse_index: I,
    pub epoch: E,
}

impl<I, E> EpochKey<I, E> {
    #[inline]
    pub const fn new(sparse_index: I, epoch: E) -> Self {
        Self {
            sparse_index,
            epoch,
        }
    }

    #[inline]
    pub const fn sparse_index(&self) -> &I {
        let Self { sparse_index, .. } = self;
        sparse_index
    }

    #[inline]
    pub const fn sparse_index_mut(&mut self) -> &mut I {
        let Self { sparse_index, .. } = self;
        sparse_index
    }

    #[inline]
    pub const fn epoch(&self) -> &E {
        let Self { epoch, .. } = self;
        epoch
    }

    #[inline]
    pub const fn epoch_mut(&mut self) -> &mut E {
        let Self { epoch, .. } = self;
        epoch
    }
}

impl<I, E> Display for EpochKey<I, E>
where
    I: Display,
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

impl<I, E> Key for EpochKey<I, E>
where
    I: SparseIndex,
    E: Epoch,
{
    type SparseIndex = I;
    type Epoch = E;

    #[inline]
    fn new(sparse_index: Self::SparseIndex, epoch: Self::Epoch) -> Self {
        EpochKey::new(sparse_index, epoch)
    }

    #[inline]
    fn sparse_index(self) -> Self::SparseIndex {
        *EpochKey::sparse_index(&self)
    }

    #[inline]
    fn epoch(self) -> Self::Epoch {
        *EpochKey::epoch(&self)
    }
}

impl From<usize> for EpochKey<usize, NoEpoch> {
    #[inline]
    fn from(sparse_index: usize) -> Self {
        EpochKey {
            sparse_index,
            epoch: NoEpoch::default(),
        }
    }
}

impl From<EpochKey<usize, NoEpoch>> for usize {
    #[inline]
    fn from(value: EpochKey<usize, NoEpoch>) -> Self {
        let EpochKey { sparse_index, .. } = value;
        sparse_index
    }
}

pub trait SparseIndex: Copy + Ord + Default + TryFrom<usize> + TryInto<usize> {}

impl SparseIndex for u8 {}
impl SparseIndex for u16 {}
impl SparseIndex for u32 {}
impl SparseIndex for u64 {}
impl SparseIndex for u128 {}
impl SparseIndex for usize {}

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
