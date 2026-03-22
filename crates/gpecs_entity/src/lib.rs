//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

pub use self::entity::{Entity, EntityEpoch, EpochFromU32Error};

mod entity;
