use std::borrow::Cow;
use std::fmt;

use bitflags::bitflags;
use perf_event_open_sys::bindings;
use perf_event_open_sys::bindings::__BindgenBitfieldUnit;
use perf_event_open_sys::bindings::perf_branch_entry;
use perf_event_open_sys::bindings::perf_mem_data_src;

use crate::parse::ParseError;
use crate::prelude::*;
use crate::ReadData;

mod sample_impl {
    use super::*;

    // We have this in its own module since it needs to be named `Sample` for the
    // `Debug` impl to look right. Plus, accessing any of the fields on this struct
    // will likely break things so better to have it in its own module so that can't
    // happen.
    option_struct! {
        pub(super) struct Sample<'a>: u32 {
            pub ip: u64,
            pub pid: u32,
            pub tid: u32,
            pub time: u64,
            pub addr: u64,
            pub id: u64,
            pub stream_id: u64,
            pub cpu: u32,
            pub period: u64,
            pub values: ReadData<'a>,
            pub callchain: Cow<'a, [u64]>,
            pub raw: Cow<'a, [u8]>,
            pub lbr_hw_index: u64,
            pub lbr: Cow<'a, [BranchEntry]>,
            pub regs_user: Registers<'a>,
            pub stack_user: Cow<'a, [u8]>,
            pub weight: u64,
            pub data_src: DataSource,
            pub transaction: Txn,
            pub regs_intr: Registers<'a>,
            pub phys_addr: u64,
            pub aux: Cow<'a, [u8]>,
            pub data_page_size: u64,
            pub code_page_size: u64
        }
    }
}

#[derive(Clone)]
pub struct Sample<'a>(sample_impl::Sample<'a>);

impl<'a> Sample<'a> {
    pub fn id(&self) -> Option<u64> {
        self.0.id().copied()
    }

    pub fn ip(&self) -> Option<u64> {
        self.0.ip().copied()
    }

    pub fn pid(&self) -> Option<u32> {
        self.0.pid().copied()
    }

    pub fn tid(&self) -> Option<u32> {
        self.0.tid().copied()
    }

    pub fn time(&self) -> Option<u64> {
        self.0.time().copied()
    }

    pub fn addr(&self) -> Option<u64> {
        self.0.addr().copied()
    }

    pub fn stream_id(&self) -> Option<u64> {
        self.0.stream_id().copied()
    }

    pub fn cpu(&self) -> Option<u32> {
        self.0.cpu().copied()
    }

    pub fn period(&self) -> Option<u64> {
        self.0.period().copied()
    }

    pub fn values(&self) -> Option<&ReadData<'a>> {
        self.0.values()
    }

    pub fn callchain(&self) -> Option<&[u64]> {
        self.0.callchain().map(|cow| &**cow)
    }

    pub fn raw(&self) -> Option<&[u8]> {
        self.0.raw().map(|cow| &**cow)
    }

    pub fn lbr_hw_index(&self) -> Option<u64> {
        self.0.lbr_hw_index().copied()
    }

    pub fn lbr(&self) -> Option<&[BranchEntry]> {
        self.0.lbr().map(|cow| &**cow)
    }

    pub fn regs_user(&self) -> Option<&Registers<'a>> {
        self.0.regs_user()
    }

    pub fn stack_user(&self) -> Option<&[u8]> {
        self.0.stack_user().map(|cow| &**cow)
    }

    pub fn weight(&self) -> Option<u64> {
        self.0.weight().copied()
    }

    pub fn data_src(&self) -> Option<DataSource> {
        self.0.data_src().copied()
    }

    pub fn transaction(&self) -> Option<Txn> {
        self.0.transaction().copied()
    }

    pub fn regs_intr(&self) -> Option<&Registers<'a>> {
        self.0.regs_intr()
    }

    pub fn phys_addr(&self) -> Option<u64> {
        self.0.phys_addr().copied()
    }

    pub fn aux(&self) -> Option<&[u8]> {
        self.0.aux().map(|cow| &**cow)
    }

    pub fn data_page_size(&self) -> Option<u64> {
        self.0.data_page_size().copied()
    }

    pub fn code_page_size(&self) -> Option<u64> {
        self.0.code_page_size().copied()
    }
}

impl<'p> Parse<'p> for Sample<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        let config = p.config();
        let sty = config.sample_type();
        let branch_hw_index = config.branch_hw_index();
        let regs_user = config.regs_user();
        let regs_intr = config.regs_intr();

        let id = p.parse_if(sty.contains(SampleFlags::IDENTIFIER))?;
        let ip = p.parse_if(sty.contains(SampleFlags::IP))?;
        let pid = p.parse_if(sty.contains(SampleFlags::TID))?;
        let tid = p.parse_if(sty.contains(SampleFlags::TID))?;
        let time = p.parse_if(sty.contains(SampleFlags::TIME))?;
        let addr = p.parse_if(sty.contains(SampleFlags::ADDR))?;
        let id = p.parse_if(sty.contains(SampleFlags::ID))?.or(id);
        let stream_id = p.parse_if(sty.contains(SampleFlags::STREAM_ID))?;
        let cpu = p.parse_if_with(sty.contains(SampleFlags::CPU), |p| {
            Ok((p.parse_u32()?, p.parse_u32()?).0)
        })?;
        let period = p.parse_if(sty.contains(SampleFlags::PERIOD))?;
        let values = p.parse_if(sty.contains(SampleFlags::READ))?;
        let callchain = p.parse_if_with(sty.contains(SampleFlags::CALLCHAIN), |p| {
            let nr = p.parse_u64()? as _;
            unsafe { p.parse_slice(nr) }
        })?;
        let raw = p.parse_if_with(sty.contains(SampleFlags::RAW), |p| {
            let size = p.parse_u64()? as _;
            p.parse_bytes(size)
        })?;
        let (lbr, lbr_hw_index) = p
            .parse_if_with(sty.contains(SampleFlags::BRANCH_STACK), |p| {
                let nr = p.parse_u64()? as usize;
                let hw_index = p.parse_if(branch_hw_index)?;
                let lbr = unsafe { p.parse_slice(nr)? };

                Ok((lbr, hw_index))
            })?
            .unzip();
        let lbr_hw_index = lbr_hw_index.flatten();
        let regs_user = p.parse_if_with(sty.contains(SampleFlags::REGS_USER), |p| {
            Registers::parse(p, regs_user)
        })?;
        let stack_user = p.parse_if_with(sty.contains(SampleFlags::STACK_USER), |p| {
            let size = p.parse_u64()? as usize;
            let mut data = p.parse_bytes(size)?;
            let dyn_size = p.parse_u64()? as usize;

            if dyn_size > data.len() {
                return Err(ParseError::custom(
                    ErrorKind::InvalidRecord,
                    "stack dyn_size was greater than the record size",
                ));
            }

            match &mut data {
                Cow::Owned(data) => data.truncate(dyn_size),
                Cow::Borrowed(data) => *data = &data[..dyn_size],
            }

            Ok(data)
        })?;
        let weight = p.parse_if(sty.contains(SampleFlags::WEIGHT))?;
        let data_src = p.parse_if(sty.contains(SampleFlags::DATA_SRC))?;
        let transaction = p.parse_if(sty.contains(SampleFlags::TRANSACTION))?;
        let regs_intr = p.parse_if_with(sty.contains(SampleFlags::REGS_INTR), |p| {
            Registers::parse(p, regs_intr)
        })?;
        let phys_addr = p.parse_if(sty.contains(SampleFlags::PHYS_ADDR))?;
        let aux = p.parse_if_with(sty.contains(SampleFlags::AUX), |p| {
            let size = p.parse_u64()? as usize;
            p.parse_bytes(size)
        })?;
        let data_page_size = p.parse_if(sty.contains(SampleFlags::DATA_PAGE_SIZE))?;
        let code_page_size = p.parse_if(sty.contains(SampleFlags::CODE_PAGE_SIZE))?;

        Ok(Self(sample_impl::Sample::new(
            ip,
            pid,
            tid,
            time,
            addr,
            id,
            stream_id,
            cpu,
            period,
            values,
            callchain,
            raw,
            lbr_hw_index,
            lbr,
            regs_user,
            stack_user,
            weight,
            data_src,
            transaction,
            regs_intr,
            phys_addr,
            aux,
            data_page_size,
            code_page_size,
        )))
    }
}

impl fmt::Debug for Sample<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Describes the captured subset of registers when a sample was taken.
///
/// See the [manpage] for all the details.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Registers<'a> {
    /// The ABI of the program from which the sample was taken.
    pub abi: SampleRegsAbi,

    /// A bitmask indicating which registers were recorded.
    ///
    /// This is configured as a part of constructing the sampler.
    pub mask: u64,

    /// The recorded values of the registers.
    pub regs: Cow<'a, [u64]>,
}

c_enum! {
    /// ABI of the program when sampling registers.
    pub struct SampleRegsAbi : u64 {
        const NONE = bindings::PERF_SAMPLE_REGS_ABI_NONE as _;
        const ABI_32 = bindings::PERF_SAMPLE_REGS_ABI_32 as _;
        const ABI_64 = bindings::PERF_SAMPLE_REGS_ABI_64 as _;
    }
}

impl<'p> Registers<'p> {
    fn parse<B, E>(p: &mut Parser<B, E>, mask: u64) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self {
            abi: p.parse()?,
            mask,
            regs: unsafe { p.parse_slice(mask.count_ones() as _)? },
        })
    }
}

impl<'p> Parse<'p> for SampleRegsAbi {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self::new(p.parse()?))
    }
}

c_enum! {
    /// Branch type as used by the last branch record.
    ///
    /// This is a field present within [`BranchEntry`]. It is not documented in the
    /// [manpage] but is present within the perf_event headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub struct BranchType : u8 {
        const UNKNOWN = bindings::PERF_BR_UNKNOWN as _;
        const COND = bindings::PERF_BR_COND as _;
        const UNCOND = bindings::PERF_BR_UNCOND as _;
        const IND = bindings::PERF_BR_IND as _;
        const CALL = bindings::PERF_BR_CALL as _;
        const IND_CALL = bindings::PERF_BR_IND_CALL as _;
        const RET = bindings::PERF_BR_RET as _;
        const SYSCALL = bindings::PERF_BR_SYSCALL as _;
        const COND_CALL = bindings::PERF_BR_COND_CALL as _;
        const COND_RET = bindings::PERF_BR_COND_RET as _;
    }

}

/// Record of a branch taken by the hardware.
#[derive(Copy, Clone, Debug)]
pub struct BranchEntry(perf_branch_entry);

impl BranchEntry {
    /// Address of the source instruction.
    ///
    /// This may not always be a branch instruction.
    pub fn from(&self) -> u64 {
        self.0.from
    }

    /// Address of the branch target.
    pub fn to(&self) -> u64 {
        self.0.to
    }

    /// Whether the branch was mispredicted.
    pub fn mispred(&self) -> bool {
        self.0.mispred() != 0
    }

    /// Whether the branch was predicted correctly.
    pub fn predicted(&self) -> bool {
        self.0.predicted() != 0
    }

    /// Whether the branch occurred within a transaction.
    pub fn in_tx(&self) -> bool {
        self.0.in_tx() != 0
    }

    /// Whether the branch was due to a transaction abort.
    pub fn abort(&self) -> bool {
        self.0.abort() != 0
    }

    /// The cycle count since the last branch.
    pub fn cycles(&self) -> u16 {
        self.0.cycles() as _
    }

    /// Branch type.
    ///
    /// This field is not documented within the manpage but is present within
    /// the perf_event headers.
    pub fn ty(&self) -> BranchType {
        BranchType(self.0.type_() as _)
    }
}

impl<'p> Parse<'p> for BranchEntry {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self(perf_branch_entry {
            from: p.parse()?,
            to: p.parse()?,
            _bitfield_align_1: [],
            _bitfield_1: __BindgenBitfieldUnit::new(u64::to_ne_bytes(p.parse()?)),
        }))
    }
}

/// Describes where in the memory hierarchy the sampled instruction came from.
///
/// See the [manpage] for a full description.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Copy, Clone, Default)]
pub struct DataSource(perf_mem_data_src);

impl DataSource {
    fn bitfield(&self) -> &bindings::perf_mem_data_src__bindgen_ty_1 {
        unsafe { &self.0.__bindgen_anon_1 }
    }

    /// Type of opcode.
    pub fn mem_op(&self) -> MemOp {
        MemOp::from_bits_retain(self.bitfield().mem_op())
    }

    /// Memory hierarchy level hit or miss.
    pub fn mem_lvl(&self) -> MemLevel {
        MemLevel::from_bits_retain(self.bitfield().mem_lvl())
    }

    /// Snoop mode.
    ///
    /// This is a combination of the flags from both the `mem_snoop` and the
    /// `mem_snoopx` fields in the kernel source.
    pub fn mem_snoop(&self) -> MemSnoop {
        MemSnoop::new(self.bitfield().mem_snoop(), self.bitfield().mem_snoopx())
    }

    /// Lock instruction.
    pub fn mem_lock(&self) -> MemLock {
        MemLock::from_bits_retain(self.bitfield().mem_lock())
    }

    /// TLB access hit or miss.
    pub fn mem_dtlb(&self) -> MemDtlb {
        MemDtlb::from_bits_retain(self.bitfield().mem_dtlb())
    }

    /// Memory hierarchy level number.
    ///
    /// This field is not documented in the [manpage] but is present within the
    /// kernel headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub fn mem_lvl_num(&self) -> MemLevelNum {
        MemLevelNum(self.bitfield().mem_lvl_num() as _)
    }

    /// Whether the memory access was remote.
    ///
    /// This field is not documented in the [manpage] but is present within the
    /// kernel headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub fn mem_remote(&self) -> bool {
        self.bitfield().mem_remote() != 0
    }

    /// Access was blocked.
    ///
    /// This field is not documented in the [manpage] but is present within the
    /// kernel headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub fn mem_blk(&self) -> MemBlk {
        MemBlk::from_bits_retain(self.bitfield().mem_blk())
    }

    pub fn mem_hops(&self) -> u8 {
        self.bitfield().mem_hops() as _
    }
}

impl fmt::Debug for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataSource")
            .field("mem_op", &self.mem_op())
            .field("mem_lvl", &self.mem_lvl())
            .field("mem_snoop", &self.mem_snoop())
            .field("mem_lock", &self.mem_lock())
            .field("mem_dtlb", &self.mem_dtlb())
            .field("mem_lvl_num", &self.mem_lvl_num())
            .field("mem_remote", &self.mem_remote())
            .field("mem_blk", &self.mem_blk())
            .field("mem_hops", &self.mem_hops())
            .finish()
    }
}

impl<'p> Parse<'p> for DataSource {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self(perf_mem_data_src { val: p.parse()? }))
    }
}

bitflags! {
    /// Memory operation.
    ///
    /// This is used by [`DataSource`].
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
    pub struct MemOp : u64 {
        const NA = bindings::PERF_MEM_OP_NA as _;
        const LOAD = bindings::PERF_MEM_OP_LOAD as _;
        const STORE = bindings::PERF_MEM_OP_STORE as _;
        const PFETCH = bindings::PERF_MEM_OP_PFETCH as _;
        const EXEC = bindings::PERF_MEM_OP_EXEC as _;
    }

    /// Location in the memory hierarchy.
    ///
    /// This is used by [`DataSource`].
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
    pub struct MemLevel : u64 {
        const NA = bindings::PERF_MEM_LVL_NA as _;
        const HIT = bindings::PERF_MEM_LVL_HIT as _;
        const MISS = bindings::PERF_MEM_LVL_MISS as _;
        const L1 = bindings::PERF_MEM_LVL_L1 as _;
        const LFB = bindings::PERF_MEM_LVL_LFB as _;
        const L2 = bindings::PERF_MEM_LVL_L2 as _;
        const L3 = bindings::PERF_MEM_LVL_L3 as _;
        const LOC_RAM = bindings::PERF_MEM_LVL_LOC_RAM as _;
        const REM_RAM1 = bindings::PERF_MEM_LVL_REM_RAM1 as _;
        const REM_RAM2 = bindings::PERF_MEM_LVL_REM_RAM2 as _;
        const REM_CCE1 = bindings::PERF_MEM_LVL_REM_CCE1 as _;
        const REM_CCE2 = bindings::PERF_MEM_LVL_REM_CCE2 as _;
        const IO = bindings::PERF_MEM_LVL_IO as _;
        const UNC = bindings::PERF_MEM_LVL_UNC as _;
    }

    /// Whether the instruction was a locked instruction.
    ///
    /// This is used by [`DataSource`].
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
    pub struct MemLock : u64 {
        const NA = bindings::PERF_MEM_LOCK_NA as _;
        const LOCKED = bindings::PERF_MEM_LOCK_LOCKED as _;
    }

    /// Memory TLB access.
    ///
    /// This is used by [`DataSource`].
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
    pub struct MemDtlb : u64 {
        const NA = bindings::PERF_MEM_TLB_NA as _;
        const HIT = bindings::PERF_MEM_TLB_HIT as _;
        const MISS = bindings::PERF_MEM_TLB_MISS as _;
        const L1 = bindings::PERF_MEM_TLB_L1 as _;
        const L2 = bindings::PERF_MEM_TLB_L2 as _;
        const WK = bindings::PERF_MEM_TLB_WK as _;
        const OS = bindings::PERF_MEM_TLB_OS as _;
    }

    /// Extended bits for [`MemSnoop`].
    ///
    /// This is used by [`DataSource`].
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
    pub struct MemSnoopX : u64 {
        const FWD = bindings::PERF_MEM_SNOOPX_FWD as _;
        const PEER = bindings::PERF_MEM_SNOOPX_PEER as _;
    }

    /// Access was blocked.
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
    pub struct MemBlk : u64 {
        const NA = bindings::PERF_MEM_BLK_NA as _;
        const DATA = bindings::PERF_MEM_BLK_DATA as _;
        const ADDR = bindings::PERF_MEM_BLK_ADDR as _;
    }
}

bitflags! {
    /// Memory snoop mode.
    ///
    /// This is used by [`DataSource`].
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
    pub struct MemSnoop : u64 {
        const NA = bindings::PERF_MEM_SNOOP_NA as _;
        const NONE = bindings::PERF_MEM_SNOOP_NONE as _;
        const HIT = bindings::PERF_MEM_SNOOP_HIT as _;
        const MISS = bindings::PERF_MEM_SNOOP_MISS as _;
        const HITM = bindings::PERF_MEM_SNOOP_HITM as _;

        const FWD = (bindings::PERF_MEM_SNOOPX_FWD as u64) << Self::SNOOPX_SHIFT;
        const PEER = (bindings::PERF_MEM_SNOOPX_PEER as u64) << Self::SNOOPX_SHIFT;
    }
}

impl MemSnoop {
    pub fn new(mem_snoop: u64, mem_snoopx: u64) -> Self {
        Self::from_bits_truncate(
            (mem_snoop & ((1 << Self::SNOOPX_SHIFT) - 1)) | (mem_snoopx << Self::SNOOPX_SHIFT),
        )
    }

    const SNOOPX_SHIFT: u64 = 5;
}

bitflags! {
    /// Info about a transactional memory event.
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
    pub struct Txn: u64 {
        const ELISION = bindings::PERF_TXN_ELISION as _;
        const TRANSACTION = bindings::PERF_TXN_TRANSACTION as _;
        const SYNC = bindings::PERF_TXN_SYNC as _;
        const ASYNC = bindings::PERF_TXN_ASYNC as _;
        const RETRY = bindings::PERF_TXN_RETRY as _;
        const CONFLICT = bindings::PERF_TXN_CONFLICT as _;
        const CAPACITY_WRITE = bindings::PERF_TXN_CAPACITY_WRITE as _;
        const CAPACITY_READ = bindings::PERF_TXN_CAPACITY_READ as _;

        const ABORT_MASK = bindings::PERF_TXN_ABORT_MASK as _;
    }
}

impl Txn {
    /// A user-specified abort code.
    pub fn abort(&self) -> u32 {
        (self.bits() >> bindings::PERF_TXN_ABORT_SHIFT) as _
    }
}

impl<'p> Parse<'p> for Txn {
    fn parse<B, E>(p: &mut Parser<B, E>) -> Result<Self>
    where
        E: Endian,
        B: ParseBuf<'p>,
    {
        Ok(Self::from_bits_retain(p.parse()?))
    }
}

c_enum! {
    /// Memory hierarchy level number.
    ///
    /// This is a field within [`DataSource`]. It is not documented in the [manpage]
    /// but is present within the perf_event headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub struct MemLevelNum : u8 {
        const L1 = bindings::PERF_MEM_LVLNUM_L1 as _;
        const L2 = bindings::PERF_MEM_LVLNUM_L2 as _;
        const L3 = bindings::PERF_MEM_LVLNUM_L3 as _;
        const L4 = bindings::PERF_MEM_LVLNUM_L4 as _;

        const ANY_CACHE = bindings::PERF_MEM_LVLNUM_ANY_CACHE as _;
        const LFB = bindings::PERF_MEM_LVLNUM_LFB as _;
        const RAM = bindings::PERF_MEM_LVLNUM_RAM as _;
        const PMEM = bindings::PERF_MEM_LVLNUM_PMEM as _;
        const NA = bindings::PERF_MEM_LVLNUM_NA as _;
    }
}

#[cfg(test)]
mod tests {
    use crate::endian::Little;

    use super::*;

    #[test]
    fn simple_parse_sample() {
        #[rustfmt::skip]
        let data: &[u8] = &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F
        ];

        let config: ParseConfig<Little> =
            ParseConfig::default().with_sample_type(SampleFlags::ADDR | SampleFlags::ID);
        let sample: Sample = Parser::new(data, config).parse().unwrap();

        assert_eq!(sample.addr(), Some(0x0706050403020100));
        assert_eq!(sample.id(), Some(0x0F0E0D0C0B0A0908));
        assert_eq!(sample.cpu(), None);
        assert_eq!(sample.time(), None);
    }
}
