#![allow(missing_docs)]

use bitflags::bitflags;
use perf_event_open_sys::bindings;

use crate::Sample;

used_in_docs!(Sample);

bitflags! {
    /// Specifies which fields to include in the sample.
    ///
    /// These values correspond to `PERF_SAMPLE_x` values. See the
    /// [manpage] for documentation on what they mean.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
    pub struct SampleFlags : u64 {
        const IP = bindings::PERF_SAMPLE_IP as _;
        const TID = bindings::PERF_SAMPLE_TID as _;
        const TIME = bindings::PERF_SAMPLE_TIME as _;
        const ADDR = bindings::PERF_SAMPLE_ADDR as _;
        const READ = bindings::PERF_SAMPLE_READ as _;
        const CALLCHAIN = bindings::PERF_SAMPLE_CALLCHAIN as _;
        const ID = bindings::PERF_SAMPLE_ID as _;
        const CPU = bindings::PERF_SAMPLE_CPU as _;
        const PERIOD = bindings::PERF_SAMPLE_PERIOD as _;
        const STREAM_ID = bindings::PERF_SAMPLE_STREAM_ID as _;
        const RAW = bindings::PERF_SAMPLE_RAW as _;
        const BRANCH_STACK = bindings::PERF_SAMPLE_BRANCH_STACK as _;
        const REGS_USER = bindings::PERF_SAMPLE_REGS_USER as _;
        const STACK_USER = bindings::PERF_SAMPLE_STACK_USER as _;
        const WEIGHT = bindings::PERF_SAMPLE_WEIGHT as _;
        const DATA_SRC = bindings::PERF_SAMPLE_DATA_SRC as _;
        const IDENTIFIER = bindings::PERF_SAMPLE_IDENTIFIER as _;
        const TRANSACTION = bindings::PERF_SAMPLE_TRANSACTION as _;
        const REGS_INTR = bindings::PERF_SAMPLE_REGS_INTR as _;
        const PHYS_ADDR = bindings::PERF_SAMPLE_PHYS_ADDR as _;
        const AUX = bindings::PERF_SAMPLE_AUX as _;
        const CGROUP = bindings::PERF_SAMPLE_CGROUP as _;

        // The following are present in perf_event.h but not yet documented
        // in the manpage.
        const DATA_PAGE_SIZE = bindings::PERF_SAMPLE_DATA_PAGE_SIZE as _;
        const CODE_PAGE_SIZE = bindings::PERF_SAMPLE_CODE_PAGE_SIZE as _;
        const WEIGHT_STRUCT = bindings::PERF_SAMPLE_WEIGHT_STRUCT as _;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
    pub struct BranchSampleFlags : u64 {
        const USER          = bindings::PERF_SAMPLE_BRANCH_USER as _;
        const KERNEL        = bindings::PERF_SAMPLE_BRANCH_KERNEL as _;
        const HV            = bindings::PERF_SAMPLE_BRANCH_HV as _;
        const ANY           = bindings::PERF_SAMPLE_BRANCH_ANY as _;
        const ANY_CALL      = bindings::PERF_SAMPLE_BRANCH_ANY_CALL as _;
        const ANY_RETURN    = bindings::PERF_SAMPLE_BRANCH_ANY_RETURN as _;
        const IND_CALL      = bindings::PERF_SAMPLE_BRANCH_IND_CALL as _;
        const ABORT_TX      = bindings::PERF_SAMPLE_BRANCH_ABORT_TX as _;
        const IN_TX         = bindings::PERF_SAMPLE_BRANCH_IN_TX as _;
        const NO_TX         = bindings::PERF_SAMPLE_BRANCH_NO_TX as _;
        const COND          = bindings::PERF_SAMPLE_BRANCH_COND as _;
        const CALL_STACK    = bindings::PERF_SAMPLE_BRANCH_CALL_STACK as _;
        const IND_JUMP      = bindings::PERF_SAMPLE_BRANCH_IND_JUMP as _;
        const CALL          = bindings::PERF_SAMPLE_BRANCH_CALL as _;
        const NO_FLAGS      = bindings::PERF_SAMPLE_BRANCH_NO_FLAGS as _;
        const NO_CYCLES     = bindings::PERF_SAMPLE_BRANCH_NO_CYCLES as _;
        const TYPE_SAVE     = bindings::PERF_SAMPLE_BRANCH_TYPE_SAVE as _;
        const HW_INDEX      = bindings::PERF_SAMPLE_BRANCH_HW_INDEX as _;
        const PRIV_SAVE     = bindings::PERF_SAMPLE_BRANCH_PRIV_SAVE as _;
    }
}

bitflags! {
    /// Flags that control what data is returned when reading from a
    /// perf_event file descriptor.
    ///
    /// See the [man page][0] for the authoritative documentation on what
    /// these flags do.
    ///
    /// [0]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
    pub struct ReadFormat : u64 {
        /// Emit the total amount of time the counter has spent enabled.
        const TOTAL_TIME_ENABLED = bindings::PERF_FORMAT_TOTAL_TIME_ENABLED as _;

        /// Emit the total amount of time the counter was actually on the
        /// CPU.
        const TOTAL_TIME_RUNNING = bindings::PERF_FORMAT_TOTAL_TIME_RUNNING as _;

        /// Emit the counter ID.
        const ID = bindings::PERF_FORMAT_ID as _;

        /// If in a group, read all the counters in the group at once.
        const GROUP = bindings::PERF_FORMAT_GROUP as _;

        /// Emit the number of lost samples for this event.
        const LOST = bindings::PERF_FORMAT_LOST as _;
    }
}

impl ReadFormat {
    // The format of a read from a group is like this
    // struct read_format {
    //     u64 nr;            /* The number of events */
    //     u64 time_enabled;  /* if PERF_FORMAT_TOTAL_TIME_ENABLED */
    //     u64 time_running;  /* if PERF_FORMAT_TOTAL_TIME_RUNNING */
    //     struct {
    //         u64 value;     /* The value of the event */
    //         u64 id;        /* if PERF_FORMAT_ID */
    //         u64 lost;      /* if PERF_FORMAT_LOST */
    //     } values[nr];
    // };

    /// The size of each element when reading a group
    pub(crate) fn element_len(&self) -> usize {
        1 + (*self & (Self::ID | Self::LOST)).bits().count_ones() as usize
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct MiscFlags : u16 {
        /// The first few bits of `MiscFlags` actually contain an enum value.
        const CPUMODE_MASK = bindings::PERF_RECORD_MISC_CPUMODE_MASK as _;

        /// Indicates that the mapping that caused a MMAP or MMAP2 record to
        /// be generated is not executable mapping.
        ///
        /// Note that [`MMAP_DATA`](Self::MMAP_DATA),
        /// [`COMM_EXEC`](Self::COMM_EXEC), and
        /// [`SWITCH_OUT`](Self::SWITCH_OUT) are all the same bit.
        const MMAP_DATA = bindings::PERF_RECORD_MISC_MMAP_DATA as _;

        /// Indicates that this COMM record was generated due to an `exec(2)`
        /// syscall.
        ///
        /// Note that [`MMAP_DATA`](Self::MMAP_DATA),
        /// [`COMM_EXEC`](Self::COMM_EXEC), and
        /// [`SWITCH_OUT`](Self::SWITCH_OUT) are all the same bit.
        const COMM_EXEC = bindings::PERF_RECORD_MISC_COMM_EXEC as _;

        /// Indicates that this `SWITCH_CPU_WIDE` record corresponds to a
        /// switch away from the current process.
        ///
        /// Note that [`MMAP_DATA`](Self::MMAP_DATA),
        /// [`COMM_EXEC`](Self::COMM_EXEC), and
        /// [`SWITCH_OUT`](Self::SWITCH_OUT) are all the same bit.
        const SWITCH_OUT = bindings::PERF_RECORD_MISC_SWITCH_OUT as _;

        /// Indicates that the [`ip`] field in [`Sample`] contains the exact
        /// instruction pointer at which the event was generated.
        ///
        /// [`ip`]: Sample::ip
        const EXACT_IP = bindings::PERF_RECORD_MISC_EXACT_IP as _;

        /// Indicates that the context switch that caused this `SWITCH` or
        /// `SWITCH_CPU_WIDE` record to be generated was a preemption.
        const SWITCH_OUT_PREEMPT = bindings::PERF_RECORD_MISC_SWITCH_OUT_PREEMPT as _;

        /// Indicates that this `MMAP2` record contains build-id data instead
        /// of the inode and device major and minor numbers.
        const MMAP_BUILD_ID = bindings::PERF_RECORD_MISC_MMAP_BUILD_ID as _;
    }
}

impl MiscFlags {
    /// Read the [`CpuMode`] stored within the first 3 bits.
    pub fn cpumode(&self) -> CpuMode {
        CpuMode::new((*self & Self::CPUMODE_MASK).bits() as _)
    }
}

c_enum! {
    /// The mode the CPU was running in at the time the sample was taken.
    pub struct CpuMode : u8 {
        const UNKNOWN = bindings::PERF_RECORD_MISC_CPUMODE_UNKNOWN as _;
        const KERNEL = bindings::PERF_RECORD_MISC_KERNEL as _;
        const USER = bindings::PERF_RECORD_MISC_USER as _;
        const HYPERVISOR = bindings::PERF_RECORD_MISC_HYPERVISOR as _;
        const GUEST_KERNEL = bindings::PERF_RECORD_MISC_GUEST_KERNEL as _;
        const GUEST_USER = bindings::PERF_RECORD_MISC_GUEST_USER as _;
    }
}

#[cfg(feature = "fuzzing")]
mod fuzzing {
    use super::*;

    use arbitrary::{Arbitrary, Result, Unstructured};

    impl<'a> Arbitrary<'a> for SampleFlags {
        fn arbitrary(u: &mut Unstructured) -> Result<Self> {
            Ok(Self::from_bits_retain(Arbitrary::arbitrary(u)?))
        }
    }

    impl<'a> Arbitrary<'a> for ReadFormat {
        fn arbitrary(u: &mut Unstructured) -> Result<Self> {
            Ok(Self::from_bits_retain(Arbitrary::arbitrary(u)?))
        }
    }
}
