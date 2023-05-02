use std::borrow::Cow;

use crate::prelude::*;
use crate::Mmap2;

/// MMAP events record memory mappings.
///
/// This struct corresponds to `PERF_RECORD_MMAP`. See the [manpage] for more
/// documentation here.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Mmap<'a> {
    pub pid: u32,
    pub tid: u32,
    pub addr: u64,
    pub len: u64,
    pub pgoff: u64,
    pub filename: Cow<'a, [u8]>,
}

impl<'a> Mmap<'a> {
    #[cfg(unix)]
    pub fn filename_os(&self) -> &std::ffi::OsStr {
        use std::os::unix::ffi::OsStrExt;

        OsStrExt::from_bytes(&self.filename)
    }

    pub fn into_owned(self) -> Mmap<'static> {
        Mmap {
            filename: self.filename.into_owned().into(),
            ..self
        }
    }
}

impl<'p> Parse<'p> for Mmap<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            pid: p.parse()?,
            tid: p.parse()?,
            addr: p.parse()?,
            len: p.parse()?,
            pgoff: p.parse()?,
            filename: p.parse_rest_trim_nul()?,
        })
    }
}

impl<'a> From<Mmap2<'a>> for Mmap<'a> {
    fn from(value: Mmap2<'a>) -> Self {
        value.into_mmap()
    }
}

#[cfg(test)]
mod tests {
    use crate::endian::Little;

    use super::*;

    #[test]
    fn test_parse() {
        let bytes: &[u8] = &[
            10, 100, 0, 0, 11, 100, 0, 0, 0, 160, 118, 129, 189, 127, 0, 0, 0, 16, 0, 0, 0, 0, 0,
            0, 0, 160, 118, 129, 189, 127, 0, 0, 47, 47, 97, 110, 111, 110, 0, 0,
        ];

        let mut parser: Parser<_, Little> = Parser::new(bytes, ParseConfig::default());
        let mmap: Mmap = parser.parse().unwrap();

        assert_eq!(mmap.pid, 25610);
        assert_eq!(mmap.tid, 25611);
        assert_eq!(mmap.addr, 0x7FBD8176A000);
        assert_eq!(mmap.len, 4096);
        assert_eq!(mmap.pgoff, 0x7FBD8176A000);
        assert_eq!(&*mmap.filename, b"//anon");
    }
}
