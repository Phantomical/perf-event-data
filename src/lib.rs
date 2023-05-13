//! Parse data emitted by the linux `perf_event_open` syscall.
//!
//! To parse data you will need the types in the [`parse`] module.
//!
//! # Example
//! ```
//! # fn main() -> perf_event_data::parse::Result<()> {
//! use perf_event_data::parse::{ParseConfig, Parser};
//! use perf_event_data::endian::Native;
//! use perf_event_data::Record;
//!
//! let data: &[u8] = &[
//!     1, 0, 0, 0,
//!     40, 0,
//!     0, 0,
//!     22, 76, 1, 0, 23, 76, 1, 0, 0, 160, 72, 150, 79, 127, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0,
//!     160, 72, 150, 79, 127, 0, 0, 47, 47, 97, 110, 111, 110, 0, 0,
//! ];
//! let config = ParseConfig::<Native>::default();
//! let mut parser = Parser::new(data, config);
//! let record: Record = parser.parse()?;
//! 
//! let mmap = match record {
//!     Record::Mmap(mmap) => mmap,
//!     _ => panic!("expected a MMAP record")
//! };
//! # Ok(())
//! # }
//! ```

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
    pub(crate) use crate::parse::{Parse, ParseBuf, ParseResult, Parser};
}

pub use crate::flags::*;
pub use crate::records::*;
pub use crate::visitor::{RecordMetadata, Visitor};
