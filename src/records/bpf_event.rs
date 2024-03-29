use crate::prelude::*;
use perf_event_open_sys::bindings;

/// BPF_EVENT records indicate when a BPF program is loaded or unloaded.
///
/// This struct corresponds to `PERF_RECORD_BPF_EVENT`. See the [manpage] for
/// more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub struct BpfEvent {
    pub ty: BpfEventType,
    pub flags: u16,
    pub id: u32,
    pub tag: [u8; 8],
}

c_enum! {
    /// Indicates the type of a [`BpfEvent`]
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    pub enum BpfEventType : u16 {
        /// The event type is unknown.
        UNKNOWN = bindings::PERF_BPF_EVENT_UNKNOWN as _,

        /// A BPF program was loaded.
        PROG_LOAD = bindings::PERF_BPF_EVENT_PROG_LOAD as _,

        /// A BPF program was unloaded.
        PROG_UNLOAD = bindings::PERF_BPF_EVENT_PROG_UNLOAD as _,
    }
}

impl BpfEventType {
    /// Create a new `BpfEventType`.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }
}

impl<'p> Parse<'p> for BpfEventType {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self::new(p.parse()?))
    }
}

impl<'p> Parse<'p> for BpfEvent {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            ty: p.parse()?,
            flags: p.parse()?,
            id: p.parse()?,
            tag: p.parse()?,
        })
    }
}
