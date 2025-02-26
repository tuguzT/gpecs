//! SPIR-V analysis by [Threaded Many-core Memory (TMM) model][tmm].
//!
//! [tmm]: https://www.sciencedirect.com/science/article/pii/S0167739X13001349

use std::{
    num::{NonZeroU32, NonZeroU64},
    time::Duration,
};

use crate::asymptotic::Expr;

/// The important characteristics of a highly-threaded, many-core architecture
/// on which algorithms (or programs) are executed.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ArchParams {
    /// *L* - time for a global memory access.
    ///
    /// The latency for accessing the slow memory
    /// (the global memory which is shared by all the core groups).
    pub global_memory_access_time: Duration,
    /// *P* - number of cores (or processors) in the machine.
    pub cores: NonZeroU64,
    /// *C* - memory access width.
    ///
    /// The number of words that can be read from slow memory
    /// to fast memory in one memory transfer.
    pub memory_access_width: NonZeroU32,
    /// *Z* - size of fast local memory per core group.
    pub local_memory_size_per_group: u64,
    /// *X* - hardware limit on number of threads per core.
    ///
    /// The number of threads an algorithm is allowed to generate per core.
    ///
    /// This limit is enforced due to many different constraints,
    /// such as constraints on the number of registers each thread uses
    /// and an explicit constraint on the number of threads.
    pub threads_per_core: NonZeroU32,
    // /// *Q* - number of cores per core group.
    // ///
    // /// Sometimes a core group can have a single core.
    // /// In this case, a many-core machine looks very much like
    // /// a multi-core machine with a large number of low-overhead hardware threads.
    // // FIXME: decide if we should remove this (it is not used anywhere later)
    // pub cores_per_group: NonZeroU64,
}

/// The parameters of the algorithm (or program)
/// which is executed on a highly-threaded, many-core architecture.
pub struct ProgramParams {
    /// *T1* - the work, or total number of operations.
    ///
    /// The total number of operations that the program must perform (including fast memory accesses).
    pub work: Expr,
    /// *T∞* - the span, or the number of operations on the critical path.
    pub span: Expr,
    /// *M* - number of global memory operations.
    ///
    /// Note that this is the total number of operations, not total number of accesses.
    /// Since many-core machines often transfer data in large chunks, multiple
    /// memory accesses can combine into one memory transfer.
    pub global_memory_accesses: Expr,
    /// *τ* - number of threads per core.
    ///
    /// There is an assumption that the work is perfectly distributed among cores.
    /// Therefore, the total number of threads in the system is *T · P*.
    ///
    /// On highly-threaded, many-core architectures, thread switching is used to hide memory latency.
    /// Therefore, it is beneficial to create as many threads as possible.
    /// However, the maximum number of threads is limited by both the hardware and the program.
    /// The software limitation has to do with parallelism, the number of threads per core is limited by *τ ≤ T1 / (T∞ · P)*.
    /// The hardware limits *τ ≤ X*.
    pub threads_per_core: NonZeroU32,
    // /// *S* - amount of local memory used per thread.
    // ///
    // /// *S* and *τ* are related parameters, since there is a limited amount of local memory in the system.
    // /// The number of threads per core is at most *τ ≤ Z / (Q · S)*.
    // // FIXME: decide if we should remove this (it is not used anywhere later)
    // pub local_memory_size_per_thread: u64,
}
