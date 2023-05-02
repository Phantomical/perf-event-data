use crate::prelude::*;
use std::borrow::Cow;

use crate::MiscFlags;

used_in_docs!(MiscFlags);

/// COMM records indicate changes in process names.
///
/// There are multiple ways that this could happen: [`execve(2)`],
/// [`prctl(PR_SET_NAME)`], as well as writing to `/proc/self/comm`.
///
/// Since Linux 3.10 the kernel will set the [`COMM_EXEC`] bit in
/// [`MiscFlags`] if the record is due to an [`execve(2)`] syscall.
/// You can set `comm_exec` when building to detect whether this is supported.
///
/// This struct corresponds to `PERF_RECORD_COMM`. See the [manpage] for more
/// documentation.
///
/// [`execve(2)`]: https://man7.org/linux/man-pages/man2/execve.2.html
/// [`prctl(PR_SET_NAME)`]: https://man7.org/linux/man-pages/man2/prctl.2.html
/// [`COMM_EXEC`]: MiscFlags::COMM_EXEC
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Comm<'a> {
    pub pid: u32,
    pub tid: u32,
    pub comm: Cow<'a, [u8]>,
}

impl<'a> Comm<'a> {
    #[cfg(unix)]
    pub fn comm_os(&self) -> &std::ffi::OsStr {
        use std::os::unix::ffi::OsStrExt;

        OsStrExt::from_bytes(&self.comm)
    }

    pub fn into_owned(self) -> Comm<'static> {
        Comm {
            comm: self.comm.into_owned().into(),
            ..self
        }
    }
}

impl<'p> Parse<'p> for Comm<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            pid: p.parse()?,
            tid: p.parse()?,
            comm: p.parse_rest_trim_nul()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endian::Little;

    #[test]
    fn test_parse() {
        #[rustfmt::skip]
        let bytes: &[u8] = &[
            0x10, 0x10, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00,
            b't', b'e', b's', b't', 0x00, 0x00, 0x00, 0x00
        ];

        let mut parser: Parser<_, Little> = Parser::new(bytes, ParseConfig::default());
        let comm: Comm = parser.parse().unwrap();

        assert_eq!(comm.pid, 0x1010);
        assert_eq!(comm.tid, 0x0500);
        assert_eq!(&*comm.comm, b"test");
    }
}