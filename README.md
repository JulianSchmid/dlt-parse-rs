[![Crates.io](https://img.shields.io/crates/v/dlt_parse.svg)](https://crates.io/crates/dlt_parse)
[![docs.rs](https://docs.rs/dlt_parse/badge.svg)](https://docs.rs/dlt_parse)
[![build status github](https://github.com/JulianSchmid/dlt-parse-rs/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/JulianSchmid/dlt-parse-rs/actions/workflows/main.yml)
[![build status gitlab](https://gitlab.com/julian.schmid/dlt-parse-rs/badges/main/pipeline.svg)](https://gitlab.com/julian.schmid/dlt-parse-rs/-/commits/main)
[![codecov](https://codecov.io/gh/JulianSchmid/dlt-parse-rs/branch/main/graph/badge.svg?token=D1LANr6nox)](https://codecov.io/gh/JulianSchmid/dlt-parse-rs)

# dlt_parse

A zero allocation rust library for basic parsing & writing DLT (Diagnostic Log and Trace)
packets. Currently only the parsing and writing of the header is supported & parsing of verbose messages.

## Usage:

By default `serde` is disabled and `std` is enabled if you add `dlt_parse` as dependency to your `Cargo.toml`:

```toml
[dependencies]
dlt_parse = "0.9.1"
```

If you additionally want `serde` support you will have to activate the `serde` feature:

```toml
[dependencies]
dlt_parse = { version = "0.9.1", features = ["serde"] }
```

If you want to use the crate in `no_std` mode you will have to disable the default features:

```toml
[dependencies]
dlt_parse = { version = "0.9.1", default-features = false }
```

## What is dlt_parse?
dlt_parse is a library that aims to provide serialisation & deserialisation funtions for DLT (Diagnostic Log and Trace) packets.
It should make it possible to anlyse recordings of DLT packets as fast as possible, as well as writing servers
that send DLT packets to the network.

Some key points are:

* It is completly written in Rust and thoroughly tested.
* Special attention has been paid to not use allocations or syscalls.
* It is possible to use the crate in an `no-std` environment.
* The package is still in development and can & will still change.

## Example: Serializing & Slicing/Deserializing DLT Packets

In this example a non verbose DLT packet is serialized and deserialized again. Specificly the serialized packet is
converted into a DltPacketSlice. This has the advantage, that not all fields have to be deserialied to
access the payload or specific fields in the header. Note that it is also possible to completely deserialize
DLT headers with the DltHeader::read function. This can make sense, if most fields of the header are used anyways.

```rust
use self::dlt_parse::{DltHeader, DltLogLevel, DltExtendedHeader, SliceIterator};

let header = {
    let mut header = DltHeader {
        is_big_endian: true, // payload & message id are encoded with big endian
        message_counter: 0,
        length: 0,
        ecu_id: None,
        session_id: None,
        timestamp: None,
        extended_header: Some(DltExtendedHeader::new_non_verbose_log(
            DltLogLevel::Debug,
            [b'a', b'p', b'p', b'i'],// application id
            [b'c', b't', b'x', b'i'],// context id
        ))
    };
    header.length = header.header_len() + 4 + 4; // header + message id + payload

    header
};

// buffer to store serialized header & payload
let mut buffer = Vec::<u8>::with_capacity(usize::from(header.length));
buffer.extend_from_slice(&header.to_bytes());

// write payload (message id 1234 & non verbose payload)
{
    // for write_all
    use std::io::Write;

    // write the message id & payload
    buffer.write_all(&1234u32.to_be_bytes()).unwrap(); // message id
    buffer.write_all(&[5,6,7,9]); // payload
}

// packets can contain multiple dlt messages, iterate through them
for dlt_message in SliceIterator::new(&buffer) {
    match dlt_message {
        Ok(dlt_slice) => {
            //check if the message is verbose or non verbose
            if let Some(typed_payload) = dlt_slice.typed_payload() {
                use dlt_parse::DltTypedPayload::*;
                match typed_payload {
                    Verbose { info, iter } => {
                        println!("verbose message of type {:?} with values:", info.into_message_type());
                        for value in iter {
                            println!("  {:?}", value);
                        }
                    }
                    NonVerbose {
                        info,
                        msg_id,
                        payload,
                    } => {
                        println!(
                            "non verbose message 0x{:x} of type {:?} and {} bytes of payload",
                            msg_id,
                            info.map(|v| v.into_message_type()),
                            payload.len()
                        );
                    }
                }
            } else {
                println!("non verbose message with incomplete message id");
            }
        },
        Err(err) => {
            //error parsing the dlt packet
            println!("ERROR: {:?}", err);
        }
    }
}
```

An complete example which includes the parsing of the ethernet & udp headers can be found in [examples/print_messages_ids.rs](examples/print_messages_ids.rs)

## References
* [Log and Trace Protocol Specification](https://www.autosar.org/fileadmin/standards/foundation/1-3/AUTOSAR_PRS_LogAndTraceProtocol.pdf)

## License
Licensed under either of Apache License, Version 2.0 or MIT license at your option. The corresponding license texts can be found in the LICENSE-APACHE file and the LICENSE-MIT file.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed as above, without any additional terms or conditions.
