use crate::prelude::*;

/// ITRACE_START records indicate when a process has started an instruction
/// trace.
///
/// This struct corresponds to `PERF_RECORD_ITRACE_START`. See the [manpage]
/// for more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct ITraceStart {
    /// Process ID of thread starting an instruction trace.
    pub pid: u32,

    /// Thread ID of thread starting an instruction trace.
    pub tid: u32,
}

impl<'p> Parse<'p> for ITraceStart {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            pid: p.parse()?,
            tid: p.parse()?,
        })
    }
}
