use std::fmt::{self, Write};

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
