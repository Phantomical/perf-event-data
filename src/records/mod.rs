//! This module contains the actual structs.
//!
//! This is mostly to separate them from the support code of this crate.

mod aux;
mod aux_output_hw_id;
mod bpf_event;
mod cgroup;
mod comm;
mod exit;
mod itrace_start;
mod ksymbol;
mod lost;
mod lost_samples;
mod mmap;
mod mmap2;
mod namespaces;
mod read;
mod sample;
mod switch_cpu_wide;
mod text_poke;
mod throttle;

pub use self::aux::*;
pub use self::aux_output_hw_id::*;
pub use self::bpf_event::*;
pub use self::cgroup::*;
pub use self::comm::*;
pub use self::exit::*;
pub use self::itrace_start::*;
pub use self::ksymbol::*;
pub use self::lost::*;
pub use self::lost_samples::*;
pub use self::mmap::*;
pub use self::mmap2::*;
pub use self::namespaces::*;
pub use self::read::*;
pub use self::sample::*;
pub use self::switch_cpu_wide::*;
pub use self::text_poke::*;
pub use self::throttle::*;

/// FORK records indicate that a process called [`fork(2)`] successfully.
///
/// This struct corresponds to `PERF_RECORD_FORK`. See the [manpage] for more
/// documentation.
///
/// [`fork(2)`]: https://man7.org/linux/man-pages/man2/fork.2.html
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
pub type Fork = Exit;

mod sample_id {
    option_struct! {
        ##[copy]
        pub(super) struct SampleId : u8 {
            pub pid: u32,
            pub tid: u32,
            pub time: u64,
            pub id: u64,
            pub stream_id: u64,
            pub cpu: u32,
        }
    }
}

use std::fmt;

use crate::prelude::*;

/// A subset of the sample fields that can be recorded in non-SAMPLE records.
///
/// This will be empty by default unless `sample_id_all` was set when
/// configuring the perf event counter.
#[derive(Copy, Clone, Default)]
pub struct SampleId(sample_id::SampleId);

impl SampleId {
    /// The process ID that generated this event.
    pub fn pid(&self) -> Option<u32> {
        self.0.pid().copied()
    }

    /// The thread ID that generated this event.
    pub fn tid(&self) -> Option<u32> {
        self.0.tid().copied()
    }

    /// The time at which this event was recorded.
    ///
    /// The clock used to record the time depends on how the clock was
    /// configured when setting up the counter.
    pub fn time(&self) -> Option<u64> {
        self.0.time().copied()
    }

    /// The unique kernel-assigned ID for the leader of this counter group.
    pub fn id(&self) -> Option<u64> {
        self.0.id().copied()
    }

    /// The unique kernel-assigned ID for the counter that generated this event.
    pub fn stream_id(&self) -> Option<u64> {
        self.0.stream_id().copied()
    }

    /// The CPU on which this event was recorded.
    pub fn cpu(&self) -> Option<u32> {
        self.0.cpu().copied()
    }

    /// Get the length in bytes that this struct would need to be parsed from
    /// the provided parser.
    pub fn estimate_len<E: Endian>(config: &ParseConfig<E>) -> usize {
        let sty = config.sample_type();

        if !config.sample_id_all() {
            return 0;
        }

        let flags = SampleFlags::TID
            | SampleFlags::TIME
            | SampleFlags::ID
            | SampleFlags::STREAM_ID
            | SampleFlags::CPU
            | SampleFlags::IDENTIFIER;

        (sty & flags).bits().count_ones() as usize * std::mem::size_of::<u64>()
    }
}

impl<'p> Parse<'p> for SampleId {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        let config = p.config();
        let sty = config.sample_type();

        if !config.sample_id_all() {
            return Ok(Self::default());
        }

        let pid = p.parse_if(sty.contains(SampleFlags::TID))?;
        let tid = p.parse_if(sty.contains(SampleFlags::TID))?;
        let time = p.parse_if(sty.contains(SampleFlags::TIME))?;
        let id = p.parse_if(sty.contains(SampleFlags::ID))?;
        let stream_id = p.parse_if(sty.contains(SampleFlags::STREAM_ID))?;
        let cpu = p.parse_if_with(sty.contains(SampleFlags::CPU), |p| {
            Ok((p.parse_u32()?, p.parse_u32()?).0)
        })?;
        let identifier = p.parse_if(sty.contains(SampleFlags::IDENTIFIER))?;

        Ok(Self(sample_id::SampleId::new(
            pid,
            tid,
            time,
            id.or(identifier),
            stream_id,
            cpu,
        )))
    }
}

impl fmt::Debug for SampleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
