use std::alloc::GlobalAlloc;

use perf_event_data::endian::Little;
use perf_event_data::parse::{ParseConfig, Parser};
use perf_event_data::Visitor;

struct ParseVisitor;

impl Visitor<'_> for ParseVisitor {
    type Output = ();

    fn visit_unimplemented(self, _: perf_event_data::RecordMetadata) {}
}

/// Allocator that panics if we allocate something too large.
struct LimitAlloc(std::alloc::System);

impl LimitAlloc {
    const MAX_SIZE: usize = 4 * 1024 * 1024;
}

unsafe impl GlobalAlloc for LimitAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        assert!(layout.size() < Self::MAX_SIZE);
        self.0.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        self.0.dealloc(ptr, layout);
    }
}

#[global_allocator]
static LIMIT_ALLOC: LimitAlloc = LimitAlloc(std::alloc::System);

fn fuzz_test(data: &[u8]) {
    let config = ParseConfig::<Little>::default();
    let mut parser = Parser::new(data, config);
    let _ = parser.parse_record(ParseVisitor);
}

#[cfg(feature = "arbitrary")]
fn fuzz_with_config(data: &[u8]) {
    use arbitrary::{Arbitrary, Unstructured};

    let mut data = Unstructured::new(data);
    let config = match ParseConfig::<Little>::arbitrary(&mut data) {
        Ok(config) => config,
        Err(_) => return,
    };
    let mut parser = Parser::new(data.take_rest(), config);
    let _ = parser.parse_record(ParseVisitor);
}

#[cfg(not(feature = "arbitrary"))]
fn fuzz_with_config(data: &[u8]) {
    unimplemented!()
}

#[test]
fn zero_header_size() {
    fuzz_test(&[0, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn overlarge_header_size() {
    fuzz_test(&[9, 0, 0, 0, 0, 251, 85, 182, 246]);
}

#[test]
fn enormous_slice() {
    fuzz_test(&[16, 0, 0, 0, 0, 180, 8, 69, 86, 81, 0, 180, 180, 8]);
}

#[test]
#[cfg_attr(not(feature = "arbitrary"), ignore = "requires the 'arbitrary' feature")]
fn buffer_smaller_than_sample_id_len() {
    fuzz_with_config(&[
        224, 115, 115, 93, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        115, 115, 115, 115, 19, 0, 0, 0, 0, 0, 0, 0, 115, 115, 115, 115, 135, 135, 135, 135, 135,
        135, 135, 135, 115, 115, 115, 115, 135, 131, 120, 135, 255, 0, 0, 115, 115, 115, 115, 115,
        115, 115, 115, 115, 115,
    ]);
}

#[test]
#[cfg_attr(not(feature = "arbitrary"), ignore = "requires the 'arbitrary' feature")]
fn oversize_alloc() {
    fuzz_with_config(&[
        214, 115, 91, 93, 115, 141, 140, 140, 148, 115, 115, 115, 115, 115, 115, 115, 145, 115,
        115, 255, 255, 255, 255, 255, 255, 255, 255, 1, 0, 0, 0, 255, 255, 255, 9, 0, 0, 0, 115,
        115, 115, 115, 115, 115, 115, 107, 114, 115, 115, 115, 255, 135, 135, 1, 0, 0, 0, 135, 135,
        135, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 224, 115, 115, 93, 115, 0, 0,
        115, 115, 115, 115, 115, 114, 115, 40, 115, 115, 115, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
        0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 0, 0, 0, 255, 255, 255,
        9, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 115, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 115, 255, 255, 38, 255, 255, 255, 1,
    ]);
}

#[test]
#[cfg_attr(not(feature = "arbitrary"), ignore = "requires the 'arbitrary' feature")]
fn bad_group() {
    fuzz_with_config(&[
        214, 115, 91, 93, 115, 115, 115, 115, 59, 115, 115, 115, 115, 115, 115, 115, 23, 0, 0, 0,
        0, 0, 0, 0, 0, 115, 115, 122, 115, 115, 115, 255, 135, 135, 9, 0, 0, 0, 135, 0, 0, 189, 0,
        115,
    ]);
}

#[test]
#[cfg_attr(not(feature = "arbitrary"), ignore = "requires the 'arbitrary' feature")]
fn oversize_read_group() {
    fuzz_with_config(&[
        214, 115, 91, 93, 115, 255, 255, 255, 255, 115, 115, 115, 115, 115, 145, 135, 9, 0, 0, 0,
        135, 255, 115, 135, 0, 0, 0, 115, 16, 115, 123, 255, 135, 135, 9, 0, 0, 0, 0, 0, 115, 16,
        115,
    ]);
}
