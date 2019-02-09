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
extern crate dlt_parse;
```

## References
* [Log and Trace Protocol Specification](https://www.autosar.org/fileadmin/user_upload/standards/foundation/1-3/AUTOSAR_PRS_LogAndTraceProtocol.pdf)

## License
Licensed under the BSD 3-Clause license. Please see the LICENSE file for more information.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed as above, without any additional terms or conditions.
