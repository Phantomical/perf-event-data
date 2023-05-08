//! Parsing interface for parsing data in to record types.
//!
//! In this crate, parsing is built on parser functions that take in a
//! [`Parser`] and produce a [`Result<T>`] where `T` is some record type. Most
//! of the time, you should be able to call [`Parser::parse`] for each of your
//! fields but [`Parser`] provides many other helper methods for when that isn't
//! enough.
//!
//! # Using the Parser
//! If you just need to parse records from a buffer (e.g., you are parsing from
//! a file or the output of `perf_event_open`) then you can use
//! [`Parser::parse_record`] along with a visitor of your choice.

use std::borrow::Cow;
use std::mem::MaybeUninit;

use perf_event_open_sys::bindings;

use crate::cowutils::CowSliceExt;
use crate::endian::Endian;
use crate::parsebuf::ParseBufCursor;
use crate::{RecordMetadata, SampleId, Visitor};

pub use crate::config::ParseConfig;
pub use crate::error::{ErrorKind, ParseError, Result};
pub use crate::parsebuf::{ParseBuf, ParseBufChunk};

/// A type that can be parsed
pub trait Parse<'p>: Sized {
    /// Parse `Self` using the provided [`Parser`] instance.
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>;
}

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
    fn split_at(&mut self, offset: usize) -> Result<Parser<ParseBufCursor<'p>, E>> {
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

    fn parse_bytes_direct(&mut self, len: usize) -> Result<Option<&'p [u8]>> {
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

    /// Directly get a reference to the next `len` bytes in the input buffer.
    pub fn parse_bytes(&mut self, mut len: usize) -> Result<Cow<'p, [u8]>> {
        if let Some(bytes) = self.parse_bytes_direct(len)? {
            return Ok(Cow::Borrowed(bytes));
        }

        let mut bytes = Vec::with_capacity(self.safe_capacity_bound::<u8>().min(len));
        while len > 0 {
            let mut chunk = self.data.chunk()?;
            chunk.truncate(len);
            bytes.extend_from_slice(&chunk);

            let chunk_len = chunk.len();
            len -= chunk_len;
            self.data.advance(chunk_len);
        }

        Ok(Cow::Owned(bytes))
    }

    /// Parse a slice in its entirety. If this returns successfully then the
    /// entire slice has been initialized.
    fn parse_to_slice(&mut self, mut slice: &mut [MaybeUninit<u8>]) -> Result<()> {
        while !slice.is_empty() {
            let chunk = self.data.chunk()?;
            let len = slice.len().min(chunk.len());

            unsafe {
                std::ptr::copy_nonoverlapping(chunk.as_ptr(), slice.as_mut_ptr() as *mut u8, len)
            };

            slice = slice.split_at_mut(len).1;
            self.data.advance(len);
        }

        Ok(())
    }

    pub(crate) fn parse_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut array = [0u8; N];
        self.parse_to_slice(unsafe { array.align_to_mut().1 })?;
        Ok(array)
    }

    /// Parse a type.
    ///
    /// If the type fails to parse then this parser will not be modified.
    pub fn parse<P: Parse<'p>>(&mut self) -> Result<P> {
        P::parse(self)
    }

    /// Parse with an explicit parsing function.
    pub fn parse_with<F, R>(&mut self, func: F) -> Result<R>
    where
        F: FnOnce(&mut Self) -> Result<R>,
    {
        func(self)
    }

    /// Parse a type only if `parse` is true.
    pub fn parse_if<P: Parse<'p>>(&mut self, parse: bool) -> Result<Option<P>> {
        self.parse_if_with(parse, P::parse)
    }

    /// `parse_if` but using an explicit parsing function.
    pub fn parse_if_with<F, R>(&mut self, parse: bool, func: F) -> Result<Option<R>>
    where
        F: FnOnce(&mut Self) -> Result<R>,
    {
        match parse {
            true => self.parse_with(func).map(Some),
            false => Ok(None),
        }
    }

    /// Parse a single byte out of the source buffer.
    pub fn parse_u8(&mut self) -> Result<u8> {
        let [byte] = self.parse_array()?;
        Ok(byte)
    }

    /// Parse a `u16` out of the source data.
    pub fn parse_u16(&mut self) -> Result<u16> {
        let array = self.parse_array()?;
        Ok(self.endian().convert_u16(array))
    }

    /// Parse a `u32` out of the source data.
    pub fn parse_u32(&mut self) -> Result<u32> {
        let array = self.parse_array()?;
        Ok(self.endian().convert_u32(array))
    }

    /// Parse a `u64` out of the source data.
    pub fn parse_u64(&mut self) -> Result<u64> {
        let array = self.parse_array()?;
        Ok(self.endian().convert_u64(array))
    }

    /// Consume the rest of the buffer and return it as a slice.
    pub fn parse_rest(&mut self) -> Result<Cow<'p, [u8]>> {
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
    pub fn parse_rest_trim_nul(&mut self) -> Result<Cow<'p, [u8]>> {
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
    pub unsafe fn parse_slice_direct<T>(&mut self, len: usize) -> Result<Option<&'p [T]>>
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
    pub unsafe fn parse_slice<T>(&mut self, len: usize) -> Result<Cow<'p, [T]>>
    where
        T: Parse<'p> + Copy,
    {
        Ok(match self.parse_slice_direct(len)? {
            Some(slice) => Cow::Borrowed(slice),
            None => Cow::Owned(self.parse_repeated(len)?),
        })
    }

    /// Parse a sequence of `len` `T`s.
    pub fn parse_repeated<T: Parse<'p>>(&mut self, len: usize) -> Result<Vec<T>> {
        let mut vec = Vec::with_capacity(len.min(self.safe_capacity_bound::<T>()));
        for _ in 0..len {
            vec.push(self.parse()?);
        }

        Ok(vec)
    }

    pub fn parse_metadata(&mut self) -> Result<(Parser<impl ParseBuf<'p>, E>, RecordMetadata)> {
        let header = self.parse()?;
        self.parse_metadata_with_header(header)
    }

    /// Parse the record metadata and return a parser for only the record bytes.
    pub fn parse_metadata_with_header(
        &mut self,
        header: bindings::perf_event_header,
    ) -> Result<(Parser<impl ParseBuf<'p>, E>, RecordMetadata)> {
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

    pub fn parse_record<V: Visitor>(&mut self, visitor: V) -> Result<V::Output<'p>> {
        let header = self.parse()?;
        self.parse_record_with_header(visitor, header)
    }

    pub fn parse_record_with_header<V: Visitor>(
        &mut self,
        visitor: V,
        header: bindings::perf_event_header,
    ) -> Result<V::Output<'p>> {
        use perf_event_open_sys::bindings::*;

        let (mut p, metadata) = self.parse_metadata_with_header(header)?;
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
}

impl<'p> Parse<'p> for u8 {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_u8()
    }
}

impl<'p> Parse<'p> for u16 {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_u16()
    }
}

impl<'p> Parse<'p> for u32 {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_u32()
    }
}

impl<'p> Parse<'p> for u64 {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_u64()
    }
}

impl<'p, const N: usize> Parse<'p> for [u8; N] {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_array()
    }
}

impl<'p> Parse<'p> for bindings::perf_event_header {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
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
