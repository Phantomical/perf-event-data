//! Parsing interface for parsing data in to record types.
//!
//! In this crate, parsing is built on parser functions that take in a
//! [`Parser`] and produce a [`Result<T>`] where `T` is some record type. Most
//! of the time, you should be able to call [`Parser::parse`] for each of your
//! fields but [`Parser`] provides many other helper methods for when that isn't
//! enough.
//!
//! # Parsing a Record
//! The quickest and easiest way to get started is to just parse everything to
//! the [`Record`] type.
//!
//! ```
//! # fn main() -> perf_event_data::parse::ParseResult<()> {
//! use perf_event_data::endian::Little;
//! use perf_event_data::parse::{ParseConfig, Parser};
//! use perf_event_data::Record;
//!
//! let data: &[u8] = // ...
//! #       perf_event_data::doctest::MMAP;
//! let config = ParseConfig::<Little>::default();
//! let mut parser = Parser::new(data, config);
//! let record: Record = parser.parse()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Parsing Custom Types
//! The types provided in this crate aim to cover all the possible types of
//! record that can be emitted by `perf_event_open`. However, if you control the
//! configuration passed to `perf_event_open` then the types in this crate can
//! be overly general. You may find it easier to define custom types that match
//! exactly the record types you know will be generated.
//!
//! To do that you will need to implement [`Parse`] for your type, use
//! [`Parser::parse_metadata`] to parse the record frame, and then use
//! [`Parser::parse`] to parse your custom record type.
//!
//! Here we define a sample type that only parses the custom fields we care
//! about.
//! ```
//! # fn main() -> perf_event_data::parse::ParseResult<()> {
//! use perf_event_data::endian::{Endian, Little};
//! use perf_event_data::parse::{Parse, ParseBuf, ParseConfig, ParseResult, Parser};
//! use perf_event_data::Registers;
//! use perf_event_open_sys::bindings::PERF_RECORD_SAMPLE;
//!
//! struct CustomSample {
//!     pub ip: u64,
//!     pub pid: u32,
//!     pub tid: u32,
//!     pub callstack: Vec<u64>,
//!     pub cgroup: u64,
//! }
//!
//! impl<'p> Parse<'p> for CustomSample {
//!     fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
//!     where
//!         B: ParseBuf<'p>,
//!         E: Endian,
//!     {
//!         Ok(Self {
//!             ip: p.parse()?,
//!             pid: p.parse()?,
//!             tid: p.parse()?,
//!             callstack: {
//!                 let len = p.parse_u64()?;
//!                 p.parse_repeated(len as usize)?
//!             },
//!             cgroup: p.parse()?,
//!         })
//!     }
//! }
//!
//! let data: &[u8] = // ...
//! #   perf_event_data::doctest::CUSTOM_SAMPLE;
//! let attr: perf_event_open_sys::bindings::perf_event_attr = // ...
//! #   Default::default();
//! let config: ParseConfig<Little> = ParseConfig::from(attr);
//! let mut parser = Parser::new(data, config);
//!
//! let (mut p, metadata) = parser.parse_metadata()?;
//!
//! assert_eq!(metadata.ty(), PERF_RECORD_SAMPLE);
//! let sample: CustomSample = p.parse()?;
//! #
//! # assert_eq!(metadata.misc(), 0);
//! # assert_eq!(sample.ip, 0x3C2B1A9948331210, "ip did not match");
//! # assert_eq!(sample.pid, 2);
//! # assert_eq!(sample.tid, 3);
//! # assert_eq!(sample.callstack.len(), 4);
//! # assert_eq!(sample.cgroup, 0xC9406500006540C9, "gcroup did not match");
//! #
//! # Ok(())
//! # }
//! ```

use std::borrow::Cow;
use std::mem::MaybeUninit;

use perf_event_open_sys::bindings;

use crate::cowutils::CowSliceExt;
use crate::endian::Endian;
use crate::parsebuf::ParseBufCursor;
use crate::{Record, RecordMetadata, SampleId, Visitor};

used_in_docs!(Record);

pub use crate::config::ParseConfig;
pub use crate::error::{ErrorKind, ParseError, ParseResult};
pub use crate::parsebuf::{ParseBuf, ParseBufChunk};

/// A type that can be parsed
pub trait Parse<'p>: Sized {
    /// Parse `Self` using the provided [`Parser`] instance.
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>;
}

/// A [`ParseConfig`] combined with a [`ParseBuf`].
///
/// This type is the base on which all parsing in this library occurs. It has a
/// number of helper methods that do common parsing operations needed when
/// implementing [`Parse`] for a record type.
///
/// # Important Methods
/// If you are using this library to parse an perf event stream emitted either
/// by [`perf_event_open(2)`][0] or by parsing a `perf.data` file then likely
/// want one of
/// - [`parse_record`](Parser::parse_record), or,
/// - [`parse_record_with_header`](Parser::parse_record_with_header)
///
/// If you are implementing [`Parse`] for a type then you will likely be using
/// - [`parse`](Parser::parse), and,
/// - [`parse_bytes`](Parser::parse_bytes)
///
/// Other methods are provided if they were needed but those should be the main
/// ones.
///
/// [0]: https://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone)]
pub struct Parser<B, E> {
    config: ParseConfig<E>,
    data: B,
}

impl<'p, B, E> Parser<B, E>
where
    E: Endian,
    B: ParseBuf<'p>,
{
    /// Create a new parser.
    pub fn new(data: B, config: ParseConfig<E>) -> Self {
        Self { config, data }
    }

    /// Get the [`ParseConfig`] instance for this `Parser`.
    #[inline]
    pub fn config(&self) -> &ParseConfig<E> {
        &self.config
    }

    /// Get the endian configuration type.
    #[inline]
    pub fn endian(&self) -> &E {
        self.config.endian()
    }

    /// Advance the current parser by `offset` and return a new parser for the
    /// data within.
    pub(crate) fn split_at(&mut self, offset: usize) -> ParseResult<Parser<ParseBufCursor<'p>, E>> {
        let cursor = ParseBufCursor::new(&mut self.data, offset)?;
        Ok(Parser::new(cursor, self.config().clone()))
    }

    /// Calculate a maximum capacity bound for a slice of `T`.
    ///
    /// This is to prevent unbounded memory allocation when parsing untrusted
    /// input such as a file on disk. The goal is that if an input is going to
    /// cause our program to run out of memory they must at least pass a
    /// corresponding number of bytes which can be handled at a higher level.
    fn safe_capacity_bound<T>(&self) -> usize {
        const DEFAULT_LEN: usize = 16384;

        let size = std::mem::size_of::<T>();
        // No memory will be allocated, we are free to do whatever
        if size == 0 {
            return usize::MAX;
        }

        // Allow allocating at most as many elements as would fit in the buffer, or
        // 16KB, whichever is larger. This should address cases where the size of T does
        // not correspond to the number of bytes read from the buffer.
        self.data.remaining_hint().unwrap_or(DEFAULT_LEN) / size
    }

    fn parse_bytes_direct(&mut self, len: usize) -> ParseResult<Option<&'p [u8]>> {
        let chunk = match self.data.chunk()? {
            ParseBufChunk::External(chunk) => chunk,
            _ => return Ok(None),
        };

        if chunk.len() < len {
            return Ok(None);
        }

        self.data.advance(len);
        Ok(Some(&chunk[..len]))
    }

    /// Safe implementation for when we cannot preallocate the buffer.
    #[cold]
    fn parse_bytes_slow(&mut self, mut len: usize) -> ParseResult<Vec<u8>> {
        let mut bytes = Vec::with_capacity(self.safe_capacity_bound::<u8>().min(len));

        while len > 0 {
            let mut chunk = self.data.chunk()?;
            chunk.truncate(len);
            bytes.extend_from_slice(&chunk);

            let chunk_len = chunk.len();
            len -= chunk_len;
            self.data.advance(chunk_len);
        }

        Ok(bytes)
    }

    /// Directly get a reference to the next `len` bytes in the input buffer.
    pub fn parse_bytes(&mut self, len: usize) -> ParseResult<Cow<'p, [u8]>> {
        if let Some(bytes) = self.parse_bytes_direct(len)? {
            return Ok(Cow::Borrowed(bytes));
        }

        match self.data.remaining_hint() {
            Some(hint) if hint >= len => (),
            _ => return Ok(Cow::Owned(self.parse_bytes_slow(len)?)),
        }

        let mut bytes = Vec::with_capacity(len);
        self.parse_to_slice(bytes.spare_capacity_mut())?;
        unsafe { bytes.set_len(len) };
        Ok(Cow::Owned(bytes))
    }

    /// Parse a slice in its entirety. If this returns successfully then the
    /// entire slice has been initialized.
    fn parse_to_slice(&mut self, slice: &mut [MaybeUninit<u8>]) -> ParseResult<()> {
        let mut dst = slice.as_mut_ptr() as *mut u8;
        let mut len = slice.len();

        while len > 0 {
            let chunk = self.data.chunk()?;
            let chunk_len = chunk.len().min(len);

            unsafe {
                std::ptr::copy_nonoverlapping(chunk.as_ptr(), dst, chunk_len);
                dst = dst.add(chunk_len);
                len -= chunk_len;
            };

            self.data.advance(chunk_len);
        }

        Ok(())
    }

    #[cold]
    fn parse_array_slow<const N: usize>(&mut self) -> ParseResult<[u8; N]> {
        let mut array = [0u8; N];
        self.parse_to_slice(unsafe { array.align_to_mut().1 })?;
        Ok(array)
    }

    pub(crate) fn parse_array<const N: usize>(&mut self) -> ParseResult<[u8; N]> {
        let chunk = self.data.chunk()?;

        if chunk.len() < N {
            return self.parse_array_slow();
        }

        let mut array = [0u8; N];
        array.copy_from_slice(&chunk[..N]);
        self.data.advance(N);
        Ok(array)
    }

    /// Parse a type.
    ///
    /// If the type fails to parse then this parser will not be modified.
    pub fn parse<P: Parse<'p>>(&mut self) -> ParseResult<P> {
        P::parse(self)
    }

    /// Parse with an explicit parsing function.
    pub fn parse_with<F, R>(&mut self, func: F) -> ParseResult<R>
    where
        F: FnOnce(&mut Self) -> ParseResult<R>,
    {
        func(self)
    }

    /// Parse a type only if `parse` is true.
    pub fn parse_if<P: Parse<'p>>(&mut self, parse: bool) -> ParseResult<Option<P>> {
        self.parse_if_with(parse, P::parse)
    }

    /// `parse_if` but using an explicit parsing function.
    pub fn parse_if_with<F, R>(&mut self, parse: bool, func: F) -> ParseResult<Option<R>>
    where
        F: FnOnce(&mut Self) -> ParseResult<R>,
    {
        match parse {
            true => self.parse_with(func).map(Some),
            false => Ok(None),
        }
    }

    /// Parse a single byte out of the source buffer.
    pub fn parse_u8(&mut self) -> ParseResult<u8> {
        let [byte] = self.parse_array()?;
        Ok(byte)
    }

    /// Parse a `u16` out of the source data.
    pub fn parse_u16(&mut self) -> ParseResult<u16> {
        let array = self.parse_array()?;
        Ok(self.endian().convert_u16(array))
    }

    /// Parse a `u32` out of the source data.
    pub fn parse_u32(&mut self) -> ParseResult<u32> {
        let array = self.parse_array()?;
        Ok(self.endian().convert_u32(array))
    }

    /// Parse a `u64` out of the source data.
    pub fn parse_u64(&mut self) -> ParseResult<u64> {
        let array = self.parse_array()?;
        Ok(self.endian().convert_u64(array))
    }

    /// Consume the rest of the buffer and return it as a slice.
    pub fn parse_rest(&mut self) -> ParseResult<Cow<'p, [u8]>> {
        let mut bytes = self.data.chunk()?.to_cow();
        self.data.advance(bytes.len());

        loop {
            match self.data.chunk() {
                Ok(chunk) => {
                    bytes.to_mut().extend_from_slice(&chunk);

                    let len = chunk.len();
                    self.data.advance(len);
                }
                Err(e) if e.kind() == ErrorKind::Eof => break,
                Err(e) => return Err(e),
            }
        }

        Ok(bytes)
    }

    /// Parse the rest of the bytes in the buffer but trim trailing nul bytes.
    pub fn parse_rest_trim_nul(&mut self) -> ParseResult<Cow<'p, [u8]>> {
        let mut bytes = self.parse_rest()?;

        // Trim padding nul bytes from the entry.
        let mut rest = &*bytes;
        while let Some((b'\0', head)) = rest.split_last() {
            rest = head;
        }

        bytes.truncate(rest.len());
        Ok(bytes)
    }

    /// Attempt to directly transmute a slice in the source buffer to one of
    /// type T.
    ///
    /// This method will only succeed if
    /// 1. the source endianness is the same as the endianness of this program,
    ///    and
    /// 2. the buffer is properly aligned for `T`.
    ///
    /// This method is mainly meant to reduce copying when parsing the records
    /// emitted directly from the kernel. If you are parsing from a buffer read
    /// in from a file then it is unlikely that you will meet all the required
    /// preconditions.
    ///
    /// # Safety
    /// It must be valid to transmute `T` directly from bytes. The `Copy` bound
    /// is a step towards ensuring this.
    pub unsafe fn parse_slice_direct<T>(&mut self, len: usize) -> ParseResult<Option<&'p [T]>>
    where
        T: Copy,
    {
        // The current endianness is not native so reinterpreting as `T` would not be
        // valid.
        if !self.endian().is_native() {
            return Ok(None);
        }

        let byte_len = len.checked_mul(std::mem::size_of::<T>()).ok_or_else(|| {
            ParseError::custom(
                ErrorKind::InvalidRecord,
                "array length in bytes larger than usize::MAX",
            )
        })?;
        let bytes = match self.parse_bytes_direct(byte_len)? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };
        let (head, slice, tail) = bytes.align_to();

        if !head.is_empty() || !tail.is_empty() {
            return Ok(None);
        }

        Ok(Some(slice))
    }

    /// Attempt to directly transmute a slice in the source buffer and, if that
    /// fails, parse it instead.
    ///
    /// # Safety
    /// This has all the same safety preconditions as
    /// [`parse_slice_direct`](Self::parse_slice_direct). That is, is must be
    /// valid to transmute bytes in to a `T` instance.
    pub unsafe fn parse_slice<T>(&mut self, len: usize) -> ParseResult<Cow<'p, [T]>>
    where
        T: Parse<'p> + Copy,
    {
        Ok(match self.parse_slice_direct(len)? {
            Some(slice) => Cow::Borrowed(slice),
            None => Cow::Owned(self.parse_repeated(len)?),
        })
    }

    /// Parse a sequence of `len` `T`s.
    pub fn parse_repeated<T: Parse<'p>>(&mut self, len: usize) -> ParseResult<Vec<T>> {
        let mut vec = Vec::with_capacity(len.min(self.safe_capacity_bound::<T>()));
        for _ in 0..len {
            vec.push(self.parse()?);
        }

        Ok(vec)
    }

    /// Parse record metadata and return a parser for the bytes of the record.
    ///
    /// If you have already read the record header, use
    /// [`parse_metadata_with_header`](Parser::parse_metadata_with_header)
    /// instead.
    pub fn parse_metadata(
        &mut self,
    ) -> ParseResult<(Parser<impl ParseBuf<'p>, E>, RecordMetadata)> {
        let header = self.parse()?;
        self.parse_metadata_with_header(header)
    }

    fn parse_metadata_with_header_impl(
        &mut self,
        header: bindings::perf_event_header,
    ) -> ParseResult<(Parser<ParseBufCursor<'p>, E>, RecordMetadata)> {
        use perf_event_open_sys::bindings::*;
        use std::mem;

        let data_len = (header.size as usize)
            .checked_sub(mem::size_of_val(&header))
            .ok_or_else(|| {
                ParseError::custom(
                    ErrorKind::InvalidRecord,
                    "header size was too small to be valid",
                )
            })?;
        let mut rp = self.split_at(data_len)?;
        // MMAP and SAMPLE records do not have the sample_id struct.
        // All other records do.
        let (p, sample_id) = match header.type_ {
            PERF_RECORD_MMAP | PERF_RECORD_SAMPLE => (rp, SampleId::default()),
            _ => {
                let remaining_len = data_len
                    .checked_sub(SampleId::estimate_len(rp.config()))
                    .ok_or_else(|| ParseError::custom(
                        ErrorKind::InvalidRecord,
                        "config has sample_id_all bit set but record does not have enough data to store the sample_id"
                    ))?;

                let p = rp.split_at(remaining_len)?;
                (p, rp.parse()?)
            }
        };

        let metadata = RecordMetadata::new(header, sample_id);
        Ok((p, metadata))
    }

    /// Parse the record metadata and return a parser for only the record bytes.
    pub fn parse_metadata_with_header(
        &mut self,
        header: bindings::perf_event_header,
    ) -> ParseResult<(Parser<impl ParseBuf<'p>, E>, RecordMetadata)> {
        self.parse_metadata_with_header_impl(header)
    }

    /// Parse a record, the record types will be visited by the `visitor`.
    pub fn parse_record<V: Visitor<'p>>(&mut self, visitor: V) -> ParseResult<V::Output> {
        let header = self.parse()?;
        self.parse_record_with_header(visitor, header)
    }

    fn parse_record_impl<V: Visitor<'p>>(
        self,
        visitor: V,
        metadata: RecordMetadata,
    ) -> ParseResult<V::Output> {
        use perf_event_open_sys::bindings::*;

        let mut p = Parser::new(self.data, self.config.with_misc(metadata.misc()));

        Ok(match metadata.ty() {
            PERF_RECORD_MMAP => visitor.visit_mmap(p.parse()?, metadata),
            PERF_RECORD_LOST => visitor.visit_lost(p.parse()?, metadata),
            PERF_RECORD_COMM => visitor.visit_comm(p.parse()?, metadata),
            PERF_RECORD_EXIT => visitor.visit_exit(p.parse()?, metadata),
            PERF_RECORD_THROTTLE => visitor.visit_throttle(p.parse()?, metadata),
            PERF_RECORD_UNTHROTTLE => visitor.visit_unthrottle(p.parse()?, metadata),
            PERF_RECORD_FORK => visitor.visit_fork(p.parse()?, metadata),
            PERF_RECORD_READ => visitor.visit_read(p.parse()?, metadata),
            PERF_RECORD_SAMPLE => visitor.visit_sample(p.parse()?, metadata),
            PERF_RECORD_MMAP2 => visitor.visit_mmap2(p.parse()?, metadata),
            PERF_RECORD_AUX => visitor.visit_aux(p.parse()?, metadata),
            PERF_RECORD_ITRACE_START => visitor.visit_itrace_start(p.parse()?, metadata),
            PERF_RECORD_LOST_SAMPLES => visitor.visit_lost_samples(p.parse()?, metadata),
            PERF_RECORD_SWITCH_CPU_WIDE => visitor.visit_switch_cpu_wide(p.parse()?, metadata),
            PERF_RECORD_NAMESPACES => visitor.visit_namespaces(p.parse()?, metadata),
            PERF_RECORD_KSYMBOL => visitor.visit_ksymbol(p.parse()?, metadata),
            PERF_RECORD_BPF_EVENT => visitor.visit_bpf_event(p.parse()?, metadata),
            PERF_RECORD_CGROUP => visitor.visit_cgroup(p.parse()?, metadata),
            PERF_RECORD_TEXT_POKE => visitor.visit_text_poke(p.parse()?, metadata),
            PERF_RECORD_AUX_OUTPUT_HW_ID => visitor.visit_aux_output_hw_id(p.parse()?, metadata),
            _ => visitor.visit_unknown(p.parse_rest()?, metadata),
        })
    }

    /// Same as [`parse_record`](Self::parse_record) but required that the
    /// header be provided.
    pub fn parse_record_with_header<V: Visitor<'p>>(
        &mut self,
        visitor: V,
        header: bindings::perf_event_header,
    ) -> ParseResult<V::Output> {
        let (p, metadata) = self.parse_metadata_with_header_impl(header)?;

        match p.data.as_slice() {
            Some(data) => {
                // Fast path: the data is all in one contiguous borrowed slice so we can
                //            parse based on that.
                let p = Parser::new(data, p.config);
                p.parse_record_impl(visitor, metadata)
            }
            // Slow path: we have either an unowned slice or multiple slices so the ParseBuf
            //            implementation needs to do more work to handle that.
            None => p.parse_record_impl(visitor, metadata),
        }
    }
}

impl<'p> Parse<'p> for u8 {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_u8()
    }
}

impl<'p> Parse<'p> for u16 {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_u16()
    }
}

impl<'p> Parse<'p> for u32 {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_u32()
    }
}

impl<'p> Parse<'p> for u64 {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_u64()
    }
}

impl<'p, const N: usize> Parse<'p> for [u8; N] {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_array()
    }
}

impl<'p> Parse<'p> for bindings::perf_event_header {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            type_: p.parse()?,
            misc: p.parse()?,
            size: p.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endian::Native;

    #[test]
    fn parse_rest() {
        let data: &[u8] = &[1, 2, 3, 4, 5];
        let mut parser = Parser::new(data, ParseConfig::<Native>::default());
        let rest = parser.parse_rest().unwrap();

        assert_eq!(data, &*rest);
    }
}
