pub mod push_many;
pub mod with_capacity;
pub mod work;

pub type Zero = ();
pub type Tiny = (u32,);
pub type Small = (f64, f64, f64);
pub type Medium = (Small, Small, Small);
pub type Big = (Small, Small, [usize; 18], String, String);
pub type Large = ([u64; 32], [u64; 32], [u64; 32], [u64; 32], [u64; 32]);

pub const SOA_FUNCTION_NAME: &str = "SoA (mine)";
pub const AOS_FUNCTION_NAME: &str = "AoS (std)";
