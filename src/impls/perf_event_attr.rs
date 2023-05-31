use perf_event_open_sys::bindings::*;

use crate::error::ParseError;
use crate::prelude::*;

/// Maximum supported size of perf_event_attr.
///
/// If you update this make sure to update the parsing code below so that it
/// properly handles the new version.
const PERF_ATTR_SIZE_MAX: u32 = PERF_ATTR_SIZE_VER8;

impl<'p> Parse<'p> for perf_event_attr {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        use std::mem;

        let mut attr = perf_event_attr::default();

        attr.type_ = p.parse()?;
        attr.size = p.parse()?;

        match attr.size {
            // Concrete sizes defined by the kernel headers.
            // We support all of these.
            PERF_ATTR_SIZE_VER0 | PERF_ATTR_SIZE_VER1 | PERF_ATTR_SIZE_VER2
            | PERF_ATTR_SIZE_VER3 | PERF_ATTR_SIZE_VER4 | PERF_ATTR_SIZE_VER5
            | PERF_ATTR_SIZE_VER6 | PERF_ATTR_SIZE_VER7 | PERF_ATTR_SIZE_VER8 => (),

            // We support larger sizes since they may be introduced in the future.
            size if size > PERF_ATTR_SIZE_MAX => (),

            // We do not support odd sizes that are not one of the kernel's defined constants.
            size => {
                return Err(ParseError::custom(
                    ErrorKind::InvalidRecord,
                    format_args!("{size} is not a valid size for a perf_event_attr struct"),
                ))
            }
        }

        let mut p = p.split_at(attr.size as usize - mem::size_of_val(&attr.size))?;

        if attr.size >= PERF_ATTR_SIZE_VER0 {
            attr.config = p.parse()?;
            attr.__bindgen_anon_1.sample_period = p.parse()?;
            attr.sample_type = p.parse()?;
            attr.read_format = p.parse()?;
            attr._bitfield_1 = __BindgenBitfieldUnit::new(u64::to_ne_bytes(p.parse()?));
            attr.__bindgen_anon_2.wakeup_events = p.parse()?;
            attr.bp_type = p.parse()?;
            attr.__bindgen_anon_3.config1 = p.parse()?;
        }

        if attr.size >= PERF_ATTR_SIZE_VER1 {
            attr.__bindgen_anon_4.config2 = p.parse()?;
        }

        if attr.size >= PERF_ATTR_SIZE_VER2 {
            attr.branch_sample_type = p.parse()?;
        }

        if attr.size >= PERF_ATTR_SIZE_VER3 {
            attr.sample_regs_user = p.parse()?;
            attr.sample_stack_user = p.parse()?;
            attr.clockid = p.parse_u32()? as _;
        }

        if attr.size >= PERF_ATTR_SIZE_VER4 {
            attr.sample_regs_intr = p.parse()?;
        }

        if attr.size >= PERF_ATTR_SIZE_VER5 {
            attr.aux_watermark = p.parse()?;
            attr.sample_max_stack = p.parse()?;
            let _ = p.parse_u16()?;
        }

        if attr.size >= PERF_ATTR_SIZE_VER6 {
            attr.aux_sample_size = p.parse()?;
            let _ = p.parse_u32()?;
        }

        if attr.size >= PERF_ATTR_SIZE_VER7 {
            attr.sig_data = p.parse()?;
        }

        if attr.size >= PERF_ATTR_SIZE_VER8 {
            attr.config3 = p.parse()?;
        }

        if attr.size > PERF_ATTR_SIZE_MAX {
            let rest = p.parse_rest()?;
            let all_zeros = rest.iter().copied().all(|b| b == 0);

            if !all_zeros {
                return Err(ParseError::custom(
                    ErrorKind::UnsupportedData,
                    "\
                    serialized perf_event_attr contains fields not supported \
                    by this version of perf-event-data\
                    ",
                ));
            }
        }

        Ok(attr)
    }
}
