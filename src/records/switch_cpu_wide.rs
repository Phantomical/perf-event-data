use perf_event_open_sys::bindings::PERF_RECORD_MISC_SWITCH_OUT;

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
pub enum SwitchCpuWide {
    /// A context switch into the current process.
    In {
        /// The process ID of the process being switched into.
        pid: u32,

        /// The thread IF of the thread being switched into.
        tid: u32,
    },

    /// A context switch away from the current process.
    Out {
        /// The process ID of the process being switched away from.
        pid: u32,

        /// The thread ID of the thread being switched away from.
        tid: u32,
    },
}

impl SwitchCpuWide {
    /// The process ID associated with the switch.
    pub fn pid(&self) -> u32 {
        match *self {
            Self::In { pid, .. } | Self::Out { pid, .. } => pid,
        }
    }

    /// The thread ID associated with the switch.
    pub fn tid(&self) -> u32 {
        match *self {
            Self::In { tid, .. } | Self::Out { tid, .. } => tid,
        }
    }
}

impl<'p> Parse<'p> for SwitchCpuWide {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        let pid = p.parse()?;
        let tid = p.parse()?;

        if p.config().misc() & PERF_RECORD_MISC_SWITCH_OUT as u16 != 0 {
            Ok(Self::Out { pid, tid })
        } else {
            Ok(Self::In { pid, tid })
        }
    }
}
