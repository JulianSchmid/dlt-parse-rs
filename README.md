[![Crates.io](https://img.shields.io/crates/v/dlt_parse.svg)](https://crates.io/crates/dlt_parse)
[![docs.rs](https://docs.rs/dlt_parse/badge.svg)](https://docs.rs/dlt_parse)
[![build status github](https://github.com/github/docs/actions/workflows/main.yml/badge.svg)
[![build status appveyor](https://ci.appveyor.com/api/projects/status/3tba6q6as9kqr1wa?svg=true)](https://ci.appveyor.com/project/JulianSchmid/dlt-parse-rs)
[![codecov](https://codecov.io/gh/JulianSchmid/dlt-parse-rs/branch/master/graph/badge.svg?token=D1LANr6nox)](https://codecov.io/gh/JulianSchmid/dlt-parse-rs)

# dlt_parse

A zero allocation rust library for basic parsing & writing DLT (Diagnostic Log and Trace)
packets. Currently only the parsing and writing of the header is supported (excluding the
verbose packet definitions).

## Usage:

First, add the following to your `Cargo.toml`:

```toml
[dependencies]
dlt_parse = "0.3.0"
```

Next, add this to your crate:

```rust
use dlt_parse;
```

## What is dlt_parse?
dlt_parse is a library that aims to provide serialisation & deserialisation funtions for DLT (Diagnostic Log and Trace) packets.
It should make it possible to anlyse recordings of DLT packets as fast as possible, as well as writing servers
that send DLT packets to the network.

Some key points are:

* It is completly written in Rust and thoroughly tested.
* Special attention has been paid to not use allocations or syscalls.
* The package is still in development and can & will still change.
* Methods for parsing verbose DLT packets are still missing (but maybe will be implemented in future versions).

## Example: Serializing & Slicing/Deserializing DLT Packets

In this example a non verbose DLT packet is serialized and deserialized again. Specificly the serialized packet is
converted into a DltPacketSlice. This has the advantage, that not all fields have to be deserialied to
access the payload or specific fields in the header. Note that it is also possible to completely deserialize
DLT headers with the DltHeader::read function. This can make sense, if most fields of the header are used anyways.

```rust
use self::dlt_parse::{DltHeader, DltLogLevel, DltExtendedHeader, SliceIterator};

let header = {
    let mut header = DltHeader {
        is_big_endian: true, //payload & message id are encoded with big endian
        version: 0,
        message_counter: 0,
        length: 0,
        ecu_id: None,
        session_id: None,
        timestamp: None,
        extended_header: Some(DltExtendedHeader::new_non_verbose_log(
            DltLogLevel::Debug,
            [b'a', b'p', b'p', b'i'],//application id
            [b'c', b't', b'x', b'i'],//context id
        ))
    };
    header.length = header.header_len() + 4 + 4; //header + message id + payload

    header
};

//buffer to store serialized header & payload
let mut buffer = Vec::<u8>::with_capacity(usize::from(header.length));
header.write(&mut buffer).unwrap();

//write payload (message id 1234 & non verbose payload)
{
    //for write_all
    use std::io::Write;

    //write the message id & payload
    buffer.write_all(&1234u32.to_be_bytes()).unwrap(); //message id
    buffer.write_all(&[5,6,7,9]); //payload
}

//packets can contain multiple dlt messages, iterate through them
for dlt_message in SliceIterator::new(&buffer) {
    match dlt_message {
        Ok(dlt_slice) => {
            //check if the message is verbose or non verbose (non verbose messages have message ids)
            if let Some(message_id) = dlt_slice.message_id() {
                println!("non verbose message {:x}", message_id);
                println!("  with payload {:?}", dlt_slice.non_verbose_payload());
            } else {
                println!("verbose message (parsing not yet supported)");
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
* [Log and Trace Protocol Specification](https://www.autosar.org/fileadmin/user_upload/standards/foundation/1-3/AUTOSAR_PRS_LogAndTraceProtocol.pdf)

## License
Licensed under the BSD 3-Clause license. Please see the LICENSE file for more information.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed as above, without any additional terms or conditions.
