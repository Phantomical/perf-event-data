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
    /// Create a `ReadValue` from a `ReadGroup` and its entry within that group.
    pub fn from_group_and_entry(group: &ReadGroup<'_>, entry: &GroupEntry) -> Self {
        Self {
            read_format: group.read_format - ReadFormat::GROUP,
            value: entry.value,
            time_enabled: group.time_enabled,
            time_running: group.time_running,
            id: entry.id,
            lost: entry.lost,
        }
    }

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

impl TryFrom<ReadGroup<'_>> for ReadValue {
    type Error = TryFromGroupError;

    fn try_from(value: ReadGroup<'_>) -> Result<Self, Self::Error> {
        let mut entries = value.entries();
        let entry = entries.next().ok_or(TryFromGroupError(()))?;

        if entries.next().is_some() {
            return Err(TryFromGroupError(()));
        }

        Ok(Self {
            read_format: value.read_format - ReadFormat::GROUP,
            value: entry.value(),
            time_enabled: value.time_enabled,
            time_running: value.time_running,
            id: entry.id,
            lost: entry.lost,
        })
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

/// The values read from a group of counters.
#[derive(Clone)]
pub struct ReadGroup<'a> {
    read_format: ReadFormat,
    time_enabled: u64,
    time_running: u64,
    data: Cow<'a, [u64]>,
}

impl<'a> ReadGroup<'a> {
    /// The number of counters contained within this group.
    pub fn len(&self) -> usize {
        self.entries().count()
    }

    /// Whether this group has any counters at all.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert all the borrowed data in this `ReadGroup` into owned data.
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

    /// Get a group entry by its index.
    pub fn get(&self, index: usize) -> Option<GroupEntry> {
        self.entries().nth(index)
    }

    /// Get a group entry by its counter id.
    pub fn get_by_id(&self, id: u64) -> Option<GroupEntry> {
        if !self.read_format.contains(ReadFormat::ID) {
            return None;
        }

        self.entries().find(|entry| entry.id() == Some(id))
    }

    /// Iterate over the entries contained within this `GroupRead`.
    pub fn entries(&self) -> GroupIter {
        GroupIter::new(self)
    }
}

impl<'a> From<ReadValue> for ReadGroup<'a> {
    fn from(value: ReadValue) -> Self {
        let mut data = Vec::with_capacity(3);
        data.push(value.value());
        data.extend(value.id());
        data.extend(value.lost());

        Self {
            read_format: value.read_format | ReadFormat::GROUP,
            time_enabled: value.time_enabled,
            time_running: value.time_running,
            data: Cow::Owned(data),
        }
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

/// The values read from a single perf event counter.
///
/// This will always include the counter value. The other fields are optional
/// depending on how the counter's `read_format` was configured.
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
                .expect("slice was not the correct size for the configured read_format")
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
/// See [`ReadGroup::entries`].
#[derive(Clone)]
pub struct GroupIter<'a> {
    iter: std::slice::ChunksExact<'a, u64>,
    read_format: ReadFormat,
}

impl<'a> GroupIter<'a> {
    fn new(group: &'a ReadGroup) -> Self {
        let read_format = group.read_format;

        Self {
            iter: group.data.chunks_exact(read_format.element_len()),
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

impl<'a> ExactSizeIterator for GroupIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

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

/// Error when attempting to convert [`ReadGroup`] to a [`ReadValue`].
#[derive(Clone, Debug)]
pub struct TryFromGroupError(());

impl fmt::Display for TryFromGroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("can only convert groups with a single element to ReadValues")
    }
}

impl std::error::Error for TryFromGroupError {}
