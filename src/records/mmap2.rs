use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt;

use perf_event_open_sys::bindings;

use crate::error::ParseError;
use crate::prelude::*;
use crate::Mmap;

used_in_docs!(OsStr);

/// MMAP2 events record memory mappings with extra info compared to MMAP
/// records.
///
/// This struct corresponds to `PERF_RECORD_MMAP2`. See the [manpage] for more
/// documentation here.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone)]
pub struct Mmap2<'a> {
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

    /// Protection information for the mapping.
    pub prot: u32,

    /// Flags used when creating the mapping.
    pub flags: u32,

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

    detail: MmapDetail,
}

#[derive(Clone)]
enum MmapDetail {
    Default {
        maj: u32,
        min: u32,
        ino: u64,
        ino_generation: u64,
    },
    BuildId {
        build_id: [u8; 20],
        len: u8,
    },
}

impl<'a> Mmap2<'a> {
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

    /// The major ID of the underlying device of the fd being mapped.
    pub fn maj(&self) -> Option<u32> {
        match &self.detail {
            MmapDetail::Default { maj, .. } => Some(*maj),
            _ => None,
        }
    }

    /// The minor ID of the underlying device of the fd being mapped.
    pub fn min(&self) -> Option<u32> {
        match &self.detail {
            MmapDetail::Default { min, .. } => Some(*min),
            _ => None,
        }
    }

    /// The inode number.
    pub fn ino(&self) -> Option<u64> {
        match &self.detail {
            MmapDetail::Default { ino, .. } => Some(*ino),
            _ => None,
        }
    }

    /// The inode generation.
    pub fn ino_generation(&self) -> Option<u64> {
        match &self.detail {
            MmapDetail::Default { ino_generation, .. } => Some(*ino_generation),
            _ => None,
        }
    }

    /// The build id of the binary being mapped.
    ///
    /// This variant will only be generated if `build_id` was set when building
    /// the counter.
    pub fn build_id(&self) -> Option<&[u8]> {
        match &self.detail {
            MmapDetail::BuildId { build_id, len } => Some(&build_id[..*len as usize]),
            _ => None,
        }
    }

    /// Convert this record to a [`Mmap`] record.
    pub fn to_mmap(&self) -> Mmap<'a> {
        self.clone().into_mmap()
    }

    /// Convert this record to a [`Mmap`] record.
    #[inline]
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

    /// Convert all the borrowed data in this `Mmap2` into owned data.
    pub fn into_owned(self) -> Mmap2<'static> {
        Mmap2 {
            filename: self.filename.into_owned().into(),
            ..self
        }
    }
}

impl<'p> Parse<'p> for Mmap2<'p> {
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
            detail: p.parse()?,
            prot: p.parse()?,
            flags: p.parse()?,
            filename: p.parse_rest_trim_nul()?,
        })
    }
}

impl<'p> Parse<'p> for MmapDetail {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        if p.config().misc() & bindings::PERF_RECORD_MISC_MMAP_BUILD_ID as u16 != 0 {
            let len: u8 = p.parse()?;
            let _ = p.parse_u8()?;
            let _ = p.parse_u16()?;
            let build_id = p.parse_array()?;

            if len as usize > build_id.len() {
                return Err(ParseError::custom(
                    ErrorKind::InvalidRecord,
                    format_args!("build_id had invalid length ({len} > 20)"),
                ));
            }

            Ok(Self::BuildId { build_id, len })
        } else {
            Ok(Self::Default {
                maj: p.parse()?,
                min: p.parse()?,
                ino: p.parse()?,
                ino_generation: p.parse()?,
            })
        }
    }
}

impl fmt::Debug for Mmap2<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Mmap2");
        dbg.field("pid", &self.pid)
            .field("tid", &self.tid)
            .field("addr", &crate::util::fmt::HexAddr(self.addr))
            .field("len", &self.len)
            .field("pgoff", &self.pgoff);

        match &self.detail {
            MmapDetail::Default {
                maj,
                min,
                ino,
                ino_generation,
            } => {
                dbg.field("maj", maj)
                    .field("min", min)
                    .field("ino", ino)
                    .field("ino_generation", ino_generation);
            }
            MmapDetail::BuildId { .. } => {
                if let Some(build_id) = self.build_id() {
                    dbg.field("build_id", &crate::util::fmt::HexStr(build_id));
                }
            }
        }

        dbg.field("prot", &self.prot)
            .field("flags", &self.flags)
            .field("filename", &crate::util::fmt::ByteStr(&self.filename));

        dbg.finish()
    }
}
