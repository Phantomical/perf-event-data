use crate::prelude::*;

/// EXIT records indicate that a process has exited.
///
/// This struct corresponds to `PERF_RECORD_EXIT`. See the [manpage] for more
/// documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Exit {
    pub pid: u32,
    pub ppid: u32,
    pub tid: u32,
    pub ptid: u32,
    pub time: u64,
}

impl<'p> Parse<'p> for Exit {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            pid: p.parse()?,
            ppid: p.parse()?,
            tid: p.parse()?,
            ptid: p.parse()?,
            time: p.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endian::Little;

    #[test]
    #[cfg_attr(not(target_endian = "little"), ignore)]
    fn test_parse() {
        #[rustfmt::skip]
        let bytes: &[u8] = &[
            0x10, 0x10, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
        ];

        let mut parser: Parser<_, Little> = Parser::new(bytes, ParseConfig::default());
        let exit: Exit = parser.parse().unwrap();

        assert_eq!(exit.pid, 0x1010);
        assert_eq!(exit.ppid, 0x0500);
        assert_eq!(exit.tid, 0x01);
        assert_eq!(exit.ptid, 0x02);
        assert_eq!(exit.time, 0x0400000003);
    }
}
