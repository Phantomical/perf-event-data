use crate::prelude::*;

/// SWITCH_CPU_WIDE records indicates a context switch when profiling in
/// cpu-wide mode.
///
/// It provides some additional info on the process being switched that is not
/// provided by SWITCH.
///
/// If the `PERF_RECORD_MISC_SWITCH_OUT` bit is set within the record header
/// then it was a context switch away from the current process, otherwise it is
/// a context switch into the current process.
///
/// This enum corresponds to `PERF_RECORD_SWITCH_CPU_WIDE`. See the [manpage]
/// for more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Copy, Clone, Debug)]
pub struct SwitchCpuWide {
    /// The process ID associated with the context switch.
    ///
    /// Depending on whether this is a switch-in or a switch-out this will be
    /// the incoming process ID or outgoing process ID, respectively.
    pub pid: u32,

    /// The thread ID associated with the context switch.
    ///
    /// Depending on whether this is a switch-in or a switch-out this will be
    /// the incoming thread ID or the outgoing thread ID, respectively.
    pub tid: u32,
}

impl<'p> Parse<'p> for SwitchCpuWide {
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
