#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use perf_event_data::endian::Little;
use perf_event_data::parse::{ParseConfig, Parser};
use perf_event_data::Visitor;

fuzz_target!(|data: &[u8]| {
    let mut data = Unstructured::new(data);
    let config = match ParseConfig::<Little>::arbitrary(&mut data) {
        Ok(config) => config,
        Err(_) => return,
    };
    let mut parser = Parser::new(data.take_rest(), config);
    let _ = parser.parse_record(ParseVisitor);
});

struct ParseVisitor;

impl Visitor<'_> for ParseVisitor {
    type Output = ();

    fn visit_unimplemented(self, _: perf_event_data::RecordMetadata) {}
}
