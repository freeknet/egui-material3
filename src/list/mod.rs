// ============================================================================
// Module:       list
// Description:  Material Design 3 List Components.
//
// Dependencies: self::rendering, self::types
// ============================================================================

//! Material Design 3 List Components

mod builder;
mod rendering;
mod types;

#[cfg(test)]
mod tests;

pub use rendering::{list, list_item};
pub use types::*;
