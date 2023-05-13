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

use perf_event_open_sys::bindings::perf_event_header;

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
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
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

/// A record emitted by the linux kernel.
///
/// This enum contains every supported record type emitted by the kernel.
/// Depending on how the perf event counter was configured only a few of these
/// will be emitted by any one counter.
#[derive(Clone, Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
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

    /// A record type that is unknown to this crate.
    ///
    /// Note that just because a record is parsed as unknown in one release of
    /// this crate does not mean it will continue to be parsed in future
    /// releases. Adding a new variant to this enum is not considered to be a
    /// breaking change.
    ///
    /// If you find yourself using the unknown variant to parse valid records
    /// emitted by the kernel please file an issue or create a PR to add support
    /// for them.
    Unknown {
        ty: u32,
        data: Cow<'a, [u8]>,
    },
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

impl<'a> crate::Visitor<'a> for RecordVisitor {
    type Output = Record<'a>;

    fn visit_unimplemented(self, metadata: crate::RecordMetadata) -> Self::Output {
        panic!(
            "parsing for records of type {} is not implemented",
            metadata.ty()
        );
    }

    fn visit_mmap(self, record: Mmap<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_lost(self, record: Lost, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_comm(self, record: Comm<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_exit(self, record: Exit, _: crate::RecordMetadata) -> Self::Output {
        Record::Exit(record)
    }

    fn visit_throttle(self, record: Throttle, _: crate::RecordMetadata) -> Self::Output {
        Record::Throttle(record)
    }

    fn visit_unthrottle(self, record: Throttle, _: crate::RecordMetadata) -> Self::Output {
        Record::Unthrottle(record)
    }

    fn visit_fork(self, record: Fork, _: crate::RecordMetadata) -> Self::Output {
        Record::Fork(record)
    }

    fn visit_read(self, record: Read<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_sample(self, record: Sample<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_mmap2(self, record: Mmap2<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_aux(self, record: Aux, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_itrace_start(self, record: ITraceStart, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_lost_samples(self, record: LostSamples, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_switch(self, _: crate::RecordMetadata) -> Self::Output {
        Record::Switch
    }

    fn visit_switch_cpu_wide(
        self,
        record: SwitchCpuWide,
        _: crate::RecordMetadata,
    ) -> Self::Output {
        record.into()
    }

    fn visit_namespaces(self, record: Namespaces<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_ksymbol(self, record: KSymbol<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_bpf_event(self, record: BpfEvent, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_cgroup(self, record: CGroup<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_text_poke(self, record: TextPoke<'a>, _: crate::RecordMetadata) -> Self::Output {
        record.into()
    }

    fn visit_aux_output_hw_id(
        self,
        record: AuxOutputHwId,
        _: crate::RecordMetadata,
    ) -> Self::Output {
        record.into()
    }

    fn visit_unknown(self, data: Cow<'a, [u8]>, metadata: crate::RecordMetadata) -> Self::Output {
        Record::Unknown {
            ty: metadata.ty(),
            data,
        }
    }
}

impl<'p> Record<'p> {
    /// Parse a `Record` using a [`perf_event_header`] that has already been
    /// parsed.
    pub fn parse_with_header<B, E>(
        p: &mut Parser<B, E>,
        header: perf_event_header,
    ) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_record_with_header(RecordVisitor, header)
    }
}

impl<'p> Parse<'p> for Record<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        p.parse_record(RecordVisitor)
    }
}
