use crate::prelude::*;

/// Lost records indicate when events are dropped by the kernel.
///
/// This will happen when the sampler ring buffer fills up and there is no
/// space left for events to be inserted.
#[derive(Copy, Clone, Debug)]
pub struct Lost {
    /// The unique event ID for the samples that were lost.
    pub id: u64,

    /// The number of events that were lost.
    pub lost: u64,
}

impl<'p> Parse<'p> for Lost {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            id: p.parse()?,
            lost: p.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::endian::Little;

    use super::*;

    #[test]
    fn test_parse() {
        #[rustfmt::skip]
        let bytes: &[u8] = &[
            0x10, 0x00, 0x99, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xAF, 0x00, 0x00, 0x00, 0x7B, 0x00, 0x00
        ];

        let mut parser: Parser<_, Little> = Parser::new(bytes, ParseConfig::default());
        let lost: Lost = parser.parse().unwrap();

        assert_eq!(lost.id, 0x990010);
        assert_eq!(lost.lost, 0x7B000000AF00);
    }
}
