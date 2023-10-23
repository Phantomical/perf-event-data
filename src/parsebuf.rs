use std::borrow::Cow;
use std::io::{BufRead, BufReader, Read};
use std::ops::Deref;

use crate::parse::{ParseError, ParseResult, Parser};

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
    #[inline]
    pub(crate) fn to_cow(self) -> Cow<'ext, [u8]> {
        match self {
            Self::Temporary(data) => Cow::Owned(data.to_vec()),
            Self::External(data) => Cow::Borrowed(data),
        }
    }

    #[inline]
    pub(crate) fn truncate(&mut self, len: usize) {
        if self.len() <= len {
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

    #[inline]
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
/// # Safety
/// - If [`remaining_hint`] returns `Some` then the returned value must be
///   accurate.
///
/// [`chunk`]: ParseBuf::chunk
/// [`advance`]: ParseBuf::advance
/// [`remaining_hint`]: ParseBuf::remaining_hint
pub unsafe trait ParseBuf<'p> {
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
    fn chunk(&mut self) -> ParseResult<ParseBufChunk<'_, 'p>>;

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

unsafe impl<'p> ParseBuf<'p> for &'p [u8] {
    #[inline]
    fn chunk(&mut self) -> ParseResult<ParseBufChunk<'_, 'p>> {
        if self.is_empty() {
            return Err(ParseError::eof());
        }

        Ok(ParseBufChunk::External(self))
    }

    #[inline]
    fn advance(&mut self, count: usize) {
        *self = self.split_at(count).1;
    }

    #[inline]
    fn remaining_hint(&self) -> Option<usize> {
        Some(self.len())
    }
}

// This impl would work for any type that implements BufRead. Unfortunately,
// that conflicts with the implementation of ParseBuf for &[u8]
unsafe impl<'p, R> ParseBuf<'p> for BufReader<R>
where
    R: Read,
{
    #[inline]
    fn chunk(&mut self) -> ParseResult<ParseBufChunk<'_, 'p>> {
        let buf = self.fill_buf()?;

        if buf.is_empty() {
            Err(ParseError::eof())
        } else {
            Ok(ParseBufChunk::Temporary(buf))
        }
    }

    #[inline]
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
    pub(crate) fn new<B>(buf: &mut B, mut len: usize) -> ParseResult<Self>
    where
        B: ParseBuf<'p>,
    {
        let mut chunks = Vec::with_capacity(2);
        let total_len = len;

        while len > 0 {
            let mut chunk = buf.chunk()?;
            chunk.truncate(len);

            if chunk.len() > 0 {
                chunks.push(chunk.to_cow());
            }

            let chunk_len = chunk.len();
            len -= chunk_len;
            buf.advance(chunk_len);
        }

        chunks.reverse();

        Ok(Self {
            chunks,
            offset: 0,
            len: total_len,
        })
    }

    pub(crate) fn as_slice(&self) -> Option<&'p [u8]> {
        if self.chunks.len() != 1 {
            return None;
        }

        match &self.chunks[0] {
            Cow::Borrowed(data) => Some(*data),
            _ => None,
        }
    }
}

impl<'p> ParseBufCursor<'p> {
    #[cold]
    fn advance_slow(&mut self) {
        while let Some(chunk) = self.chunks.last() {
            if self.offset < chunk.len() {
                break;
            }

            self.offset -= chunk.len();
            self.chunks.pop();
        }

        if self.chunks.is_empty() {
            assert_eq!(self.offset, 0, "advanced past the end of the buffer");
        }
    }
}

unsafe impl<'p> ParseBuf<'p> for ParseBufCursor<'p> {
    #[inline]
    fn chunk(&mut self) -> ParseResult<ParseBufChunk<'_, 'p>> {
        match self.chunks.last().ok_or_else(ParseError::eof)? {
            Cow::Borrowed(data) => Ok(ParseBufChunk::External(&data[self.offset..])),
            Cow::Owned(data) => Ok(ParseBufChunk::Temporary(&data[self.offset..])),
        }
    }

    #[inline]
    fn advance(&mut self, count: usize) {
        self.offset = self
            .offset
            .checked_add(count)
            .expect("advanced past the end of the buffer");

        self.len
            .checked_sub(count)
            .expect("advanced past the end of the buffer");

        match self.chunks.last() {
            Some(chunk) if chunk.len() > self.offset => (),
            _ => self.advance_slow(),
        }
    }

    #[inline]
    fn remaining_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ChunkBuf<'a>(Vec<&'a [u8]>);

    unsafe impl<'p> ParseBuf<'p> for ChunkBuf<'p> {
        fn chunk(&mut self) -> ParseResult<ParseBufChunk<'_, 'p>> {
            self.0
                .first()
                .copied()
                .map(ParseBufChunk::External)
                .ok_or_else(ParseError::eof)
        }

        fn advance(&mut self, mut count: usize) {
            while let Some(chunk) = self.0.first_mut() {
                if count < chunk.len() {
                    chunk.advance(count);
                    break;
                } else {
                    count -= chunk.len();
                    self.0.remove(0);
                }
            }
        }
    }

    #[test]
    fn cursor_over_split() {
        let mut buf = ChunkBuf(vec![b"abcdef", b"012456789"]);
        let _cursor = ParseBufCursor::new(&mut buf, 8);
    }

    #[test]
    fn cursor_zero_split() {
        let mut buf = ChunkBuf(vec![b"", b"01234"]);
        let _cursor = ParseBufCursor::new(&mut buf, 4);
    }
}
