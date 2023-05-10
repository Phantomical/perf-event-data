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
