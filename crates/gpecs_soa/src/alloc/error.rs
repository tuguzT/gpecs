use core::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Display},
};

/// The error type for `try_reserve` methods.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TryReserveError {
    kind: TryReserveErrorKind,
}

impl TryReserveError {
    /// Details about the allocation that caused the error.
    #[inline]
    #[must_use]
    pub fn kind(&self) -> TryReserveErrorKind {
        let Self { kind } = self;
        kind.clone()
    }
}

/// Details of the allocation that caused a [`TryReserveError`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TryReserveErrorKind {
    /// Error due to the computed capacity exceeding the collection's maximum
    /// (usually `isize::MAX` bytes).
    CapacityOverflow,

    /// The memory allocator returned an error.
    AllocError {
        /// The layout of allocation request that failed.
        layout: Layout,

        #[doc(hidden)]
        non_exhaustive: (),
    },
}

impl From<TryReserveErrorKind> for TryReserveError {
    #[inline]
    fn from(kind: TryReserveErrorKind) -> Self {
        Self { kind }
    }
}

impl From<LayoutError> for TryReserveErrorKind {
    /// Always evaluates to [`TryReserveErrorKind::CapacityOverflow`].
    #[inline]
    fn from(_: LayoutError) -> Self {
        TryReserveErrorKind::CapacityOverflow
    }
}

impl Display for TryReserveError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("memory allocation failed")?;

        let Self { kind } = self;
        let reason = match kind {
            TryReserveErrorKind::CapacityOverflow => {
                " because the computed capacity exceeded the collection's maximum"
            }
            TryReserveErrorKind::AllocError { .. } => {
                " because the memory allocator returned an error"
            }
        };
        fmt.write_str(reason)
    }
}

impl Error for TryReserveError {}

#[inline]
pub fn alloc_error(layout: Layout) -> TryReserveErrorKind {
    TryReserveErrorKind::AllocError {
        layout,
        non_exhaustive: (),
    }
}
