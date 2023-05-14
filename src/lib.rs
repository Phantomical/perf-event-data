//! Parse data emitted by the linux `perf_event_open` syscall.
//!
//! This crate is organized like this:
//! - The root contains all the data types that records can be parsed into. This
//!   includes not only the types corresponding to the perf records but also
//!   those that make up their fields, and so on.
//! - The [`parse`][mod] module contains types and traits needed to implement
//!   parsing support. Most types exposed in the root implement [`Parse`] but to
//!   actually make use of that you will need the [`Parser`] and [`ParseConfig`]
//!   types from the [`parse`][mod] module.
//! - The [`endian`] module contains some types for converting values to the
//!   native endian. You likely won't have to interact with it other than
//!   picking one type to use when creating a [`ParseConfig`].
//!
//! [mod]: crate::parse
//! [`Parse`]: crate::parse::Parse
//! [`Parser`]: crate::parse::Parser
//! [`ParseConfig`]: crate::parse::ParseConfig
//!
//! # Example
//! Parsing a mmap record directly from its raw byte format.
//! ```
//! # fn main() -> perf_event_data::parse::ParseResult<()> {
//! use perf_event_data::endian::Little;
//! use perf_event_data::parse::{ParseConfig, Parser};
//! use perf_event_data::Record;
//!
//! let data: &[u8] = &[
//!     1, 0, 0, 0, 0, 0, 48, 0, 22, 76, 1, 0, 23, 76, 1, 0, 0, 160, 72, 150, 79, 127, 0, 0, 0, 16,
//!     0, 0, 0, 0, 0, 0, 0, 160, 72, 150, 79, 127, 0, 0, 47, 47, 97, 110, 111, 110, 0, 0,
//! ];
//! let config = ParseConfig::<Little>::default();
//! let mut parser = Parser::new(data, config);
//! let record: Record = parser.parse()?;
//!
//! let mmap = match record {
//!     Record::Mmap(mmap) => mmap,
//!     _ => panic!("expected a MMAP record"),
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

/// Common data used in doctests.
///
/// This way it doesn't need to be repeated multiple times unless we want to
/// show it as part of the doc test.
#[doc(hidden)]
pub mod doctest {
    pub const MMAP: &[u8] = &[
        1, 0, 0, 0, 0, 0, 48, 0, 22, 76, 1, 0, 23, 76, 1, 0, 0, 160, 72, 150, 79, 127, 0, 0, 0, 16,
        0, 0, 0, 0, 0, 0, 0, 160, 72, 150, 79, 127, 0, 0, 47, 47, 97, 110, 111, 110, 0, 0,
    ];

    pub const CUSTOM_SAMPLE: &[u8] = &[
        0x09, 0x00, 0x00, 0x00, // type
        0x00, 0x00, // misc
        0x49, 0x00, // size
        0x10, 0x12, 0x33, 0x48, 0x99, 0x1A, 0x2B, 0x3C, // ip
        0x02, 0x00, 0x00, 0x00, // pid
        0x03, 0x00, 0x00, 0x00, // tid
        0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // nr
        0x01, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // stack1
        0x02, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // stack2
        0x03, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // stack3
        0x04, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // stack4
        0xC9, 0x40, 0x65, 0x00, 0x00, 0x65, 0x40, 0xC9, // cgroup
    ];
}
