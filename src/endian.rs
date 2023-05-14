//! Traits and types for converting between a source endianness and that of the
//! current host.
//!
//! Usually you will want [`Native`] endian if parsing data emitted by the
//! kernel otherwise you will likely want [`Dynamic`] endian if parsing data
//! from a file, although [`Little`] or [`Big`] endian may also work.

/// A trait containing the required conversion functions needed to parse perf
/// records.
///
/// # Safety
/// If [`is_native`](Endian::is_native) returns true when the source endianness
/// does not match the current host endianness then UB will occur.
pub unsafe trait Endian: Copy + Clone {
    /// Convert a `u16` from the source endian to the native endian.
    fn convert_u16(&self, bytes: [u8; 2]) -> u16;

    /// Convert a `u32` from the source endian to the native endian.
    fn convert_u32(&self, bytes: [u8; 4]) -> u32;

    /// Convert a `u64` from the source endian to the native endian.
    fn convert_u64(&self, bytes: [u8; 8]) -> u64;

    /// Whether the source endian is the same as the current native endian.
    ///
    /// If this returns `true` then the parser will attempt to avoid copying
    /// some data out of the source buffer, instead reinterpreting it when
    /// possible.
    fn is_native(&self) -> bool {
        false
    }
}

/// Native endian.
///
/// This type performs no endianness conversion.
#[derive(Copy, Clone, Debug, Default)]
pub struct Native;

unsafe impl Endian for Native {
    #[inline]
    fn convert_u16(&self, bytes: [u8; 2]) -> u16 {
        u16::from_ne_bytes(bytes)
    }

    #[inline]
    fn convert_u32(&self, bytes: [u8; 4]) -> u32 {
        u32::from_ne_bytes(bytes)
    }

    #[inline]
    fn convert_u64(&self, bytes: [u8; 8]) -> u64 {
        u64::from_ne_bytes(bytes)
    }

    #[inline]
    fn is_native(&self) -> bool {
        true
    }
}

/// Little endian.
#[derive(Copy, Clone, Debug, Default)]
pub struct Little;

unsafe impl Endian for Little {
    #[inline]
    fn convert_u16(&self, bytes: [u8; 2]) -> u16 {
        u16::from_le_bytes(bytes)
    }

    #[inline]
    fn convert_u32(&self, bytes: [u8; 4]) -> u32 {
        u32::from_le_bytes(bytes)
    }

    #[inline]
    fn convert_u64(&self, bytes: [u8; 8]) -> u64 {
        u64::from_le_bytes(bytes)
    }

    #[inline]
    fn is_native(&self) -> bool {
        let bytes = [b'a', b'b'];

        u16::from_le_bytes(bytes) == u16::from_ne_bytes(bytes)
    }
}

/// Big endian.
#[derive(Copy, Clone, Debug, Default)]
pub struct Big;

unsafe impl Endian for Big {
    #[inline]
    fn convert_u16(&self, bytes: [u8; 2]) -> u16 {
        u16::from_be_bytes(bytes)
    }

    #[inline]
    fn convert_u32(&self, bytes: [u8; 4]) -> u32 {
        u32::from_be_bytes(bytes)
    }

    #[inline]
    fn convert_u64(&self, bytes: [u8; 8]) -> u64 {
        u64::from_be_bytes(bytes)
    }

    #[inline]
    fn is_native(&self) -> bool {
        let bytes = [b'a', b'b'];

        u16::from_be_bytes(bytes) == u16::from_ne_bytes(bytes)
    }
}

/// Either big or little endian, chosen at runtime.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Dynamic {
    /// Big endian.
    Big,

    /// Little endian.
    Little,
}

unsafe impl Endian for Dynamic {
    fn convert_u16(&self, bytes: [u8; 2]) -> u16 {
        match self {
            Self::Big => Big.convert_u16(bytes),
            Self::Little => Little.convert_u16(bytes),
        }
    }

    fn convert_u32(&self, bytes: [u8; 4]) -> u32 {
        match self {
            Self::Big => Big.convert_u32(bytes),
            Self::Little => Little.convert_u32(bytes),
        }
    }

    fn convert_u64(&self, bytes: [u8; 8]) -> u64 {
        match self {
            Self::Big => Big.convert_u64(bytes),
            Self::Little => Little.convert_u64(bytes),
        }
    }

    fn is_native(&self) -> bool {
        match self {
            Self::Big => Big.is_native(),
            Self::Little => Little.is_native(),
        }
    }
}
