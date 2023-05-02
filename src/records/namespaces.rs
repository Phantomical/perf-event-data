use std::borrow::Cow;

use perf_event_open_sys::bindings;

use crate::prelude::*;

/// NAMESPACES records include namespace information of a process.
///
/// This struct corresponds to `PERF_RECORD_NAMESPACES`. See the [manpage] for
/// more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Namespaces<'a> {
    /// Process ID.
    pub pid: u32,

    /// Thread ID.
    pub tid: u32,

    /// Entries for various namespaces.
    ///
    /// Specific namespaces have fixed indices within this array. Accessors
    /// have been provided for some of these. See the [manpage] for the full
    /// documentation.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub namespaces: Cow<'a, [NamespaceEntry]>,
}

/// An individual namespace entry.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct NamespaceEntry {
    pub dev: u64,
    pub inode: u64,
}

impl<'a> Namespaces<'a> {
    /// Network namepsace
    pub fn network(&self) -> Option<&NamespaceEntry> {
        self.namespaces.get(bindings::NET_NS_INDEX as usize)
    }

    /// UTS namespace.
    pub fn uts(&self) -> Option<&NamespaceEntry> {
        self.namespaces.get(bindings::USER_NS_INDEX as usize)
    }

    /// IPC namespace.
    pub fn ipc(&self) -> Option<&NamespaceEntry> {
        self.namespaces.get(bindings::IPC_NS_INDEX as usize)
    }

    /// PID namespace.
    pub fn pid(&self) -> Option<&NamespaceEntry> {
        self.namespaces.get(bindings::PID_NS_INDEX as usize)
    }

    /// User namespace.
    pub fn user(&self) -> Option<&NamespaceEntry> {
        self.namespaces.get(bindings::USER_NS_INDEX as usize)
    }

    /// Cgroup namespace.
    pub fn cgroup(&self) -> Option<&NamespaceEntry> {
        self.namespaces.get(bindings::CGROUP_NS_INDEX as usize)
    }
}

impl<'p> Parse<'p> for NamespaceEntry {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            dev: p.parse()?,
            inode: p.parse()?,
        })
    }
}

impl<'p> Parse<'p> for Namespaces<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        let pid = p.parse()?;
        let tid = p.parse()?;
        let len = p.parse_u64()? as usize;
        let namespaces = unsafe { p.parse_slice(len)? };

        Ok(Self {
            pid,
            tid,
            namespaces,
        })
    }
}
