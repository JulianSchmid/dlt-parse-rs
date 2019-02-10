[![Crates.io](https://img.shields.io/crates/v/dlt_parse.svg)](https://crates.io/crates/dlt_parse)
[![Build Status](https://travis-ci.org/JulianSchmid/dlt-parse-rs.svg?branch=master)](https://travis-ci.org/JulianSchmid/dlt-parse-rs)
[![Coverage Status](https://codecov.io/gh/JulianSchmid/dlt-parse-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/JulianSchmid/dlt-parse-rs)

# dlt_parse

A zero allocation rust library for basic parsing & writing DLT (Diagnostic Log and Trace)
packets. Currently only the parsing and writing of the header is supported (excluding the
verbose packet definitions).

## Usage:

First, add the following to your `Cargo.toml`:

```toml
[dependencies]
dlt_parse = "0.1.0"
```

Next, add this to your crate root:

```rust
use dlt_parse;
```

Or for pre 2018 Rust:

```rust
extern crate dlt_parse;
```

## Slicing non-verbose packets

Slicing the packets allows reading a dlt header & payload without reading the entire packet.

```rust
use self::dlt_parse::{DltHeader, DltExtendedHeader, SliceIterator};

let header = {
    let mut header = DltHeader {
        is_big_endian: true, //payload & message id are encoded with big endian
        version: 0,
        message_counter: 0,
        length: 0,
        ecu_id: None,
        session_id: None,
        timestamp: None,
        extended_header: Some(DltExtendedHeader::new_non_verbose(
            123,//application id
            1,//context id
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
    //byteorder crate is used for writing the message id with  the correct endianess
    use byteorder::{BigEndian, WriteBytesExt};

    //write the message id & payload
    buffer.write_u32::<BigEndian>(1234).unwrap(); //message id
    buffer.write_all(&[1,2,3,4]); //payload
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
