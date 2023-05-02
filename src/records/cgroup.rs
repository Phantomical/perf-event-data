use crate::prelude::*;
use std::borrow::Cow;

/// CGROUP records indicate when a new cgroup is created and activated.
///
/// This struct corresponds to `PERF_RECORD_CGROUP`. See the [manpage] for more
/// documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone)]
pub struct Cgroup<'a> {
    /// The cgroup ID.
    pub id: u64,

    /// Path of the cgroup from the root.
    pub path: Cow<'a, [u8]>,
}

impl<'a> Cgroup<'a> {
    /// Get `path` as a [`Path`](std::path::Path).
    #[cfg(unix)]
    pub fn path_os(&self) -> &std::path::Path {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::Path;

        Path::new(OsStr::from_bytes(&self.path))
    }

    pub fn into_owned(self) -> Cgroup<'static> {
        Cgroup {
            path: self.path.into_owned().into(),
            ..self
        }
    }
}

impl<'p> Parse<'p> for Cgroup<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            id: p.parse()?,
            path: p.parse_rest_trim_nul()?,
        })
    }
}