//! A zero allocation rust library for basic parsing & writing DLT (Diagnostic Log and Trace)
//! packets. Currently only the parsing and writing of the header is supported (excluding the
//! verbose packet definitions).
//!
//! # Usage:
//!
//! First, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! dlt_parse = "0.4.0"
//! ```
//!
//! Next, add this to your crate:
//!
//! ```
//! use dlt_parse;
//! ```
//!
//! # What is dlt_parse?
//! dlt_parse is a library that aims to provide serialisation & deserialisation funtions for DLT (Diagnostic Log and Trace) packets.
//! It should make it possible to anlyse recordings of DLT packets as fast as possible, as well as writing servers
//! that send DLT packets to the network.
//!
//! Some key points are:
//!
//! * It is completly written in Rust and thoroughly tested.
//! * Special attention has been paid to not use allocations or syscalls.
//! * It is possible to use the crate in an `no-std` environment.
//! * The package is still in development and can & will still change.
//! * Methods for parsing verbose DLT packets are still missing (but maybe will be implemented in future versions).
//!
//! # Example: Serializing & Slicing/Deserializing DLT Packets
//!
//! In this example a non verbose DLT packet is serialized and deserialized again. Specificly the serialized packet is
//! converted into a DltPacketSlice. This has the advantage, that not all fields have to be deserialied to
//! access the payload or specific fields in the header. Note that it is also possible to completely deserialize
//! DLT headers with the DltHeader::read function. This can make sense, if most fields of the header are used anyways.
//!
//! ```
//! use self::dlt_parse::{DltHeader, DltLogLevel, DltExtendedHeader, SliceIterator};
//!
//! let header = {
//!     let mut header = DltHeader {
//!         is_big_endian: true, //payload & message id are encoded with big endian
//!         message_counter: 0,
//!         length: 0,
//!         ecu_id: None,
//!         session_id: None,
//!         timestamp: None,
//!         extended_header: Some(DltExtendedHeader::new_non_verbose_log(
//!             DltLogLevel::Debug,
//!             [b'a', b'p', b'p', b'i'],//application id
//!             [b'c', b't', b'x', b'i'],//context id
//!         ))
//!     };
//!     header.length = header.header_len() + 4 + 4; //header + message id + payload
//!
//!     header
//! };
//!
//! //buffer to store serialized header & payload
//! let mut buffer = Vec::<u8>::with_capacity(usize::from(header.length));
//! buffer.extend_from_slice(&header.to_bytes());
//!
//! //write payload (message id 1234 & non verbose payload)
//! {
//!     //for write_all
//!     use std::io::Write;
//!
//!     //write the message id & payload
//!     buffer.write_all(&1234u32.to_be_bytes()).unwrap(); //message id
//!     buffer.write_all(&[5,6,7,9]); //payload
//! }
//!
//! //packets can contain multiple dlt messages, iterate through them
//! for dlt_message in SliceIterator::new(&buffer) {
//!     match dlt_message {
//!         Ok(dlt_slice) => {
//!             //check if the message is verbose or non verbose (non verbose messages have message ids)
//!             if let Some(message_id) = dlt_slice.message_id() {
//!                 println!("non verbose message {:x}", message_id);
//!                 println!("  with payload {:?}", dlt_slice.non_verbose_payload());
//!             } else {
//!                 println!("verbose message (parsing not yet supported)");
//!             }
//!         },
//!         Err(err) => {
//!             //error parsing the dlt packet
//!             println!("ERROR: {:?}", err);
//!         }
//!     }
//! }
//! ```
//!
//! An complete example which includes the parsing of the ethernet & udp headers can be found in [examples/print_messages_ids.rs](https://github.com/JulianSchmid/dlt-parse-rs/blob/0.1.0/examples/print_messages_ids.rs)
//!
//! # References
//! * [Log and Trace Protocol Specification](https://www.autosar.org/fileadmin/user_upload/standards/foundation/1-3/AUTOSAR_PRS_LogAndTraceProtocol.pdf)

#![no_std]

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
extern crate proptest;
#[cfg(any(feature = "std", test))]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod dlt_extended_header;
pub use dlt_extended_header::*;
mod dlt_header;
pub use dlt_header::*;
mod dlt_packet_slice;
pub use dlt_packet_slice::*;
mod dlt_slice_iterator;
pub use dlt_slice_iterator::*;
pub mod error;

/// Module for decoding .dlt files or other formats that use the DLT storage header.
pub mod storage;

#[cfg(test)]
use alloc::{format, vec, vec::Vec};
use arrayvec::ArrayVec;
use core::slice::from_raw_parts;
#[cfg(feature = "std")]
use std::io;
#[cfg(test)]
mod proptest_generators;

/// Maximum value that can be encoded in the DLT header version field (has only 3 bits).
const MAX_VERSION: u8 = 0b111;

const EXTDENDED_HEADER_FLAG: u8 = 0b1;
const BIG_ENDIAN_FLAG: u8 = 0b10;
const ECU_ID_FLAG: u8 = 0b100;
const SESSION_ID_FLAG: u8 = 0b1000;
const TIMESTAMP_FLAG: u8 = 0b10000;

///Shifted value in the msin extended header field for dlt "log" messages.
const EXT_MSIN_MSTP_TYPE_LOG: u8 = 0x0 << 1;
///Shifted value in the msin extended header field for dlt "trace" messages.
const EXT_MSIN_MSTP_TYPE_TRACE: u8 = 0x1 << 1;
///Shifted value in the msin extended header field for dlt "network trace" messages.
const EXT_MSIN_MSTP_TYPE_NW_TRACE: u8 = 0x2 << 1;
///Shifted value in the msin extended header field for dlt "control" messages.
const EXT_MSIN_MSTP_TYPE_CONTROL: u8 = 0x3 << 1;

///Log level for dlt log messages.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DltLogLevel {
    ///Fatal system error.
    Fatal = 0x1,
    ///SWC error.
    Error = 0x2,
    ///Correct behavior cannot be ensured.
    Warn = 0x3,
    ///Message of LogLevel type “Information”.
    Info = 0x4,
    ///Message of LogLevel type “Debug”.
    Debug = 0x5,
    ///Message of LogLevel type "Verbose".
    Verbose = 0x6,
}

///Types of application trace messages that can be sent via dlt if the message type
///is specified as "trace".
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DltTraceType {
    ///Value of variable.
    Variable = 0x1,
    ///Call of a function.
    FunctionIn = 0x2,
    ///Return of a function.
    FunctionOut = 0x3,
    ///State of a state machine.
    State = 0x4,
    ///RTE Events.
    Vfb = 0x5,
}

///Network type specified in a network trace dlt message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DltNetworkType {
    ///Inter-Process-Communication.
    Ipc,
    ///CAN communication bus.
    Can,
    ///FlexRay communication bus.
    Flexray,
    ///Most communication bus.
    Most,
    ///Ethernet communication bus.
    Ethernet,
    ///SOME/IP communication.
    SomeIp,
    ///User defined settings (note that the maximum allowed value is 0xf or 15).
    UserDefined(u8),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DltControlMessageType {
    ///Request control message.
    Request = 0x1,
    ///Respond control message.
    Response = 0x2,
}

///Message type info field (contains the the information of the message type & message type info field)
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DltMessageType {
    ///Dlt log message with a log level
    Log(DltLogLevel),
    ///Message tracing the value of a function, variable or other parts of the system.
    Trace(DltTraceType),
    ///Message containing tracing informations from a networking system.
    NetworkTrace(DltNetworkType),
    ///A dlt control message (e.g. for setting the log level).
    Control(DltControlMessageType),
}

impl DltMessageType {
    /// Attempts to read the message type from the first byte of
    /// the dlt extended message header.
    fn from_byte(value: u8) -> Option<DltMessageType> {
        use DltMessageType::*;

        const MSIN_MASK: u8 = 0b1111_0000;

        match value & 0b0000_1110 {
            EXT_MSIN_MSTP_TYPE_LOG => {
                use DltLogLevel::*;
                match (value & MSIN_MASK) >> 4 {
                    0x1 => Some(Log(Fatal)),
                    0x2 => Some(Log(Error)),
                    0x3 => Some(Log(Warn)),
                    0x4 => Some(Log(Info)),
                    0x5 => Some(Log(Debug)),
                    0x6 => Some(Log(Verbose)),
                    //undefined values
                    _ => None,
                }
            }
            EXT_MSIN_MSTP_TYPE_TRACE => {
                use DltTraceType::*;
                match (value & MSIN_MASK) >> 4 {
                    0x1 => Some(Trace(Variable)),
                    0x2 => Some(Trace(FunctionIn)),
                    0x3 => Some(Trace(FunctionOut)),
                    0x4 => Some(Trace(State)),
                    0x5 => Some(Trace(Vfb)),
                    //undefined values
                    _ => None,
                }
            }
            EXT_MSIN_MSTP_TYPE_NW_TRACE => {
                use DltNetworkType::*;
                match (value & MSIN_MASK) >> 4 {
                    0x1 => Some(NetworkTrace(Ipc)),
                    0x2 => Some(NetworkTrace(Can)),
                    0x3 => Some(NetworkTrace(Flexray)),
                    0x4 => Some(NetworkTrace(Most)),
                    0x5 => Some(NetworkTrace(Ethernet)),
                    0x6 => Some(NetworkTrace(SomeIp)),
                    //user defined
                    other => Some(NetworkTrace(UserDefined(other))),
                }
            }
            EXT_MSIN_MSTP_TYPE_CONTROL => {
                use DltControlMessageType::*;
                match (value & MSIN_MASK) >> 4 {
                    0x1 => Some(Control(Request)),
                    0x2 => Some(Control(Response)),
                    //undefined values
                    _ => None,
                }
            }
            _ => None,
        }
    }

    ///Set message type info and based on that the message type.
    pub fn to_byte(&self) -> Result<u8, error::RangeError> {
        use error::RangeError::NetworkTypekUserDefinedOutsideOfRange;
        use DltMessageType::*;
        use DltNetworkType::UserDefined;

        //check ranges
        if let NetworkTrace(UserDefined(user_defined_value)) = *self {
            if !(7..=0xf).contains(&user_defined_value) {
                return Err(NetworkTypekUserDefinedOutsideOfRange(user_defined_value));
            }
        }

        //determine message type & message type info
        let (message_type, message_type_info) = match self {
            Log(ref level) => (EXT_MSIN_MSTP_TYPE_LOG, *level as u8),
            Trace(ref trace_type) => (EXT_MSIN_MSTP_TYPE_TRACE, *trace_type as u8),
            NetworkTrace(ref nw_trace_type) => {
                use DltNetworkType::*;

                (
                    EXT_MSIN_MSTP_TYPE_NW_TRACE,
                    match *nw_trace_type {
                        Ipc => 0x1,
                        Can => 0x2,
                        Flexray => 0x3,
                        Most => 0x4,
                        Ethernet => 0x5,
                        SomeIp => 0x6,
                        UserDefined(value) => value,
                    },
                )
            }
            Control(ref control_msg_type) => (EXT_MSIN_MSTP_TYPE_CONTROL, *control_msg_type as u8),
        };

        Ok(message_type | ((message_type_info << 4) & 0b1111_0000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod dlt_log_level {
        use super::*;
        use DltLogLevel::*;

        #[test]
        fn clone_eq() {
            const VALUES: [(DltLogLevel, u8); 6] = [
                (Fatal, 1),
                (Error, 2),
                (Warn, 3),
                (Info, 4),
                (Debug, 5),
                (Verbose, 6),
            ];

            for v0 in &VALUES {
                // identity property
                assert_eq!(v0.0, v0.0.clone());
                assert_eq!(v0.0 as u8, v0.1);

                for v1 in &VALUES {
                    assert_eq!(v0.0 != v1.0, v0.1 != v1.1,);
                }
            }
        }

        #[test]
        fn debug() {
            const VALUES: [(DltLogLevel, &str); 6] = [
                (Fatal, "Fatal"),
                (Error, "Error"),
                (Warn, "Warn"),
                (Info, "Info"),
                (Debug, "Debug"),
                (Verbose, "Verbose"),
            ];
            for v in &VALUES {
                assert_eq!(v.1, format!("{:?}", v.0));
            }
        }
    }

    mod dlt_trace_type {
        use super::*;
        use DltTraceType::*;

        #[test]
        fn clone_eq() {
            const VALUES: [(DltTraceType, u8); 5] = [
                (Variable, 1),
                (FunctionIn, 2),
                (FunctionOut, 3),
                (State, 4),
                (Vfb, 5),
            ];

            for v0 in &VALUES {
                // identity property
                assert_eq!(v0.0, v0.0.clone());
                assert_eq!(v0.0 as u8, v0.1);

                for v1 in &VALUES {
                    assert_eq!(v0.0 != v1.0, v0.1 != v1.1,);
                }
            }
        }

        #[test]
        fn debug() {
            const VALUES: [(DltTraceType, &str); 5] = [
                (Variable, "Variable"),
                (FunctionIn, "FunctionIn"),
                (FunctionOut, "FunctionOut"),
                (State, "State"),
                (Vfb, "Vfb"),
            ];
            for v in &VALUES {
                assert_eq!(v.1, format!("{:?}", v.0));
            }
        }
    }

    mod dlt_network_type {
        use super::*;
        use DltNetworkType::*;

        #[test]
        fn clone_eq() {
            const VALUES: [(DltNetworkType, u8); 8] = [
                (Ipc, 1),
                (Can, 2),
                (Flexray, 3),
                (Most, 4),
                (Ethernet, 5),
                (SomeIp, 6),
                (UserDefined(0x7), 0x7),
                (UserDefined(0xf), 0xf),
            ];

            for v0 in &VALUES {
                assert_eq!(v0.0, v0.0.clone());

                for v1 in &VALUES {
                    assert_eq!(v0.0 != v1.0, v0.1 != v1.1,);
                }
            }
        }

        #[test]
        fn debug() {
            const VALUES: [(DltNetworkType, &str); 8] = [
                (Ipc, "Ipc"),
                (Can, "Can"),
                (Flexray, "Flexray"),
                (Most, "Most"),
                (Ethernet, "Ethernet"),
                (SomeIp, "SomeIp"),
                (UserDefined(0x7), "UserDefined(7)"),
                (UserDefined(0xf), "UserDefined(15)"),
            ];
            for v in &VALUES {
                assert_eq!(v.1, format!("{:?}", v.0));
            }
        }
    }

    mod dlt_control_message_type {
        use super::*;
        use DltControlMessageType::*;

        #[test]
        fn clone_eq() {
            const VALUES: [(DltControlMessageType, u8); 2] = [(Request, 1), (Response, 2)];

            for v0 in &VALUES {
                // identity property
                assert_eq!(v0.0, v0.0.clone());
                assert_eq!(v0.0.clone() as u8, v0.1);

                for v1 in &VALUES {
                    assert_eq!(v0.0 != v1.0, v0.1 != v1.1,);
                }
            }
        }

        #[test]
        fn debug() {
            const VALUES: [(DltControlMessageType, &str); 2] =
                [(Request, "Request"), (Response, "Response")];
            for v in &VALUES {
                assert_eq!(v.1, format!("{:?}", v.0));
            }
        }
    }

    mod dlt_message_type {
        use super::*;

        use DltControlMessageType::*;
        use DltLogLevel::*;
        use DltMessageType::*;
        use DltNetworkType::*;
        use DltTraceType::*;

        const VALUES: [(DltMessageType, u8); 28] = [
            (Log(Fatal), 0b0001_0000),
            (Log(Error), 0b0010_0000),
            (Log(Warn), 0b0011_0000),
            (Log(Info), 0b0100_0000),
            (Log(Debug), 0b0101_0000),
            (Log(Verbose), 0b0110_0000),
            (Trace(Variable), 0b0001_0010),
            (Trace(FunctionIn), 0b0010_0010),
            (Trace(FunctionOut), 0b0011_0010),
            (Trace(State), 0b0100_0010),
            (Trace(Vfb), 0b0101_0010),
            (NetworkTrace(Ipc), 0b0001_0100),
            (NetworkTrace(Can), 0b0010_0100),
            (NetworkTrace(Flexray), 0b0011_0100),
            (NetworkTrace(Most), 0b0100_0100),
            (NetworkTrace(Ethernet), 0b0101_0100),
            (NetworkTrace(SomeIp), 0b0110_0100),
            (NetworkTrace(UserDefined(0x7)), 0b0111_0100),
            (NetworkTrace(UserDefined(0x8)), 0b1000_0100),
            (NetworkTrace(UserDefined(0x9)), 0b1001_0100),
            (NetworkTrace(UserDefined(0xA)), 0b1010_0100),
            (NetworkTrace(UserDefined(0xB)), 0b1011_0100),
            (NetworkTrace(UserDefined(0xC)), 0b1100_0100),
            (NetworkTrace(UserDefined(0xD)), 0b1101_0100),
            (NetworkTrace(UserDefined(0xE)), 0b1110_0100),
            (NetworkTrace(UserDefined(0xF)), 0b1111_0100),
            (Control(Request), 0b0001_0110),
            (Control(Response), 0b0010_0110),
        ];

        #[test]
        fn clone_eq() {
            for v0 in &VALUES {
                // identity property
                assert_eq!(v0.0, v0.0.clone());

                for v1 in &VALUES {
                    assert_eq!(v0.0 != v1.0, v0.1 != v1.1,);
                }
            }
        }

        #[test]
        fn debug() {
            const DBG_VALUES: [(DltMessageType, &str); 5] = [
                (Log(Fatal), "Log(Fatal)"),
                (Trace(Variable), "Trace(Variable)"),
                (NetworkTrace(Ipc), "NetworkTrace(Ipc)"),
                (
                    NetworkTrace(UserDefined(0x7)),
                    "NetworkTrace(UserDefined(7))",
                ),
                (Control(Request), "Control(Request)"),
            ];
            for v in &DBG_VALUES {
                assert_eq!(v.1, format!("{:?}", v.0));
            }
        }

        #[test]
        fn from_byte() {
            // valid values
            for value in &VALUES {
                assert_eq!(value.0, DltMessageType::from_byte(value.1).unwrap());
                // with verbose flag set
                assert_eq!(value.0, DltMessageType::from_byte(value.1 | 1).unwrap());
            }

            // invalid log
            assert!(DltMessageType::from_byte(0).is_none());
            assert!(DltMessageType::from_byte(1).is_none()); // with verbose
            for i in 7..=0b1111 {
                assert!(DltMessageType::from_byte(i << 4).is_none());
                // with verbose
                assert!(DltMessageType::from_byte((i << 4) | 1).is_none());
            }

            // invalid trace
            assert!(DltMessageType::from_byte(0b0000_0010).is_none());
            assert!(DltMessageType::from_byte(0b0000_0011).is_none());
            for i in 6..=0b1111 {
                assert!(DltMessageType::from_byte((i << 4) | 0b0010).is_none());
                // with verbose
                assert!(DltMessageType::from_byte((i << 4) | 0b0011).is_none());
            }

            // invalid control
            assert!(DltMessageType::from_byte(0b0000_0110).is_none());
            assert!(DltMessageType::from_byte(0b0000_0111).is_none());
            for i in 3..=0b1111 {
                assert!(DltMessageType::from_byte((i << 4) | 0b0110).is_none());
                // with verbose
                assert!(DltMessageType::from_byte((i << 4) | 0b0111).is_none());
            }
        }

        #[test]
        fn to_byte() {
            // valid values
            for value in &VALUES {
                assert_eq!(value.0.to_byte().unwrap(), value.1);
            }

            // invalid user defined errors
            // first run two explicitly to check the error contains the
            // actual value
            use error::RangeError::NetworkTypekUserDefinedOutsideOfRange;
            assert_matches!(
                NetworkTrace(UserDefined(0)).to_byte().unwrap_err(),
                NetworkTypekUserDefinedOutsideOfRange(0)
            );
            assert_matches!(
                NetworkTrace(UserDefined(1)).to_byte().unwrap_err(),
                NetworkTypekUserDefinedOutsideOfRange(1)
            );
            // check the rest of the range of invalid values
            for value in 0..7 {
                assert_matches!(
                    NetworkTrace(UserDefined(value)).to_byte().unwrap_err(),
                    NetworkTypekUserDefinedOutsideOfRange(_)
                );
            }
            for value in 16..=0xff {
                assert_matches!(
                    NetworkTrace(UserDefined(value)).to_byte().unwrap_err(),
                    NetworkTypekUserDefinedOutsideOfRange(_)
                );
            }
        }
    }
} // mod tests
