#[macro_use]
extern crate derive_destructure;

pub mod telescope; // TODO: should this be public? this should really be its own crate
pub mod trees;
pub mod data;
pub mod methods;
pub mod example;

pub use data::Data;
pub use trees::*;
pub use methods::*;
