use perf_event_open_sys::bindings::perf_event_attr;

use crate::endian::Endian;
use crate::flags::BranchSampleFlags;
use crate::{ReadFormat, SampleFlags};

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct RawParseConfig {
    sample_type: SampleFlags,
    read_format: ReadFormat,
    sample_regs_user: u64,
    sample_regs_intr: u64,
    sample_id_all: bool,
    branch_hw_index: bool,
}

/// All the configuration data needed to parse any perf record.
#[derive(Clone, Debug, Default)]
pub struct ParseConfig<E> {
    config: RawParseConfig,
    endian: E,
}

impl<E> ParseConfig<E> {
    /// Use this `ParseConfig` with a different `Endian`.
    pub fn with_endian<E2: Endian>(self, endian: E2) -> ParseConfig<E2> {
        ParseConfig {
            endian,
            config: self.config,
        }
    }

    #[allow(dead_code)]
    /// Used for testing, please open an issue if you need this.
    pub(crate) fn with_sample_type(mut self, sample_type: SampleFlags) -> Self {
        self.config.sample_type = sample_type;
        self
    }
}

impl<E: Endian> ParseConfig<E> {
    /// Flags controlling what fields are returned by the kernel when reading
    /// from a counter.
    pub fn read_format(&self) -> ReadFormat {
        self.config.read_format
    }

    /// Flags indicating which fields are captured by the kernel when
    /// collecting a sample.
    pub fn sample_type(&self) -> SampleFlags {
        self.config.sample_type
    }

    /// Bitmask indicating which user-space registers are saved when the kernel
    /// takes a sample.
    pub fn regs_user(&self) -> u64 {
        self.config.sample_regs_user
    }

    /// Bitmask indicating which registers are saved when the kernel takes a
    /// sample.
    ///
    /// Unlike [`regs_user`](Self::regs_user), the source registers may be in
    /// either kernel-space or user-space depending on where the perf sampling
    /// interrupt occurred.
    pub fn regs_intr(&self) -> u64 {
        self.config.sample_regs_intr
    }

    pub(crate) fn sample_id_all(&self) -> bool {
        self.config.sample_id_all
    }

    pub(crate) fn branch_hw_index(&self) -> bool {
        self.config.branch_hw_index
    }

    /// The [`Endian`] for this `ParseConfig`.
    pub fn endian(&self) -> &E {
        &self.endian
    }
}

impl From<perf_event_attr> for RawParseConfig {
    fn from(attrs: perf_event_attr) -> Self {
        Self {
            sample_type: SampleFlags::from_bits_retain(attrs.sample_type),
            read_format: ReadFormat::from_bits_retain(attrs.read_format),
            sample_regs_user: attrs.sample_regs_user,
            sample_regs_intr: attrs.sample_regs_intr,
            branch_hw_index: BranchSampleFlags::from_bits_retain(attrs.branch_sample_type)
                .contains(BranchSampleFlags::HW_INDEX),
            sample_id_all: attrs.sample_id_all() != 0,
        }
    }
}

impl<E> From<perf_event_attr> for ParseConfig<E>
where
    E: Default,
{
    fn from(value: perf_event_attr) -> Self {
        Self {
            endian: E::default(),
            config: RawParseConfig::from(value),
        }
    }
}
