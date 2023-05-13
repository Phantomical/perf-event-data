//! Rust structs for records emitted by `perf` and `perf_event_open`.

#![warn(missing_docs)]

// Needs to be first so other modules can see the macros.
#[macro_use]
mod macros;

mod config;
mod cowutils;
pub mod endian;
mod error;
mod flags;
pub mod parse;
mod parsebuf;
mod records;
mod visitor;

mod prelude {
    #[allow(unused_imports)]
    pub(crate) use crate::config::ParseConfig;
    pub(crate) use crate::endian::Endian;
    pub(crate) use crate::error::ErrorKind;
    pub(crate) use crate::flags::{ReadFormat, SampleFlags};
    pub(crate) use crate::parse::{Parse, ParseBuf, Parser, Result};
}

pub use crate::flags::*;
pub use crate::records::*;
pub use crate::visitor::{RecordMetadata, Visitor};
