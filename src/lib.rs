//! Parse data emitted by the linux `perf_event_open` syscall.
//!
//! # Important Types
//! - [`Record`] is an enum that can parse any known record type emitted by
//!   [`perf_event_open`]. You will likely want to start here.
//! - The [`parse`] module has the [`Parser`] and [`ParseConfig`] types which
//!   you will need in order to parse anything.
//!
//! [`perf_event_open`]: https://man7.org/linux/man-pages/man2/perf_event_open.2.html
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
//!     0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, 0x16, 0x4C, 0x01, 0x00, 0x17, 0x4C, 0x01,
//!     0x00, 0x00, 0xA0, 0x48, 0x96, 0x4F, 0x7F, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
//!     0x00, 0x00, 0x00, 0xA0, 0x48, 0x96, 0x4F, 0x7F, 0x00, 0x00, 0x2F, 0x2F, 0x61, 0x6E, 0x6F,
//!     0x6E, 0x00, 0x00,
//! ];
//!
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
//!
//! # Crate Orginization
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
//! # Parsing `perf.data` files
//! This crate doesn't yet have support for this, although it could be used as
//! part of implementing a larger parser. If you would like to do this please
//! open an issue!

#![warn(missing_docs)]
// bitflags generates this all over the place so better to silence it.
#![allow(clippy::assign_op_pattern)]

// Needs to be first so other modules can see the macros.
#[macro_use]
mod macros;

mod config;
pub mod endian;
mod error;
mod flags;
mod impls;
pub mod parse;
mod parsebuf;
mod records;
mod util;
mod visitor;

mod prelude {
    #[allow(unused_imports)]
    pub(crate) use crate::config::ParseConfig;
    pub(crate) use crate::endian::Endian;
    pub(crate) use crate::error::ErrorKind;
    pub(crate) use crate::flags::{ReadFormat, SampleFlags};
    pub(crate) use crate::parse::{Parse, ParseBuf, ParseResult, Parser};
    pub(crate) use c_enum::c_enum;
}

pub use crate::flags::*;
pub use crate::records::*;
pub use crate::visitor::{RecordMetadata, Visitor};

/// Common data used in doctests.
///
/// This way it doesn't need to be repeated multiple times unless we want to
/// show it as part of the doc test.
///
/// It is also used to verify that the examples within the README work.
#[doc(hidden)]
pub mod doctest {
    #[doc = include_str!("../README.md")]
    pub mod readme {}

    pub const MMAP: &[u8] = &[
        0x01, 0x00, 0x00, 0x00, // type (MMAP)
        0x00, 0x00, // misc
        0x30, 0x00, // size
        0x16, 0x4C, 0x01, 0x00, // pid
        0x17, 0x4C, 0x01, 0x00, // tid
        0x00, 0xA0, 0x48, 0x96, 0x4F, 0x7F, 0x00, 0x00, // addr
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // len
        0x00, 0xA0, 0x48, 0x96, 0x4F, 0x7F, 0x00, 0x00, // pgoff
        0x2F, 0x2F, 0x61, 0x6E, 0x6F, 0x6E, 0x00, 0x00, // filename
    ];

    pub const CUSTOM_SAMPLE: &[u8] = &[
        0x09, 0x00, 0x00, 0x00, // type (SAMPLE)
        0x00, 0x00, // misc
        0x48, 0x00, // size
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
