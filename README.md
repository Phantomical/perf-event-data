# perf-event-data

<div style="text-align: center">

![ci badge]
[![crates.io badge]](https://crates.io/crates/perf-event-data)
[![docs.rs badge]](https://docs.rs/perf-event-data)

</div>

[ci badge]: https://img.shields.io/github/actions/workflow/status/phantomical/perf-event-data/dispatch.yml?branch=master&style=flat-square
[docs.rs badge]: https://img.shields.io/docsrs/perf-event-data?style=flat-square
[crates.io badge]: https://img.shields.io/crates/v/perf-event-data?style=flat-square

Parse data emitted by [`perf_event_open`] into usable rust structs.

## Getting Started

- The `Record` type is an enum with every known record type.
- The `parse` module has what you need to parse bytes into known records.

Putting it all together, we get

```rust
use perf_event_data::endian::Native;
use perf_event_data::parse::{ParseConfig, Parser};
use perf_event_data::Record;

fn main() {
    let data: &[u8] = &[
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00,
        0x16, 0x4C, 0x01, 0x00, 0x17, 0x4C, 0x01, 0x00,
        0x00, 0xA0, 0x48, 0x96, 0x4F, 0x7F, 0x00, 0x00,
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xA0, 0x48, 0x96, 0x4F, 0x7F, 0x00, 0x00,
        0x2F, 0x2F, 0x61, 0x6E, 0x6F, 0x6E, 0x00, 0x00, 
    ];

    let config = ParseConfig::<Native>::default();
    let mut parser = Parser::new(data, config);
    let record: Record = parser.parse().expect("failed to parse the record");

    // ...
}
```

[`perf_event_open`]: https://man7.org/linux/man-pages/man2/perf_event_open.2.html
