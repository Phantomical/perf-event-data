use crate::prelude::*;

/// AUX_OUTPUT_HW_ID events allow matching data written to the aux area with
/// an architecture-specific hadrware ID.
///
/// This is needed when combining Intel PT along with sampling multiple PEBS
/// events. See the docs within `perf_event.h` for more explanation.
///
/// This struct corresponds to `PERF_RECORD_AUX_OUTPUT_HW_ID`. At the time of
/// writing it is not yet documented in the [manpage]. However, there is
/// documentation present within [the kernel source][src].
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
/// [src]: https://sourcegraph.com/github.com/torvalds/linux@eb7081409f94a9a8608593d0fb63a1aa3d6f95d8/-/blob/tools/include/uapi/linux/perf_event.h?L1205
#[derive(Copy, Clone, Debug)]
pub struct AuxOutputHwId {
    /// An architecture-specific hardware ID.
    pub hw_id: u64,
}

impl<'p> Parse<'p> for AuxOutputHwId {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self { hw_id: p.parse()? })
    }
}
