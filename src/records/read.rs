#![allow(missing_docs)]

use crate::error::ParseError;
use crate::prelude::*;
use std::borrow::Cow;
use std::fmt;
use std::iter::FusedIterator;

/// READ events happen when the kernel records the counters on its own.
///
/// This only happens when `inherit_stat` is enabled.
///
/// This struct corresponds to `PERF_RECORD_READ`. See the [manpage] for more
/// documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Read {
    /// The process ID.
    pub pid: u32,

    /// The thread ID.
    pub tid: u32,

    /// The value read from the counter during task switch.
    pub values: ReadValue,
}

#[derive(Clone, Debug)]
pub enum ReadData<'a> {
    /// Data for only a single counter.
    ///
    /// This is what will be generated if the [`ParseConfig`]'s `read_format`
    /// did not contain `READ_FORMAT_GROUP`.
    Single(ReadValue),

    /// Data for all counters in a group.
    Group(ReadGroup<'a>),
}

impl<'a> ReadData<'a> {
    pub fn into_owned(self) -> ReadData<'static> {
        match self {
            Self::Single(data) => ReadData::Single(data),
            Self::Group(data) => ReadData::Group(data.into_owned()),
        }
    }

    /// The duration for which this event was enabled, in nanoseconds.
    pub fn time_enabled(&self) -> Option<u64> {
        match self {
            Self::Single(data) => data.time_enabled(),
            Self::Group(data) => data.time_enabled(),
        }
    }

    /// The duration for which this event was running, in nanoseconds.
    ///
    /// This will be less than `time_enabled` if the kernel ended up having to
    /// multiplex multiple counters on the CPU.
    pub fn time_running(&self) -> Option<u64> {
        match self {
            Self::Single(data) => data.time_running(),
            Self::Group(data) => data.time_running(),
        }
    }
}

/// Data read from a counter.
#[derive(Clone)]
pub struct ReadValue {
    read_format: ReadFormat,
    value: u64,
    time_enabled: u64,
    time_running: u64,
    id: u64,
    lost: u64,
}

impl ReadValue {
    /// The value of the counter.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// The duration for which this event was enabled, in nanoseconds.
    pub fn time_enabled(&self) -> Option<u64> {
        self.read_format
            .contains(ReadFormat::TOTAL_TIME_ENABLED)
            .then_some(self.time_enabled)
    }

    /// The duration for which this event was running, in nanoseconds.
    ///
    /// This will be less than `time_enabled` if the kernel ended up having to
    /// multiplex multiple counters on the CPU.
    pub fn time_running(&self) -> Option<u64> {
        self.read_format
            .contains(ReadFormat::TOTAL_TIME_RUNNING)
            .then_some(self.time_running)
    }

    /// The kernel-assigned unique ID for the counter.
    pub fn id(&self) -> Option<u64> {
        self.read_format.contains(ReadFormat::ID).then_some(self.id)
    }

    /// The number of lost samples of this event.
    pub fn lost(&self) -> Option<u64> {
        self.read_format
            .contains(ReadFormat::LOST)
            .then_some(self.lost)
    }
}

impl fmt::Debug for ReadValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let read_format = self.read_format;
        let mut dbg = debug_if! {
            f.debug_struct("SingleRead") => {
                value => self.value,
                time_enabled if read_format.contains(ReadFormat::TOTAL_TIME_ENABLED) => self.time_enabled,
                time_running if read_format.contains(ReadFormat::TOTAL_TIME_RUNNING) => self.time_running,
                id if read_format.contains(ReadFormat::ID) => self.id,
                lost if read_format.contains(ReadFormat::LOST) => self.lost,

            }
        };

        dbg.finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct ReadGroup<'a> {
    read_format: ReadFormat,
    time_enabled: u64,
    time_running: u64,
    data: Cow<'a, [u64]>,
}

impl<'a> ReadGroup<'a> {
    pub fn len(&self) -> usize {
        self.data.len() / self.read_format.element_len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn into_owned(self) -> ReadGroup<'static> {
        ReadGroup {
            data: self.data.into_owned().into(),
            ..self
        }
    }

    /// The duration for which this event was enabled, in nanoseconds.
    pub fn time_enabled(&self) -> Option<u64> {
        self.read_format
            .contains(ReadFormat::TOTAL_TIME_ENABLED)
            .then_some(self.time_enabled)
    }

    /// The duration for which this event was running, in nanoseconds.
    ///
    /// This will be less than `time_enabled` if the kernel ended up having to
    /// multiplex multiple counters on the CPU.
    pub fn time_running(&self) -> Option<u64> {
        self.read_format
            .contains(ReadFormat::TOTAL_TIME_RUNNING)
            .then_some(self.time_running)
    }

    /// Iterate over the entries contained within this `GroupRead`.
    pub fn entries(&self) -> GroupIter {
        GroupIter::new(self)
    }
}

impl fmt::Debug for ReadGroup<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Entries<'a>(GroupIter<'a>);

        impl fmt::Debug for Entries<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.clone()).finish()
            }
        }

        let read_format = self.read_format;
        let mut dbg = debug_if! {
            f.debug_struct("GroupRead") => {
                time_enabled if read_format.contains(ReadFormat::TOTAL_TIME_ENABLED) => self.time_enabled,
                time_running if read_format.contains(ReadFormat::TOTAL_TIME_RUNNING) => self.time_running,
                entries => Entries(self.entries()),
            }
        };

        dbg.finish_non_exhaustive()
    }
}

#[derive(Copy, Clone)]
pub struct GroupEntry {
    read_format: ReadFormat,
    value: u64,
    id: u64,
    lost: u64,
}

impl GroupEntry {
    /// The value of the counter.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// The kernel-assigned unique ID for the counter.
    pub fn id(&self) -> Option<u64> {
        self.read_format.contains(ReadFormat::ID).then_some(self.id)
    }

    /// The number of lost samples of this event.
    pub fn lost(&self) -> Option<u64> {
        self.read_format
            .contains(ReadFormat::LOST)
            .then_some(self.lost)
    }

    fn new(config: ReadFormat, slice: &[u64]) -> Self {
        let mut iter = slice.iter().copied();
        let mut read = || {
            iter.next()
                .expect("slice was not the corred size for the configured read_format")
        };

        Self {
            read_format: config,
            value: read(),
            id: config.contains(ReadFormat::ID).then(&mut read).unwrap_or(0),
            lost: config
                .contains(ReadFormat::LOST)
                .then(&mut read)
                .unwrap_or(0),
        }
    }
}

impl fmt::Debug for GroupEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let read_format = self.read_format;

        let mut dbg = debug_if! {
            f.debug_struct("GroupEntry") => {
                value => self.value(),
                id if read_format.contains(ReadFormat::ID) => self.id,
                lost if read_format.contains(ReadFormat::LOST) => self.lost,
            }
        };

        dbg.finish_non_exhaustive()
    }
}

/// Iterator over the entries of a group.
///
/// See [`GroupRead::entries`].
#[derive(Clone)]
pub struct GroupIter<'a> {
    iter: std::slice::Chunks<'a, u64>,
    read_format: ReadFormat,
}

impl<'a> GroupIter<'a> {
    fn new(group: &'a ReadGroup) -> Self {
        let read_format = group.read_format;

        Self {
            iter: group.data.chunks(read_format.element_len()),
            read_format,
        }
    }
}

impl<'a> Iterator for GroupIter<'a> {
    type Item = GroupEntry;

    fn next(&mut self) -> Option<Self::Item> {
        Some(GroupEntry::new(self.read_format, self.iter.next()?))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(GroupEntry::new(self.read_format, self.iter.nth(n)?))
    }

    fn last(self) -> Option<Self::Item> {
        Some(GroupEntry::new(self.read_format, self.iter.last()?))
    }
}

impl<'a> DoubleEndedIterator for GroupIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(GroupEntry::new(self.read_format, self.iter.next_back()?))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(GroupEntry::new(self.read_format, self.iter.nth_back(n)?))
    }
}

impl<'a> ExactSizeIterator for GroupIter<'a> {}

impl<'a> FusedIterator for GroupIter<'a> {}

impl<'p> Parse<'p> for ReadValue {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        let read_format = p.config().read_format();

        if read_format.contains(ReadFormat::GROUP) {
            return Err(ParseError::custom(
                ErrorKind::UnsupportedConfig,
                "attempted to parse a SingleRead with a config that has GROUP set in read_format",
            ));
        }

        if !(read_format - ReadFormat::all()).is_empty() {
            return Err(ParseError::custom(
                ErrorKind::UnsupportedConfig,
                "read_format contains unsupported flags",
            ));
        }

        Ok(Self {
            read_format,
            value: p.parse()?,
            time_enabled: p
                .parse_if(read_format.contains(ReadFormat::TOTAL_TIME_ENABLED))?
                .unwrap_or(0),
            time_running: p
                .parse_if(read_format.contains(ReadFormat::TOTAL_TIME_RUNNING))?
                .unwrap_or(0),
            id: p
                .parse_if(read_format.contains(ReadFormat::ID))?
                .unwrap_or(0),
            lost: p
                .parse_if(read_format.contains(ReadFormat::LOST))?
                .unwrap_or(0),
        })
    }
}

impl<'p> Parse<'p> for ReadGroup<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        let read_format = p.config().read_format();

        if !read_format.contains(ReadFormat::GROUP) {
            return Err(ParseError::custom(
                ErrorKind::UnsupportedConfig,
                "attempted to parse a GroupRead with a config that does not have GROUP set in read_format"
            ));
        }

        if !(read_format - ReadFormat::all()).is_empty() {
            return Err(ParseError::custom(
                ErrorKind::UnsupportedConfig,
                "read_format contains unsupported flags",
            ));
        }

        let nr = p.parse_u64()? as usize;
        let time_enabled = p
            .parse_if(read_format.contains(ReadFormat::TOTAL_TIME_ENABLED))?
            .unwrap_or(0);
        let time_running = p
            .parse_if(read_format.contains(ReadFormat::TOTAL_TIME_RUNNING))?
            .unwrap_or(0);

        let element_len = read_format.element_len();
        let data_len = nr //
            .checked_mul(element_len)
            .ok_or_else(|| {
                ParseError::custom(
                    ErrorKind::InvalidRecord,
                    "number of elements in group read was too large for data type",
                )
            })?;
        let data = unsafe { p.parse_slice(data_len)? };

        Ok(Self {
            read_format,
            time_enabled,
            time_running,
            data,
        })
    }
}

impl<'p> Parse<'p> for ReadData<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        let read_format = p.config().read_format();

        if read_format.contains(ReadFormat::GROUP) {
            Ok(Self::Group(p.parse()?))
        } else {
            Ok(Self::Single(p.parse()?))
        }
    }
}

impl<'p> Parse<'p> for Read {
    fn parse<B, E>(p: &mut Parser<B, E>) -> ParseResult<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            pid: p.parse()?,
            tid: p.parse()?,
            values: p.parse()?,
        })
    }
}
