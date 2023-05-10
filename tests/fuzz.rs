use perf_event_data::endian::Native;
use perf_event_data::parse::{ParseConfig, Parser};
use perf_event_data::Visitor;
use perf_event_open_sys::bindings::perf_event_attr;

fn fuzz_test(data: &[u8]) {
    struct ParseVisitor;

    impl Visitor<'_> for ParseVisitor {
        type Output = ();

        fn visit_unimplemented<'a>(self, _: perf_event_data::RecordMetadata) {}
    }

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
}

#[test]
fn fuzz_test_1() {
    let data = [
        224, 115, 115, 93, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
        115, 115, 115, 115, 115, 59, 115, 115, 115, 115, 115, 115, 115, 115, 130, 115, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 197, 197, 197, 197, 197, 197, 255, 255, 255, 255, 38, 255, 255, 255, 255,
        224, 115, 115, 93, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
        115, 115, 255, 255, 255, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
        115, 115, 115, 115, 255, 255, 255, 255, 255, 255, 255, 255, 115, 115, 115,
    ];
    fuzz_test(&data);
}
