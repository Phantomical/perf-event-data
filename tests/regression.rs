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

#[cfg(feature = "fuzzing")]
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

#[cfg(not(feature = "fuzzing"))]
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
#[cfg_attr(not(feature = "fuzzing"), ignore = "requires the 'fuzzing' feature")]
fn buffer_smaller_than_sample_id_len() {
    fuzz_with_config(&[
        224, 115, 115, 93, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        115, 115, 115, 115, 19, 0, 0, 0, 0, 0, 0, 0, 115, 115, 115, 115, 135, 135, 135, 135, 135,
        135, 135, 135, 115, 115, 115, 115, 135, 131, 120, 135, 255, 0, 0, 115, 115, 115, 115, 115,
        115, 115, 115, 115, 115,
    ]);
}
