//! This module contains the actual structs.
//!
//! This is mostly to separate them from the support code of this crate.

#![allow(missing_docs)]

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

use std::borrow::Cow;
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

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Record<'a> {
    Mmap(Mmap<'a>),
    Lost(Lost),
    Comm(Comm<'a>),
    Exit(Exit),
    Throttle(Throttle),
    Unthrottle(Throttle),
    Fork(Fork),
    Read(Read<'a>),
    Sample(Box<Sample<'a>>),
    Mmap2(Mmap2<'a>),
    Aux(Aux),
    ITraceStart(ITraceStart),
    LostSamples(LostSamples),
    Switch,
    SwitchCpuWide(SwitchCpuWide),
    Namespaces(Namespaces<'a>),
    KSymbol(KSymbol<'a>),
    BpfEvent(BpfEvent),
    CGroup(CGroup<'a>),
    TextPoke(TextPoke<'a>),
    AuxOutputHwId(AuxOutputHwId),
    Unknown { ty: u32, data: Cow<'a, [u8]> },
}

macro_rules! record_from {
    ($ty:ident) => {
        impl<'a> From<$ty> for Record<'a> {
            fn from(value: $ty) -> Self {
                Self::$ty(value)
            }
        }
    };
    ($ty:ident<$lt:lifetime>) => {
        impl<$lt> From<$ty<$lt>> for Record<$lt> {
            fn from(value: $ty<$lt>) -> Self {
                Self::$ty(value)
            }
        }
    };
}

record_from!(Mmap<'a>);
record_from!(Lost);
record_from!(Comm<'a>);
// These are both the same struct
// record_from!(Exit);
// record_from!(Fork);
record_from!(Read<'a>);
record_from!(Mmap2<'a>);
record_from!(Aux);
record_from!(ITraceStart);
record_from!(LostSamples);
record_from!(SwitchCpuWide);
record_from!(Namespaces<'a>);
record_from!(KSymbol<'a>);
record_from!(BpfEvent);
record_from!(CGroup<'a>);
record_from!(TextPoke<'a>);
record_from!(AuxOutputHwId);

impl<'a> From<Sample<'a>> for Record<'a> {
    fn from(value: Sample<'a>) -> Self {
        Self::Sample(Box::new(value))
    }
}

struct RecordVisitor;

impl crate::Visitor for RecordVisitor {
    type Output<'a> = crate::parse::Result<Record<'a>>;

    fn visit_unimplemented<'a>(self, metadata: crate::RecordMetadata) -> Self::Output<'a> {
        panic!(
            "parsing for records of type {} is not implemented",
            metadata.ty()
        );
    }

    fn visit_mmap(self, record: Mmap<'_>, _: crate::RecordMetadata) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_lost<'a>(self, record: Lost, _: crate::RecordMetadata) -> Self::Output<'a> {
        Ok(record.into())
    }

    fn visit_comm(self, record: Comm<'_>, _: crate::RecordMetadata) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_exit<'a>(self, record: Exit, _: crate::RecordMetadata) -> Self::Output<'a> {
        Ok(Record::Exit(record))
    }

    fn visit_throttle<'a>(self, record: Throttle, _: crate::RecordMetadata) -> Self::Output<'a> {
        Ok(Record::Throttle(record))
    }

    fn visit_unthrottle<'a>(self, record: Throttle, _: crate::RecordMetadata) -> Self::Output<'a> {
        Ok(Record::Unthrottle(record))
    }

    fn visit_fork<'a>(self, record: Fork, _: crate::RecordMetadata) -> Self::Output<'a> {
        Ok(Record::Fork(record))
    }

    fn visit_read(self, record: Read<'_>, _: crate::RecordMetadata) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_sample(self, record: Sample<'_>, _: crate::RecordMetadata) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_mmap2(self, record: Mmap2<'_>, _: crate::RecordMetadata) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_aux<'a>(self, record: Aux, _: crate::RecordMetadata) -> Self::Output<'a> {
        Ok(record.into())
    }

    fn visit_itrace_start<'a>(
        self,
        record: ITraceStart,
        _: crate::RecordMetadata,
    ) -> Self::Output<'a> {
        Ok(record.into())
    }

    fn visit_lost_samples<'a>(
        self,
        record: LostSamples,
        _: crate::RecordMetadata,
    ) -> Self::Output<'a> {
        Ok(record.into())
    }

    fn visit_switch<'a>(self, _: crate::RecordMetadata) -> Self::Output<'a> {
        Ok(Record::Switch)
    }

    fn visit_switch_cpu_wide<'a>(
        self,
        record: SwitchCpuWide,
        _: crate::RecordMetadata,
    ) -> Self::Output<'a> {
        Ok(record.into())
    }

    fn visit_namespaces(
        self,
        record: Namespaces<'_>,
        _: crate::RecordMetadata,
    ) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_ksymbol(self, record: KSymbol<'_>, _: crate::RecordMetadata) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_bpf_event<'a>(self, record: BpfEvent, _: crate::RecordMetadata) -> Self::Output<'a> {
        Ok(record.into())
    }

    fn visit_cgroup(self, record: CGroup<'_>, _: crate::RecordMetadata) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_text_poke(self, record: TextPoke<'_>, _: crate::RecordMetadata) -> Self::Output<'_> {
        Ok(record.into())
    }

    fn visit_aux_output_hw_id<'a>(
        self,
        record: AuxOutputHwId,
        _: crate::RecordMetadata,
    ) -> Self::Output<'a> {
        Ok(record.into())
    }

    fn visit_unknown(
        self,
        data: Cow<'_, [u8]>,
        metadata: crate::RecordMetadata,
    ) -> Self::Output<'_> {
        Ok(Record::Unknown {
            ty: metadata.ty(),
            data,
        })
    }
}
