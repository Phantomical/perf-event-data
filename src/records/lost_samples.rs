use crate::prelude::*;

/// LOST_SAMPLES records indicate that some samples were lost while using
/// hardware sampling.
///
/// This struct corresponds to `PERF_RECORD_LOST_SAMPLES`. See the [manpage]
/// for more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct LostSamples {
    /// The number of potentially lost samples.
    pub lost: u64,
}

impl<'p> Parse<'p> for LostSamples {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self { lost: p.parse()? })
    }
}
