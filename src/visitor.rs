use std::borrow::Cow;

use perf_event_open_sys::bindings;

use crate::parse::Parser;
use crate::*;

used_in_docs!(Parser);

/// Extra record data emitted by the kernel that is common to all records.
#[derive(Clone, Debug)]
pub struct RecordMetadata {
    ty: u32,
    misc: u16,
    sample_id: SampleId,
}

impl RecordMetadata {
    #[inline]
    pub(crate) fn new(header: bindings::perf_event_header, sample_id: SampleId) -> Self {
        Self {
            ty: header.type_,
            misc: header.misc,
            sample_id,
        }
    }

    /// The type of this record, as emitted by the kernel.
    #[inline]
    pub fn ty(&self) -> u32 {
        self.ty
    }

    /// Miscellaneous flags set by the kernel.
    #[inline]
    pub fn misc(&self) -> u16 {
        self.misc
    }

    /// If `sample_id_all` was set when configuring the record then this will
    /// contain a subset of the fields configured to be sampled.
    ///
    /// Note that, even if `sample_id_all` is set, MMAP and SAMPLE records will
    /// always have an empty `SampleId`. If you want the `SampleId` fields
    /// to be set then configure the kernel to generate MMAP2 records
    /// instead.
    #[inline]
    pub fn sample_id(&self) -> &SampleId {
        &self.sample_id
    }
}

/// A visitor for visiting parsed records.
///
/// This is used in combination with [`Parser::parse_record`] to parse the
/// record types that you are interested in.
///
/// # Implementing a Visitor
/// To implement a visitor define your output type and implement
/// `visit_unimplemented`, then, implement whichever `visit_*` method that is
/// for the event you are interested in:
///
/// ```
/// # use perf_event_data::{Visitor, RecordMetadata};
/// struct MyVisitor;
///
/// impl Visitor<'_> for MyVisitor {
///     type Output = ();
///
///     fn visit_unimplemented(self, metadata: RecordMetadata) -> Self::Output {
///         println!("got a record with type {}", metadata.ty());
///     }
/// }
/// ```
#[allow(unused_variables)]
pub trait Visitor<'a>: Sized {
    /// The output type for this visitor.
    type Output;

    /// Called by the other `visit_*` methods if they are not implemented.
    ///
    /// When implementing a visitor this is this one method that it is required
    /// to implement.
    fn visit_unimplemented(self, metadata: RecordMetadata) -> Self::Output;

    /// Visit a [`Mmap`] record.
    ///
    /// By default, [`visit_mmap2`](Visitor::visit_mmap2) forwards to this
    /// method.
    fn visit_mmap(self, record: Mmap<'a>, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`Lost`] record.
    fn visit_lost(self, record: Lost, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`Comm`] record.
    fn visit_comm(self, record: Comm<'a>, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit an [`Exit`] record.
    fn visit_exit(self, record: Exit, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a THROTTLE record.
    fn visit_throttle(self, record: Throttle, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit an UNTHROTTLE record.
    fn visit_unthrottle(self, record: Throttle, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`Fork`] record.
    fn visit_fork(self, record: Fork, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`Read`] record.
    fn visit_read(self, record: Read, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`Sample`] record.
    fn visit_sample(self, record: Sample<'a>, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`Mmap2`] record.
    ///
    /// If not implemented, this forwards to
    /// [`visit_mmap`](Visitor::visit_mmap).
    fn visit_mmap2(self, record: Mmap2<'a>, metadata: RecordMetadata) -> Self::Output {
        self.visit_mmap(record.into_mmap(), metadata)
    }

    /// Visit an [`Aux`] record.
    fn visit_aux(self, record: Aux, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit an [`ITraceStart`] record.
    fn visit_itrace_start(self, record: ITraceStart, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`LostSamples`] record.
    fn visit_lost_samples(self, record: LostSamples, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a SWITCH record.
    fn visit_switch(self, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`SwitchCpuWide`] record.
    fn visit_switch_cpu_wide(
        self,
        record: SwitchCpuWide,
        metadata: RecordMetadata,
    ) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`Namespaces`] record.
    fn visit_namespaces(self, record: Namespaces<'a>, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`KSymbol`] record.
    fn visit_ksymbol(self, record: KSymbol<'a>, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`BpfEvent`] record.
    fn visit_bpf_event(self, record: BpfEvent, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`CGroup`] record.
    fn visit_cgroup(self, record: CGroup<'a>, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`TextPoke`] record.
    fn visit_text_poke(self, record: TextPoke<'a>, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a [`AuxOutputHwId`] record.
    fn visit_aux_output_hw_id(
        self,
        record: AuxOutputHwId,
        metadata: RecordMetadata,
    ) -> Self::Output {
        self.visit_unimplemented(metadata)
    }

    /// Visit a record not supported by this library.
    ///
    /// Note that support for new record types may be added in new minor
    /// versions of `perf-event-data`. This visitor method is provided as a
    /// backstop so that users can still choose to handle these should they need
    /// it.
    ///
    /// If you find yourself using this for a record type emitted by
    /// `perf_event_open` please create an issue or submit a PR to add the
    /// record to `perf-event-data` itself.
    fn visit_unknown(self, data: Cow<'a, [u8]>, metadata: RecordMetadata) -> Self::Output {
        self.visit_unimplemented(metadata)
    }
}
