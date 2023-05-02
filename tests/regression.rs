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

#[test]
fn zero_header_size() {
    let data: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0];
    let config = ParseConfig::<Little>::default();
    let mut parser = Parser::new(data, config);
    let _ = parser.parse_record(ParseVisitor);
}

#[test]
fn overlarge_header_size() {
    let data: &[u8] = &[9, 0, 0, 0, 0, 251, 85, 182, 246];
    let config = ParseConfig::<Little>::default();
    let mut parser = Parser::new(data, config);
    let _ = parser.parse_record(ParseVisitor);
}
