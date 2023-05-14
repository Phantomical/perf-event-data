use std::fmt;

use bitflags::bitflags;
use perf_event_open_sys::bindings::{self, perf_event_attr, PERF_SAMPLE_BRANCH_HW_INDEX};

use crate::endian::Endian;
use crate::{ReadFormat, SampleFlags};

bitflags! {
    /// The set of flags used by the kernel is a lot smaller than the full
    /// available 64 bits. We can shrink the size of the `ParseConfig` object
    /// by cramming them all together into one bitfield.
    ///
    /// This will obviously stop working at some point so there is a canary
    /// test below that will fail once there is only 8 bits of buffer left. At
    /// that point we will need to split this bitfield apart (likely moving
    /// sample_type to its own field).
    #[derive(Copy, Clone, Debug, Default)]
    struct ConfigFlags : u64 {
        const READ_FORMAT = ((1u64 << Self::READ_FORMAT_WIDTH) - 1);
        const SAMPLE_TYPE = (u64::MAX << Self::READ_FORMAT_WIDTH) & (Self::SAMPLE_ID_ALL.bits() - 1);

        const SAMPLE_ID_ALL   = 1 << 46;
        const BRANCH_HW_INDEX = 1 << 47;
        const MISC = u64::MAX << Self::MISC_OFFSET;
    }
}

#[allow(dead_code)]
impl ConfigFlags {
    const MISC_WIDTH: u32 = u16::BITS;
    // Note: we keep one additional bit around within read_format so that we can
    //       mark that we have bits that are not supported by the version of
    //       perf_event_open_sys2 that this crate was compiled against.
    const READ_FORMAT_WIDTH: u32 = (bindings::PERF_FORMAT_MAX - 1).count_ones() + 1;
    const SAMPLE_TYPE_WIDTH: u32 = (bindings::PERF_SAMPLE_MAX - 1).count_ones();

    const READ_FORMAT_OFFSET: u32 = 0;
    const SAMPLE_TYPE_OFFSET: u32 = Self::READ_FORMAT_WIDTH;
    const SAMPLE_ID_ALL_OFFSET: u32 = Self::BRANCH_HW_INDEX_OFFSET - 1;
    const BRANCH_HW_INDEX_OFFSET: u32 = Self::MISC_OFFSET - 1;
    const MISC_OFFSET: u32 = u64::BITS - Self::MISC_WIDTH;
}

impl ConfigFlags {
    fn new(
        read_format: ReadFormat,
        sample_type: SampleFlags,
        sample_id_all: bool,
        branch_hw_index: bool,
        misc: u16,
    ) -> Self {
        let mut bits = 0u64;
        bits |= (sample_id_all as u64) << Self::SAMPLE_ID_ALL_OFFSET;
        bits |= (branch_hw_index as u64) << Self::BRANCH_HW_INDEX_OFFSET;
        bits |= (misc as u64) << Self::MISC_OFFSET;

        let mut flags = Self::from_bits_retain(bits);
        flags.set_read_format(read_format);
        flags.set_sample_type(sample_type);
        flags.set_misc(misc);

        flags
    }

    fn read_format(&self) -> ReadFormat {
        ReadFormat::from_bits_retain((*self & Self::READ_FORMAT).bits() >> Self::READ_FORMAT_OFFSET)
    }

    fn sample_type(&self) -> SampleFlags {
        SampleFlags::from_bits_retain(
            (*self & Self::SAMPLE_TYPE).bits() >> Self::SAMPLE_TYPE_OFFSET,
        )
    }

    fn sample_id_all(&self) -> bool {
        self.contains(Self::SAMPLE_ID_ALL)
    }

    fn branch_hw_index(&self) -> bool {
        self.contains(Self::BRANCH_HW_INDEX)
    }

    fn misc(&self) -> u16 {
        ((*self & Self::MISC).bits() >> Self::MISC_OFFSET) as _
    }

    fn set_misc(&mut self, misc: u16) {
        *self &= !Self::MISC;
        *self |= Self::from_bits_retain((misc as u64) << Self::MISC_OFFSET);
    }

    fn set_sample_type(&mut self, sample_type: SampleFlags) {
        *self &= !Self::SAMPLE_TYPE;
        *self |= Self::from_bits_retain(sample_type.bits() << Self::SAMPLE_TYPE_OFFSET)
            & Self::SAMPLE_TYPE;
    }

    fn set_read_format(&mut self, read_format: ReadFormat) {
        *self &= !Self::READ_FORMAT;
        *self |= Self::from_bits_retain(read_format.bits() << Self::READ_FORMAT_OFFSET)
            & Self::READ_FORMAT;
        *self |= Self::from_bits_retain(
            ((read_format.bits() >> Self::READ_FORMAT_WIDTH != 0) as u64)
                << (Self::READ_FORMAT_OFFSET + Self::READ_FORMAT_WIDTH - 1),
        );
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub(crate) struct RawParseConfig {
    config_flags: ConfigFlags,
    sample_regs_user: u64,
    sample_regs_intr: u64,
}

/// All the configuration data needed to parse any perf record.
#[derive(Clone, Default)]
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
        self.config.config_flags.set_sample_type(sample_type);
        self
    }

    pub(crate) fn with_misc(mut self, misc: u16) -> Self {
        self.config.config_flags.set_misc(misc);
        self
    }
}

impl<E> ParseConfig<E> {
    /// Flags controlling what fields are returned by the kernel when reading
    /// from a counter.
    pub fn read_format(&self) -> ReadFormat {
        self.config.config_flags.read_format()
    }

    /// Flags indicating which fields are captured by the kernel when
    /// collecting a sample.
    pub fn sample_type(&self) -> SampleFlags {
        self.config.config_flags.sample_type()
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
        self.config.config_flags.sample_id_all()
    }

    pub(crate) fn branch_hw_index(&self) -> bool {
        self.config.config_flags.branch_hw_index()
    }

    pub(crate) fn misc(&self) -> u16 {
        self.config.config_flags.misc()
    }

    /// The [`Endian`] for this `ParseConfig`.
    pub fn endian(&self) -> &E {
        &self.endian
    }
}

impl From<perf_event_attr> for RawParseConfig {
    fn from(attrs: perf_event_attr) -> Self {
        Self {
            config_flags: ConfigFlags::new(
                ReadFormat::from_bits_retain(attrs.read_format),
                SampleFlags::from_bits_retain(attrs.sample_type),
                attrs.sample_id_all() != 0,
                (attrs.branch_sample_type & PERF_SAMPLE_BRANCH_HW_INDEX as u64) != 0,
                0,
            ),
            sample_regs_user: attrs.sample_regs_user,
            sample_regs_intr: attrs.sample_regs_intr,
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

impl<E: fmt::Debug> fmt::Debug for ParseConfig<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseConfig")
            .field("read_format", &self.read_format())
            .field("sample_type", &self.sample_type())
            .field("sample_id_all", &self.sample_id_all())
            .field("branch_hw_index", &self.branch_hw_index())
            .field("misc", &format_args!("0x{:X}", self.misc()))
            .field("regs_user", &format_args!("0x{:X}", self.regs_user()))
            .field("regs_intr", &format_args!("0x{:X}", self.regs_intr()))
            .finish()
    }
}

#[cfg(feature = "arbitrary")]
mod fuzzing {
    use super::*;

    use arbitrary::{Arbitrary, Result, Unstructured};

    impl<'a, E: Endian + Default> arbitrary::Arbitrary<'a> for ParseConfig<E> {
        fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
            Ok(Self {
                endian: E::default(),
                config: RawParseConfig::arbitrary(u)?,
            })
        }
    }

    impl<'a> Arbitrary<'a> for ConfigFlags {
        fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
            Ok(Self::from_bits_retain(Arbitrary::arbitrary(u)?))
        }
    }
}

#[test]
fn assert_sufficient_spare_sample_type_bits() {
    assert!(ConfigFlags::SAMPLE_TYPE.bits().count_ones() >= ConfigFlags::SAMPLE_TYPE_WIDTH + 8)
}
