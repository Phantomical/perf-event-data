use std::fmt::{self, UpperHex, Write};

/// Write an array of bytes containing possibly invalid UTF-8 as if it was a
/// debug string.
///
/// This prints all the valid UTF-8 parts of the string using
/// `char::escape_debug` and the invalid parts using `u8::escape_default`.
pub(crate) struct ByteStr<'a>(pub &'a [u8]);

impl fmt::Debug for ByteStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut data = self.0;
        f.write_char('"')?;

        while !data.is_empty() {
            let (text, errlen) = match std::str::from_utf8(data) {
                Ok(text) => (text, 0),
                Err(e) => {
                    (
                        // SAFETY: this part of the string has been validated
                        unsafe { std::str::from_utf8_unchecked(&data[..e.valid_up_to()]) },
                        e.error_len().unwrap_or(data.len() - e.valid_up_to()),
                    )
                }
            };

            for c in text.chars().flat_map(|c| c.escape_debug()) {
                f.write_char(c)?;
            }

            data = &data[text.len()..];
            let (error, rest) = data.split_at(errlen);
            data = rest;

            for b in error.iter().flat_map(|b| b.escape_ascii()) {
                f.write_char(b as char)?;
            }
        }

        f.write_char('"')
    }
}

/// Format a byte array as hex.
pub(crate) struct HexStr<'a>(pub &'a [u8]);

impl fmt::Debug for HexStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &b in self.0 {
            let nibbles = [b & 0xF, b >> 4];

            for n in nibbles {
                let c = match n {
                    0x0..=0x9 => b'0' + n,
                    0xA..=0xF => b'A' + n,
                    _ => unreachable!(),
                };

                f.write_char(c as char)?;
            }
        }

        Ok(())
    }
}

pub(crate) struct HexAddr<T>(pub T);

impl<T: UpperHex> fmt::Debug for HexAddr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#016X}", self.0))
    }
}
