use std::borrow::Cow;

use crate::prelude::*;
use crate::Mmap;

/// MMAP2 events record memory mappings with extra info compared to MMAP
/// records.
///
/// This struct corresponds to `PERF_RECORD_MMAP2`. See the [manpage] for more
/// documentation here.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Mmap2<'a> {
    pub pid: u32,
    pub tid: u32,
    pub addr: u64,
    pub len: u64,
    pub pgoff: u64,
    pub maj: u32,
    pub min: u32,
    pub ino: u64,
    pub ino_generation: u64,
    pub prot: u32,
    pub flags: u32,
    pub filename: Cow<'a, [u8]>,
}

impl<'a> Mmap2<'a> {
    #[cfg(unix)]
    pub fn filename_os(&self) -> &std::ffi::OsStr {
        use std::os::unix::ffi::OsStrExt;

        OsStrExt::from_bytes(&self.filename)
    }

    /// Convert this record to a [`Mmap`] record.
    pub fn to_mmap(&self) -> Mmap<'a> {
        self.clone().into_mmap()
    }

    /// Convert this record to a [`Mmap`] record.
    pub fn into_mmap(self) -> Mmap<'a> {
        Mmap {
            pid: self.pid,
            tid: self.tid,
            addr: self.addr,
            len: self.len,
            pgoff: self.pgoff,
            filename: self.filename,
        }
    }

    pub fn into_owned(self) -> Mmap2<'static> {
        Mmap2 {
            filename: self.filename.into_owned().into(),
            ..self
        }
    }
}

impl<'p> Parse<'p> for Mmap2<'p> {
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
            maj: p.parse()?,
            min: p.parse()?,
            ino: p.parse()?,
            ino_generation: p.parse()?,
            prot: p.parse()?,
            flags: p.parse()?,
            filename: p.parse_rest_trim_nul()?,
        })
    }
}
