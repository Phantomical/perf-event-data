use perf_event_data::endian::Little;
use perf_event_data::parse::{ParseConfig, Parser};
use perf_event_data::Visitor;

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

fn fuzz_test(data: &[u8]) {
    let config = ParseConfig::<Little>::default();
    let mut parser = Parser::new(data, config);
    let _ = parser.parse_record(ParseVisitor);
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
