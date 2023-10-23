use std::borrow::Cow;

use crate::prelude::*;

/// TEXT_POKE records indicate a change in the kernel text.
///
/// This struct corresponds to `PERF_RECORD_TEXT_POKE`. See the [manpage] for
/// more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct TextPoke<'a> {
    /// The address of the change.
    pub addr: u64,

    /// The old bytes at `addr`.
    pub old_bytes: Cow<'a, [u8]>,

    /// The new bytes at `addr`.
    pub new_bytes: Cow<'a, [u8]>,
}

impl<'a> TextPoke<'a> {
    /// Convert all the borrowed data in this `TextPoke` into owned data.
    pub fn to_owned(self) -> TextPoke<'static> {
        TextPoke {
            old_bytes: self.old_bytes.into_owned().into(),
            new_bytes: self.new_bytes.into_owned().into(),
            ..self
        }
    }
}

impl<'p> Parse<'p> for TextPoke<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        use crate::util::cow::CowSliceExt;

        let addr = p.parse()?;
        let old_len = p.parse_u16()? as usize;
        let new_len = p.parse_u16()? as usize;

        // The records emitted by perf_event_open always have a length that is a
        // multiple of 8. Strictly speaking, we don't have to do this since this is the
        // end of the record and higher levels should avoid this being a problem, but
        // it's best to do things right here anyways.
        let full_len = round_up_mod(old_len + new_len, 4, 8);
        let bytes = p.parse_bytes(full_len)?;

        let (old_bytes, mut new_bytes) = bytes.split_at(old_len);
        new_bytes.truncate(new_len);

        Ok(Self {
            addr,
            old_bytes,
            new_bytes,
        })
    }
}

/// Round v up so that it is equal to k (mod m)
fn round_up_mod(v: usize, k: usize, m: usize) -> usize {
    assert!(k < m);

    v + match v % m {
        vm if vm <= k => k - vm,
        vm if vm > k => (k + m) - vm,
        _ => unreachable!(),
    }
}
