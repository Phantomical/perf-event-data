#![no_main]

use libfuzzer_sys::fuzz_target;
use perf_event_data::endian::Native;
use perf_event_data::parse::{ParseConfig, Parser};
use perf_event_data::Visitor;
use perf_event_open_sys::bindings::perf_event_attr;

fuzz_target!(|data: &[u8]| {
    let attr_len = std::mem::size_of::<perf_event_attr>();
    if data.len() < attr_len {
        return;
    }

    let (head, rest) = data.split_at(attr_len);
    let mut attr = perf_event_attr::default();
    unsafe {
        std::ptr::copy_nonoverlapping(head.as_ptr(), &mut attr as *mut _ as *mut u8, attr_len);
    }

    let config = ParseConfig::<Native>::from(attr);
    let mut parser = Parser::new(rest, config);

    let _ = parser.parse_record(ParseVisitor);
});

struct ParseVisitor;

impl Visitor for ParseVisitor {
    type Output<'a> = ();

    fn visit_unimplemented<'a>(
        self,
        _metadata: perf_event_data::RecordMetadata,
    ) -> Self::Output<'a> {
        ()
    }
}
