pub mod constants;
pub mod utils;
pub mod types;

#[cfg(test)]
mod tests;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{EnumCount, EnumIter, EnumString, IntoStaticStr};

use arbitrary_int::{i40, traits::Integer, u5, u18, u23, u40, u46};

pub use strum::IntoEnumIterator;


use constants::*;
use utils::*;
