use std::borrow::Cow;
use std::io::{BufRead, BufReader, Read};
use std::ops::Deref;

use crate::parse::{ParseError, Parser, Result};

used_in_docs!(Parser);

/// A continuous chunk of data read from a [`ParseBuf`].
///
/// A `ParseBufChunk` has two variants:
/// - [`Temporary`] is for when the data being referenced is owned by the
///   [`ParseBuf`] instance itself. This cannot be kept around and will not be
///   kept in borrowed form while parsing.
/// - [`External`] is for when the data being referenced is borrowed from
///   elsewhere. This allows record parsing to avoid having to copy the data
///   and, if possible, should be slightly faster.
///
/// When implmenting a [`ParseBuf`] instance, you should return [`External`] if
/// possible.
///
/// [`Temporary`]: ParseBufChunk::Temporary
/// [`External`]: ParseBufChunk::External
#[derive(Copy, Clone, Debug)]
pub enum ParseBufChunk<'tmp, 'ext: 'tmp> {
    /// Data owned by the current [`ParseBuf`] instance. Will only remain valid
    /// until [`ParseBuf::advance`] is called.
    Temporary(&'tmp [u8]),

    /// Data not owned by the [`ParseBuf`] instance. Will remain valid even
    /// after the [`ParseBuf`] is dropped.
    External(&'ext [u8]),
}

impl<'tmp, 'ext: 'tmp> ParseBufChunk<'tmp, 'ext> {
    pub(crate) fn to_cow(self) -> Cow<'ext, [u8]> {
        match self {
            Self::Temporary(data) => Cow::Owned(data.to_vec()),
            Self::External(data) => Cow::Borrowed(data),
        }
    }

    pub(crate) fn truncate(&mut self, len: usize) {
        if self.len() >= len {
            return;
        }

        match self {
            Self::Temporary(data) => *data = data.split_at(len).0,
            Self::External(data) => *data = data.split_at(len).0,
        }
    }
}

impl<'tmp, 'ext: 'tmp> Deref for ParseBufChunk<'tmp, 'ext> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Temporary(bytes) => bytes,
            Self::External(bytes) => bytes,
        }
    }
}

/// A data source from which [`Parser`] can parse data.
///
/// A [`ParseBuf`] has two main components:
/// - An internal buffer that stores some amount of data. [`chunk`] returns a
///   view into this buffer.
/// - A position, [`advance`] moves this forward.
///
/// [`chunk`]: ParseBuf::chunk
/// [`advance`]: ParseBuf::advance
pub trait ParseBuf<'p> {
    /// Returns a chunk starting at the current position.
    ///
    /// This method must never return an empty chunk. If an empty chunk would be
    /// returned, it should return an error instead. [`ParseError::eof`] has
    /// been provided for this, though it is not required to use it.
    ///
    /// This method must keep returning the same data until [`advance`] has been
    /// called to move past it.
    ///
    /// See the documentation for [`ParseBufChunk`] for an explanation on when
    /// to use [`ParseBufChunk::Temporary`] vs [`ParseBufChunk::External`].
    ///
    /// [`advance`]: ParseBuf::advance
    fn chunk(&mut self) -> Result<ParseBufChunk<'_, 'p>>;

    /// Advance this buffer past `count` bytes.
    fn advance(&mut self, count: usize);

    /// An indicator of how many bytes are left, if supported.
    ///
    /// This is used for some optimizations within [`Parser`], if `Some` is
    /// returned then the value must be accurate.
    fn remaining_hint(&self) -> Option<usize> {
        None
    }
}

impl<'p> ParseBuf<'p> for &'p [u8] {
    fn chunk(&mut self) -> Result<ParseBufChunk<'_, 'p>> {
        if self.is_empty() {
            return Err(ParseError::eof());
        }

        Ok(ParseBufChunk::External(self))
    }

    fn advance(&mut self, count: usize) {
        *self = self.split_at(count).1;
    }

    fn remaining_hint(&self) -> Option<usize> {
        Some(self.len())
    }
}

// This impl would work for any type that implements BufRead. Unfortunately,
// that conflicts with the implementation of ParseBuf for &[u8]
impl<'p, R> ParseBuf<'p> for BufReader<R>
where
    R: Read,
{
    fn chunk(&mut self) -> Result<ParseBufChunk<'_, 'p>> {
        let buf = self.fill_buf()?;

        if buf.is_empty() {
            Err(ParseError::eof())
        } else {
            Ok(ParseBufChunk::Temporary(buf))
        }
    }

    fn advance(&mut self, count: usize) {
        self.consume(count)
    }
}

pub(crate) struct ParseBufCursor<'p> {
    chunks: Vec<Cow<'p, [u8]>>,
    offset: usize,
    len: usize,
}

impl<'p> ParseBufCursor<'p> {
    pub(crate) fn new<B>(buf: &mut B, mut len: usize) -> Result<Self>
    where
        B: ParseBuf<'p>,
    {
        let mut chunks = Vec::with_capacity(2);
        let total_len = len;

        while len > 0 {
            let mut chunk = buf.chunk()?;
            chunk.truncate(len);

            len -= chunk.len();
            chunks.push(chunk.to_cow());
        }

        chunks.reverse();

        Ok(Self {
            chunks,
            offset: 0,
            len: total_len,
        })
    }
}

impl<'p> ParseBuf<'p> for ParseBufCursor<'p> {
    fn chunk(&mut self) -> Result<ParseBufChunk<'_, 'p>> {
        match self.chunks.last().ok_or_else(ParseError::eof)? {
            Cow::Borrowed(data) => Ok(ParseBufChunk::External(&data[self.offset..])),
            Cow::Owned(data) => Ok(ParseBufChunk::Temporary(&data[self.offset..])),
        }
    }

    fn advance(&mut self, count: usize) {
        self.len = self
            .len
            .checked_sub(count)
            .expect("advanced past the end of the buffer");
        self.offset += count;

        if let Some(chunk) = self.chunks.last() {
            if self.offset >= chunk.len() {
                self.offset -= chunk.len();
                self.chunks.pop();
            }
        }
    }

    fn remaining_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}
