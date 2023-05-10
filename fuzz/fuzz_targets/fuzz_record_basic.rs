#![no_main]

use libfuzzer_sys::fuzz_target;
use perf_event_data::endian::Little;
use perf_event_data::parse::{ParseConfig, Parser};
use perf_event_data::Visitor;

fuzz_target!(|data: &[u8]| {
    let config = ParseConfig::<Little>::default();
    let mut parser = Parser::new(data, config);
    let _ = parser.parse_record(ParseVisitor);
});

struct ParseVisitor;

impl Visitor<'_> for ParseVisitor {
    type Output = ();

    fn visit_unimplemented(self, _: perf_event_data::RecordMetadata) {}
}
