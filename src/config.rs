use perf_event_open_sys::bindings::{perf_event_attr, PERF_SAMPLE_BRANCH_HW_INDEX};

use crate::endian::Endian;
use crate::{ReadFormat, SampleFlags};

#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub(crate) struct RawParseConfig {
    sample_type: SampleFlags,
    read_format: ReadFormat,
    sample_regs_user: u64,
    sample_regs_intr: u64,
    sample_id_all: bool,
    branch_hw_index: bool,
    misc: u16,
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

    pub(crate) fn with_misc(mut self, misc: u16) -> Self {
        self.config.misc = misc;
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

    pub(crate) fn misc(&self) -> u16 {
        self.config.misc
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
            branch_hw_index: (attrs.branch_sample_type & PERF_SAMPLE_BRANCH_HW_INDEX as u64) != 0,
            sample_id_all: attrs.sample_id_all() != 0,
            misc: 0,
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

#[cfg(feature = "arbitrary")]
impl<'a, E: Endian + Default> arbitrary::Arbitrary<'a> for ParseConfig<E> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(Self {
            endian: E::default(),
            config: RawParseConfig::arbitrary(u)?,
        })
    }
}
