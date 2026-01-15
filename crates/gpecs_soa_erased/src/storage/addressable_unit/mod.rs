mod cpu;
mod gpu;

/// Marker trait for an addressible unit of memory for a given target (CPU or GPU).
pub trait AddressableUnit: Copy + Default + 'static {}

/// Marker trait indicating that self type & all of its fields
/// could be addressed by a given addressible unit.
pub unsafe trait AddressableBy<A>
where
    A: AddressableUnit,
{
}
