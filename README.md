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
dlt_parse = "0.10.0"
```

If you additionally want `serde` support you will have to activate the `serde` feature:

```toml
[dependencies]
dlt_parse = { version = "0.10.0", features = ["serde"] }
```

If you want to use the crate in `no_std` mode you will have to disable the default features:

```toml
[dependencies]
dlt_parse = { version = "0.10.0", default-features = false }
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
            // check what type of message was received
            match dlt_slice.typed_payload() {
                Ok(typed_payload) => {
                    use dlt_parse::DltTypedPayload::*;
                    match typed_payload {
                        UnknownNv(p) => {
                            println!(
                                "non verbose message 0x{:x} (unknown) with {} bytes of payload.",
                                p.msg_id,
                                p.payload.len(),
                            );
                        }
                        LogNv(p) => {
                            println!(
                                "non verbose log message 0x{:x} with log level {:?} and {} bytes of payload.",
                                p.msg_id,
                                p.log_level,
                                p.payload.len(),
                            );
                        }
                        LogV(p) => {
                            println!(
                                "verbose log message with log level {:?} and values:",
                                p.log_level
                            );
                            for value in p.iter {
                                println!("  {:?}", value);
                            }
                        }
                        TraceNv(p) => {
                            println!(
                                "non verbose trace message 0x{:x} of type {:?} and {} bytes of payload.",
                                p.msg_id,
                                p.trace_type,
                                p.payload.len(),
                            );
                        }
                        TraceV(p) => {
                            println!(
                                "verbose trace message with of type {:?} and values:",
                                p.trace_type
                            );
                            for value in p.iter {
                                println!("  {:?}", value);
                            }
                        }
                        NetworkNv(p) => {
                            println!(
                                "non verbose network message 0x{:x} of type {:?} and {} bytes of payload.",
                                p.msg_id,
                                p.net_type,
                                p.payload.len(),
                            );
                        }
                        NetworkV(p) => {
                            println!(
                                "verbose network message with of type {:?} and values:",
                                p.net_type
                            );
                            for value in p.iter {
                                println!("  {:?}", value);
                            }
                        }
                        ControlNv(p) => {
                            println!("non verbose control message {:?} with service id: {} and {} bytes of payload.", p.msg_type, p.service_id, p.payload.len());
                        }
                        ControlV(p) => {
                            println!("verbose control message {:?} with values:", p.msg_type);
                            for value in p.iter {
                                println!("  {:?}", value);
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("message with payload error received: {}", err);
                }
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
* [COVESA DLT Filetransfer](https://github.com/COVESA/dlt-daemon/blob/603f0e4bb87478f7d3e95c89b37790e55ff1e4e5/doc/dlt_filetransfer.md)

## License
Licensed under either of Apache License, Version 2.0 or MIT license at your option. The corresponding license texts can be found in the LICENSE-APACHE file and the LICENSE-MIT file.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed as above, without any additional terms or conditions.
