use std::borrow::Cow;

use perf_event_open_sys::bindings;

use crate::*;

#[derive(Clone, Debug)]
pub struct RecordMetadata {
    ty: u32,
    misc: MiscFlags,
    sample_id: SampleId,
}

impl RecordMetadata {
    pub(crate) fn new(header: bindings::perf_event_header, sample_id: SampleId) -> Self {
        Self {
            ty: header.type_,
            misc: MiscFlags::from_bits_retain(header.misc),
            sample_id,
        }
    }

    /// The type of this record, as emitted by the kernel.
    pub fn ty(&self) -> u32 {
        self.ty
    }

    /// Miscellaneous flags set by the kernel.
    pub fn misc(&self) -> MiscFlags {
        self.misc
    }

    /// If `sample_id_all` was set when configuring the record then this will
    /// contain a subset of the fields configured to be sampled.
    ///
    /// Note that, even if `sample_id_all` is set, MMAP records will always have
    /// an empty `SampleId`. If you want the `SampleId` fields to be set then
    /// configure the kernel to generate MMAP2 records instead.
    pub fn sample_id(&self) -> &SampleId {
        &self.sample_id
    }
}

#[allow(unused_variables)]
pub trait Visitor: Sized {
    type Output<'a>;

    fn visit_mmap(self, record: Mmap<'_>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_lost<'a>(self, record: Lost, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_comm(self, record: Comm<'_>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_exit<'a>(self, record: Exit, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_throttle<'a>(self, record: Throttle, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_unthrottle<'a>(self, record: Throttle, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_fork<'a>(self, record: Fork, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_read(self, record: Read<'_>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_sample(self, record: Sample<'_>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_mmap2(self, record: Mmap2<'_>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_mmap(record.into_mmap(), metadata)
    }

    fn visit_aux<'a>(self, record: Aux, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_itrace_start<'a>(
        self,
        record: ITraceStart,
        metadata: RecordMetadata,
    ) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_lost_samples<'a>(
        self,
        record: LostSamples,
        metadata: RecordMetadata,
    ) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_switch<'a>(self, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_switch_cpu_wide<'a>(self, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_namespaces(
        self,
        record: Namespaces<'_>,
        metadata: RecordMetadata,
    ) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_ksymbol(self, record: KSymbol<'_>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_bpf_event<'a>(self, record: BpfEvent, metadata: RecordMetadata) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_cgroup(self, record: CGroup<'_>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_text_poke(self, record: TextPoke<'_>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_aux_output_hw_id<'a>(
        self,
        record: AuxOutputHwId,
        metadata: RecordMetadata,
    ) -> Self::Output<'a> {
        self.visit_unimplemented(metadata)
    }

    fn visit_unknown(self, data: Cow<'_, [u8]>, metadata: RecordMetadata) -> Self::Output<'_> {
        self.visit_unimplemented(metadata)
    }

    fn visit_unimplemented<'a>(self, metadata: RecordMetadata) -> Self::Output<'a>;
}
