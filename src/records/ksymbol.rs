use std::borrow::Cow;

use bitflags::bitflags;
use perf_event_open_sys::bindings;

use crate::prelude::*;

/// KSYMBOL records indicate symbols being registered or unregistered within
/// the kernel.
///
/// This struct corresponds to `PERF_RECORD_KSYMBOL`. See the [manpage] for
/// more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct KSymbol<'a> {
    pub addr: u64,
    pub len: u32,
    pub ksym_type: KSymbolType,
    pub flags: KSymbolFlags,
    pub name: Cow<'a, [u8]>,
}

impl<'a> KSymbol<'a> {
    pub fn into_owned(self) -> KSymbol<'static> {
        KSymbol {
            name: self.name.into_owned().into(),
            ..self
        }
    }
}

c_enum! {
    /// The type of the kernel symbol.
    pub struct KSymbolType : u16 {
        const UNKNOWN = bindings::PERF_RECORD_KSYMBOL_TYPE_UNKNOWN as _;
        const BPF = bindings::PERF_RECORD_KSYMBOL_TYPE_BPF as _;
        const OOL = bindings::PERF_RECORD_KSYMBOL_TYPE_OOL as _;
    }
}

bitflags! {
    /// Flags for [`KSymbol`].
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    pub struct KSymbolFlags : u16 {
        /// If set, this means that the symbol is being unregistered.
        const UNREGISTER = bindings::PERF_RECORD_KSYMBOL_FLAGS_UNREGISTER as _;
    }
}

impl<'p> Parse<'p> for KSymbolType {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self::new(p.parse()?))
    }
}

impl<'p> Parse<'p> for KSymbolFlags {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self::from_bits_retain(p.parse()?))
    }
}

impl<'p> Parse<'p> for KSymbol<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            addr: p.parse()?,
            len: p.parse()?,
            ksym_type: p.parse()?,
            flags: p.parse()?,
            name: p.parse_rest_trim_nul()?,
        })
    }
}
