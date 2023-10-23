use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt;

use crate::prelude::*;
use crate::Mmap2;

/// MMAP events record memory mappings.
///
/// This struct corresponds to `PERF_RECORD_MMAP`. See the [manpage] for more
/// documentation here.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone)]
pub struct Mmap<'a> {
    /// The process ID.
    pub pid: u32,

    /// The thread ID.
    pub tid: u32,

    /// The address that the mapping was placed at in the process' address
    /// space.
    pub addr: u64,

    /// The length, in bytes, of the allocated memory.
    pub len: u64,

    /// The page offset of the memory mapping.
    pub pgoff: u64,

    /// The path to the file that is being mapped, if there is one.
    ///
    /// # Notes
    /// - Not all memory mappings have a path on the file system. In cases where
    ///   there is no such path then this will be a label(ish) string from the
    ///   kernel (e.g. `[stack]`, `[heap]`, `[vdso]`, etc.)
    /// - Just because the mapping has a path doesn't necessarily mean that the
    ///   file at that path was the file that was mapped. The file may have been
    ///   deleted in the meantime or the process may be under a chroot.
    ///
    /// If you need to be able to tell whether the file at the path is the same
    /// one as was mapped you will need to use [`Mmap2`] instead.
    pub filename: Cow<'a, [u8]>,
}

impl<'a> Mmap<'a> {
    /// The path to the file that is being mapped, as an [`OsStr`].
    ///
    /// # Notes
    /// - Not all memory mappings have a path on the file system. In cases where
    ///   there is no such path then this will be a label(ish) string from the
    ///   kernel (e.g. `[stack]`, `[heap]`, `[vdso]`, etc.)
    /// - Just because the mapping has a path doesn't necessarily mean that the
    ///   file at that path was the file that was mapped. The file may have been
    ///   deleted in the meantime or the process may be under a chroot.
    ///
    /// If you need to be able to tell whether the file at the path is the same
    /// one as was mapped you will need to use [`Mmap2`] instead.
    #[cfg(unix)]
    pub fn filename_os(&self) -> &OsStr {
        use std::os::unix::ffi::OsStrExt;

        OsStrExt::from_bytes(&self.filename)
    }

    /// Convert all the borrowed data in this `Mmap` into owned data.
    pub fn into_owned(self) -> Mmap<'static> {
        Mmap {
            filename: self.filename.into_owned().into(),
            ..self
        }
    }
}

impl<'p> Parse<'p> for Mmap<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
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

impl fmt::Debug for Mmap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mmap")
            .field("pid", &self.pid)
            .field("tid", &self.tid)
            .field("addr", &format_args!("{:#016X}", &self.addr))
            .field("len", &self.len)
            .field("pgoff", &self.pgoff)
            .field("filename", &crate::util::fmt::ByteStr(&self.filename))
            .finish()
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
