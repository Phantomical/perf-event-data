use crate::prelude::*;

/// Record for a throttle or unthrottle events.
///
/// These are generated when the sampler generates too many events during a
/// given timer tick. In that case, the kernel will disable the counter for
/// the rest of the tick and instead generate a throttle/unthrottle record
/// pair indicating when throttling started and ended.
///
/// This struct is used for both `PERF_RECORD_THROTTLE` and
/// `PERF_RECORD_UNTHROTTLE`. See the [manpage] for more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Throttle {
    pub time: u64,
    pub id: u64,
    pub stream_id: u64,
}

impl<'p> Parse<'p> for Throttle {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            time: p.parse()?,
            id: p.parse()?,
            stream_id: p.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::endian::Little;

    use super::*;

    #[test]
    #[cfg_attr(not(target_endian = "little"), ignore)]
    fn test_parse() {
        #[rustfmt::skip]
        let bytes: &[u8] = &[
            0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80,
            0x90, 0xA0, 0xB0, 0xC0, 0xD0, 0xE0, 0xF0, 0x00,
            0xEF, 0xBE, 0xAD, 0xDE, 0xFE, 0xCA, 0xEF, 0xBE,
        ];

        let mut parser: Parser<_, Little> = Parser::new(bytes, ParseConfig::default());
        let throttle: Throttle = parser.parse().unwrap();

        assert_eq!(throttle.time, 0x8070605040302010);
        assert_eq!(throttle.id, 0x00F0E0D0C0B0A090);
        assert_eq!(throttle.stream_id, 0xBEEFCAFEDEADBEEF);
    }
}
