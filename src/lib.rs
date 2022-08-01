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
//! dlt_parse = "0.3.0"
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
//!         version: 0,
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
//! header.write(&mut buffer).unwrap();
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

use std::fmt;
use std::io;
use std::slice::from_raw_parts;

#[cfg(test)]
extern crate proptest;
#[cfg(test)]
mod proptest_generators;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod error;

///Errors that can occure on reading a dlt header.
#[derive(Debug)]
pub enum ReadError {
    ///Error if the slice is smaller then dlt length field or minimal size.
    UnexpectedEndOfSlice(error::UnexpectedEndOfSliceError),
    ///Error if the dlt length is smaller then the header the calculated header size based on the flags (+ minimum payload size of 4 bytes/octetets)
    LengthSmallerThenMinimum {
        required_length: usize,
        length: usize,
    },
    ///Standard io error.
    IoError(io::Error),
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::IoError(err)
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReadError::IoError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ReadError::*;

        match self {
            UnexpectedEndOfSlice(err) => {
                write!(f, "ReadError: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.", err.actual_size, err.minimum_size)
            }
            LengthSmallerThenMinimum {
                required_length,
                length,
            } => {
                write!(f, "ReadError: The length of {} in the dlt header is smaller then minimum required size of {} bytes.", length, required_length)
            }
            IoError(err) => err.fmt(f),
        }
    }
}

///Errors that can occur when serializing a dlt header.
#[derive(Debug)]
pub enum WriteError {
    VersionTooLarge(u8),
    IoError(io::Error),
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> WriteError {
        WriteError::IoError(err)
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WriteError::IoError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WriteError::*;

        match self {
            VersionTooLarge(version) => {
                write!(
                    f,
                    "WriteError: DLT version {} is larger then the maximum supported value of {}",
                    version, MAX_VERSION
                )
            }
            IoError(err) => err.fmt(f),
        }
    }
}

/// Error that can occur when an out of range value is passed to a function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RangeError {
    /// Error if the user defined value is outside the range of 7-15
    NetworkTypekUserDefinedOutsideOfRange(u8),
}

impl std::error::Error for RangeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RangeError::*;

        match self {
            NetworkTypekUserDefinedOutsideOfRange(value) => {
                write!(f, "RangeError: Message type info field user defined value of {} outside of the allowed range of 7-15.", value)
            }
        }
    }
}

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

///A dlt message header
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DltHeader {
    ///If true the payload is encoded in big endian. This does not influence the fields of the dlt header, which is always encoded in big endian.
    pub is_big_endian: bool,
    pub version: u8,
    pub message_counter: u8,
    pub length: u16,
    pub ecu_id: Option<u32>,
    pub session_id: Option<u32>,
    pub timestamp: Option<u32>,
    pub extended_header: Option<DltExtendedHeader>,
}

impl DltHeader {
    ///Deserialize a DltHeader & TpHeader from the given reader.
    pub fn read<T: io::Read + Sized>(reader: &mut T) -> Result<DltHeader, ReadError> {
        // read the standard header that is always present
        let standard_header_start = {
            let mut standard_header_start: [u8; 4] = [0; 4];
            reader.read_exact(&mut standard_header_start)?;
            standard_header_start
        };

        //first lets read the header type
        let header_type = standard_header_start[0];
        //let extended_header = 0 != header_type & EXTDENDED_HEADER_FLAG;
        Ok(DltHeader {
            is_big_endian: 0 != header_type & BIG_ENDIAN_FLAG,
            version: (header_type >> 5) & MAX_VERSION,
            message_counter: standard_header_start[1],
            length: u16::from_be_bytes([standard_header_start[2], standard_header_start[3]]),
            ecu_id: if 0 != header_type & ECU_ID_FLAG {
                Some({
                    let mut buffer: [u8; 4] = [0; 4];
                    reader.read_exact(&mut buffer)?;
                    u32::from_be_bytes(buffer)
                })
            } else {
                None
            },
            session_id: if 0 != header_type & SESSION_ID_FLAG {
                Some({
                    let mut buffer: [u8; 4] = [0; 4];
                    reader.read_exact(&mut buffer)?;
                    u32::from_be_bytes(buffer)
                })
            } else {
                None
            },
            timestamp: if 0 != header_type & TIMESTAMP_FLAG {
                Some({
                    let mut buffer: [u8; 4] = [0; 4];
                    reader.read_exact(&mut buffer)?;
                    u32::from_be_bytes(buffer)
                })
            } else {
                None
            },
            extended_header: if 0 != header_type & EXTDENDED_HEADER_FLAG {
                Some({
                    let mut buffer: [u8; 10] = [0; 10];
                    reader.read_exact(&mut buffer)?;

                    DltExtendedHeader {
                        message_info: buffer[0],
                        number_of_arguments: buffer[1],
                        application_id: [buffer[2], buffer[3], buffer[4], buffer[5]],
                        context_id: [buffer[6], buffer[7], buffer[8], buffer[9]],
                    }
                })
            } else {
                None
            },
        })
    }

    ///Serializes the header to the given writer.
    pub fn write<T: io::Write + Sized>(&self, writer: &mut T) -> Result<(), WriteError> {
        //pre check if the ranges of all fields are valid
        if self.version > MAX_VERSION {
            return Err(WriteError::VersionTooLarge(self.version));
        }

        {
            let length_be = self.length.to_be_bytes();
            let standard_header_start: [u8; 4] = [
                //header type bitfield
                {
                    let mut result = 0;
                    if self.extended_header.is_some() {
                        result |= EXTDENDED_HEADER_FLAG;
                    }
                    if self.is_big_endian {
                        result |= BIG_ENDIAN_FLAG;
                    }
                    if self.ecu_id.is_some() {
                        result |= ECU_ID_FLAG;
                    }
                    if self.session_id.is_some() {
                        result |= SESSION_ID_FLAG;
                    }
                    if self.timestamp.is_some() {
                        result |= TIMESTAMP_FLAG;
                    }
                    result |= (self.version << 5) & 0b1110_0000;
                    result
                },
                self.message_counter,
                length_be[0],
                length_be[1],
            ];

            writer.write_all(&standard_header_start)?;
        }

        if let Some(value) = self.ecu_id {
            writer.write_all(&value.to_be_bytes())?;
        }

        if let Some(value) = self.session_id {
            writer.write_all(&value.to_be_bytes())?;
        }

        if let Some(value) = self.timestamp {
            writer.write_all(&value.to_be_bytes())?;
        }

        //write the extended header if it exists
        match &self.extended_header {
            Some(value) => {
                let bytes: [u8; 10] = [
                    value.message_info,
                    value.number_of_arguments,
                    value.application_id[0],
                    value.application_id[1],
                    value.application_id[2],
                    value.application_id[3],
                    value.context_id[0],
                    value.context_id[1],
                    value.context_id[2],
                    value.context_id[3],
                ];
                writer.write_all(&bytes)?;
            }
            None => {}
        }
        Ok(())
    }

    ///Returns if the package is a verbose package
    #[inline]
    pub fn is_verbose(&self) -> bool {
        match &self.extended_header {
            None => false, //only packages with extended headers can be verbose
            Some(ext) => ext.is_verbose(),
        }
    }

    ///Return the byte/octed size of the serialized header (including extended header)
    #[inline]
    pub fn header_len(&self) -> u16 {
        4 + match self.ecu_id {
            Some(_) => 4,
            None => 0,
        } + match self.session_id {
            Some(_) => 4,
            None => 0,
        } + match self.timestamp {
            Some(_) => 4,
            None => 0,
        } + match self.extended_header {
            Some(_) => 10,
            None => 0,
        }
    }
}

///Extended dlt header (optional header in the dlt header)
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DltExtendedHeader {
    pub message_info: u8,
    pub number_of_arguments: u8,
    pub application_id: [u8; 4],
    pub context_id: [u8; 4],
}

impl DltExtendedHeader {
    ///Create a extended header for a non verbose log message with given application id & context id.
    pub fn new_non_verbose_log(
        log_level: DltLogLevel,
        application_id: [u8; 4],
        context_id: [u8; 4],
    ) -> DltExtendedHeader {
        DltExtendedHeader {
            message_info: DltMessageType::Log(log_level).to_byte().unwrap(),
            number_of_arguments: 0,
            application_id,
            context_id,
        }
    }

    ///Create a extended header for a non verbose message with given message type, application id & context id.
    pub fn new_non_verbose(
        message_type: DltMessageType,
        application_id: [u8; 4],
        context_id: [u8; 4],
    ) -> Result<DltExtendedHeader, RangeError> {
        Ok(DltExtendedHeader {
            message_info: message_type.to_byte()?,
            number_of_arguments: 0,
            application_id,
            context_id,
        })
    }

    ///Returns true if the extended header flags the message as a verbose message.
    #[inline]
    pub fn is_verbose(&self) -> bool {
        0 != self.message_info & 0b1
    }

    ///Sets or unsets the is_verbose bit in the DltExtendedHeader.
    #[inline]
    pub fn set_is_verbose(&mut self, is_verbose: bool) {
        if is_verbose {
            self.message_info |= 0b1;
        } else {
            self.message_info &= 0b1111_1110;
        }
    }

    ///Returns message type info or `Option::None` for reserved values.
    #[inline]
    pub fn message_type(&self) -> Option<DltMessageType> {
        DltMessageType::from_byte(self.message_info)
    }

    ///Set message type info and based on that the message type.
    #[inline]
    pub fn set_message_type(&mut self, value: DltMessageType) -> Result<(), RangeError> {
        let encoded = value.to_byte()?;

        //unset old message type & set the new one
        self.message_info &= 0b0000_0001;
        self.message_info |= encoded;

        //all good
        Ok(())
    }
}

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
    pub fn to_byte(&self) -> Result<u8, RangeError> {
        use DltMessageType::*;
        use DltNetworkType::UserDefined;
        use RangeError::NetworkTypekUserDefinedOutsideOfRange;

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

///A slice containing an dlt header & payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltPacketSlice<'a> {
    slice: &'a [u8],
    header_len: usize,
}

impl<'a> DltPacketSlice<'a> {
    ///Read the dlt header and create a slice containing the dlt header & payload.
    pub fn from_slice(slice: &'a [u8]) -> Result<DltPacketSlice<'_>, ReadError> {
        if slice.len() < 4 {
            return Err(ReadError::UnexpectedEndOfSlice(
                error::UnexpectedEndOfSliceError{
                    layer: error::Layer::DltHeader, 
                    minimum_size: 4,
                    actual_size: slice.len(),
                }
            ));
        }

        let length = u16::from_be_bytes(
            // SAFETY:
            // Safe as it is checked beforehand that the slice
            // has at least 4 bytes.
            unsafe { [*slice.get_unchecked(2), *slice.get_unchecked(3)] },
        ) as usize;

        if slice.len() < length {
            return Err(ReadError::UnexpectedEndOfSlice(
                error::UnexpectedEndOfSliceError{
                    layer: error::Layer::DltHeader,
                    minimum_size: length,
                    actual_size: slice.len(),
                }
            ));
        }

        // calculate the minimum size based on the header flags
        //
        // SAFETY:
        // Safe as it is checked beforehand that the slice
        // has at least 4 bytes.
        let header_type = unsafe { slice.get_unchecked(0) };

        //the header size has at least 4 bytes
        let header_len = if 0 != header_type & ECU_ID_FLAG {
            4 + 4
        } else {
            4
        };

        let header_len = if 0 != header_type & SESSION_ID_FLAG {
            header_len + 4
        } else {
            header_len
        };

        let header_len = if 0 != header_type & TIMESTAMP_FLAG {
            header_len + 4
        } else {
            header_len
        };

        let header_len = if 0 != header_type & EXTDENDED_HEADER_FLAG {
            header_len + 10
        } else {
            header_len
        };

        //the minimum size is composed out of the header size
        // + the minimum size for the payload (4 for message id in non verbose
        // or 4 for the typeinfo in verbose)
        if length < header_len + 4 {
            return Err(ReadError::LengthSmallerThenMinimum {
                required_length: header_len + 4,
                length,
            });
        }

        //looks ok -> create the DltPacketSlice
        Ok(DltPacketSlice {
            // SAFETY:
            // Safe as it is checked beforehand that the slice
            // has at least length bytes.
            slice: unsafe { from_raw_parts(slice.as_ptr(), length) },
            header_len,
        })
    }

    ///Returns if an extended header is present.
    #[inline]
    pub fn has_extended_header(&self) -> bool {
        // SAFETY:
        // Safe as it is checked in from_slice that the slice
        // has at least a length of 4 bytes.
        0 != unsafe { self.slice.get_unchecked(0) } & 0b1
    }

    ///Returns if the numbers in the payload are encoded in big endian.
    #[inline]
    pub fn is_big_endian(&self) -> bool {
        // SAFETY:
        // Safe as it is checked in from_slice that the slice
        // has at least a length of 4 bytes.
        0 != unsafe { self.slice.get_unchecked(0) } & 0b10
    }

    ///Returns if the dlt package is verbose or non verbose.
    #[inline]
    pub fn is_verbose(&self) -> bool {
        if self.has_extended_header() {
            // SAFETY:
            // Safe as if the extended header is present the
            // header_len is checked in from_slice to be at least
            // 10 bytes.
            0 != unsafe { self.slice.get_unchecked(self.header_len - 10) } & 0b1
        } else {
            false
        }
    }

    ///Returns the dlt extended header if present
    #[inline]
    pub fn extended_header(&self) -> Option<DltExtendedHeader> {
        if self.has_extended_header() {
            // SAFETY:
            // Safe as if the extended header is present the
            // header_len is set in from_slice to be at least
            // 10 bytes and also checked against the slice length.
            unsafe {
                let ext_slice = from_raw_parts(self.slice.as_ptr().add(self.header_len - 10), 10);
                Some(DltExtendedHeader {
                    message_info: *ext_slice.get_unchecked(0),
                    number_of_arguments: *ext_slice.get_unchecked(1),
                    application_id: [
                        *ext_slice.get_unchecked(2),
                        *ext_slice.get_unchecked(3),
                        *ext_slice.get_unchecked(4),
                        *ext_slice.get_unchecked(5),
                    ],
                    context_id: [
                        *ext_slice.get_unchecked(6),
                        *ext_slice.get_unchecked(7),
                        *ext_slice.get_unchecked(8),
                        *ext_slice.get_unchecked(9),
                    ],
                })
            }
        } else {
            None
        }
    }

    ///Returns the message type if a parsable message type is present
    #[inline]
    pub fn message_type(&self) -> Option<DltMessageType> {
        if self.has_extended_header() {
            DltMessageType::from_byte(
                // SAFETY:
                // Safe as if the extended header is present the
                // header_len is set in from_slice to be at least
                // 10 bytes and also checked against the slice length.
                unsafe { *self.slice.get_unchecked(self.header_len - 10) },
            )
        } else {
            None
        }
    }

    ///Returns the message id if the message is a non verbose message otherwise None is returned.
    #[inline]
    pub fn message_id(&self) -> Option<u32> {
        if self.is_verbose() {
            None
        } else {
            // SAFETY:
            // Safe as the slice len is checked to be at least
            // header_len + 4 in from_slice.
            let id_bytes = unsafe {
                [
                    *self.slice.get_unchecked(self.header_len),
                    *self.slice.get_unchecked(self.header_len + 1),
                    *self.slice.get_unchecked(self.header_len + 2),
                    *self.slice.get_unchecked(self.header_len + 3),
                ]
            };
            if self.is_big_endian() {
                Some(u32::from_be_bytes(id_bytes))
            } else {
                Some(u32::from_le_bytes(id_bytes))
            }
        }
    }

    ///Returns the slice containing the dlt header + payload.
    #[inline]
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }

    ///Returns a slice containing the payload of the dlt message
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        // SAFETY:
        // Safe as the slice len is checked to be at least
        // header_len + 4 in from_slice.
        unsafe {
            from_raw_parts(
                self.slice.as_ptr().add(self.header_len),
                self.slice.len() - self.header_len,
            )
        }
    }

    ///Returns a slice containing the payload of a non verbose message (after the message id).
    pub fn non_verbose_payload(&self) -> &'a [u8] {
        // SAFETY:
        // Safe as the slice len is checked to be at least
        // header_len + 4 in from_slice.
        unsafe {
            from_raw_parts(
                self.slice.as_ptr().add(self.header_len + 4),
                self.slice.len() - self.header_len - 4,
            )
        }
    }

    ///Deserialize the dlt header
    pub fn header(&self) -> DltHeader {
        // SAFETY:
        // Safe as it is checked in from_slice that the slice
        // has at least a length of 4 bytes.
        let header_type = unsafe { *self.slice.get_unchecked(0) };
        let (is_big_endian, version) = {
            (
                0 != header_type & BIG_ENDIAN_FLAG,
                (header_type >> 5) & MAX_VERSION,
            )
        };
        // SAFETY:
        // Safe as it is checked in from_slice that the slice
        // has at least a length of 4 bytes.
        let message_counter = unsafe { *self.slice.get_unchecked(1) };
        let length = u16::from_be_bytes(
            // SAFETY:
            // Safe as it is checked in from_slice that the slice
            // has at least the length of 4 bytes.
            unsafe { [*self.slice.get_unchecked(2), *self.slice.get_unchecked(3)] },
        );

        let (ecu_id, slice) = if 0 != header_type & ECU_ID_FLAG {
            (
                Some(u32::from_be_bytes(
                    // SAFETY:
                    // Safe as it is checked in from_slice that the slice
                    // has the length to contain the standard & extended header
                    // based on the flags contained in the standard header.
                    unsafe {
                        [
                            *self.slice.get_unchecked(4),
                            *self.slice.get_unchecked(5),
                            *self.slice.get_unchecked(6),
                            *self.slice.get_unchecked(7),
                        ]
                    },
                )),
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has the length to contain the standard & extended header
                // based on the flags contained in the standard header.
                unsafe { from_raw_parts(self.slice.as_ptr().add(8), self.slice.len() - 8) },
            )
        } else {
            (
                None,
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has at least the length of 4 bytes.
                unsafe {
                    // go after the standard header base
                    from_raw_parts(self.slice.as_ptr().add(4), self.slice.len() - 4)
                },
            )
        };

        let (session_id, slice) = if 0 != header_type & SESSION_ID_FLAG {
            (
                Some(u32::from_be_bytes(
                    // SAFETY:
                    // Safe as it is checked in from_slice that the slice
                    // has the length to contain the standard & extended header
                    // based on the flags contained in the standard header.
                    unsafe {
                        [
                            *slice.get_unchecked(0),
                            *slice.get_unchecked(1),
                            *slice.get_unchecked(2),
                            *slice.get_unchecked(3),
                        ]
                    },
                )),
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has the length to contain the standard & extended header
                // based on the flags contained in the standard header.
                unsafe { from_raw_parts(slice.as_ptr().add(4), slice.len() - 4) },
            )
        } else {
            (None, slice)
        };

        let (timestamp, slice) = if 0 != header_type & TIMESTAMP_FLAG {
            (
                Some(u32::from_be_bytes(
                    // SAFETY:
                    // Safe as it is checked in from_slice that the slice
                    // has the length to contain the standard & extended header
                    // based on the flags contained in the standard header.
                    unsafe {
                        [
                            *slice.get_unchecked(0),
                            *slice.get_unchecked(1),
                            *slice.get_unchecked(2),
                            *slice.get_unchecked(3),
                        ]
                    },
                )),
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has the length to contain the standard & extended header
                // based on the flags contained in the standard header.
                unsafe { from_raw_parts(slice.as_ptr().add(4), slice.len() - 4) },
            )
        } else {
            (None, slice)
        };

        let extended_header = if 0 != header_type & EXTDENDED_HEADER_FLAG {
            Some(DltExtendedHeader {
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has the length to contain the standard & extended header
                // based on the flags contained in the standard header.
                message_info: unsafe { *slice.get_unchecked(0) },
                number_of_arguments: unsafe { *slice.get_unchecked(1) },
                application_id: unsafe {
                    [
                        *slice.get_unchecked(2),
                        *slice.get_unchecked(3),
                        *slice.get_unchecked(4),
                        *slice.get_unchecked(5),
                    ]
                },
                context_id: unsafe {
                    [
                        *slice.get_unchecked(6),
                        *slice.get_unchecked(7),
                        *slice.get_unchecked(8),
                        *slice.get_unchecked(9),
                    ]
                },
            })
        } else {
            None
        };

        DltHeader {
            is_big_endian,
            version,
            message_counter,
            length,
            ecu_id,
            session_id,
            timestamp,
            extended_header,
        }
    }
}

///Allows iterating over the someip message in a udp or tcp payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceIterator<'a> {
    slice: &'a [u8],
}

impl<'a> SliceIterator<'a> {
    #[inline]
    pub fn new(slice: &'a [u8]) -> SliceIterator<'a> {
        SliceIterator { slice }
    }
}

impl<'a> Iterator for SliceIterator<'a> {
    type Item = Result<DltPacketSlice<'a>, ReadError>;

    #[inline]
    fn next(&mut self) -> Option<Result<DltPacketSlice<'a>, ReadError>> {
        if !self.slice.is_empty() {
            //parse
            let result = DltPacketSlice::from_slice(self.slice);

            //move the slice depending on the result
            match &result {
                Err(_) => {
                    //error => move the slice to an len = 0 position so that the iterator ends
                    let len = self.slice.len();
                    self.slice = &self.slice[len..];
                }
                Ok(ref value) => {
                    //by the length just taken by the slice
                    self.slice = &self.slice[value.slice().len()..];
                }
            }

            //return parse result
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use proptest::prelude::*;
    use crate::proptest_generators::*;
    use super::*;
    use std::io::Cursor;
    use std::io::Write;

    mod dlt_header {

        use super::*;

        proptest! {
            #[test]
            fn write_read(ref dlt_header in dlt_header_any()) {
                let mut buffer = Vec::new();
                dlt_header.write(&mut buffer).unwrap();
                let mut reader = Cursor::new(&buffer[..]);
                let result = DltHeader::read(&mut reader).unwrap();
                assert_eq!(dlt_header, &result);
            }
        }

        proptest! {
            #[test]
            fn read_length_error(ref dlt_header in dlt_header_any()) {
                let mut buffer = Vec::new();
                dlt_header.write(&mut buffer).unwrap();
                let reduced_len = buffer.len() - 1;
                let mut reader = Cursor::new(&buffer[..reduced_len]);
                assert_matches!(DltHeader::read(&mut reader), Err(ReadError::IoError(_)));
            }
        }

        proptest! {
            #[test]
            fn write_version_error(ref dlt_header in dlt_header_any(),
                                   version in MAX_VERSION+1..std::u8::MAX) {
                let mut input = dlt_header.clone();
                input.version = version;
                let mut buffer = Vec::new();
                assert_matches!(input.write(&mut buffer), Err(WriteError::VersionTooLarge(_)));
            }
        }

        proptest! {
            #[test]
            fn write_io_error(ref header in dlt_header_any()) {
                let mut buffer: Vec<u8> = Vec::with_capacity(
                    header.header_len().into()
                );
                for len in 0..header.header_len() {
                    buffer.resize(len.into(), 0);
                    let mut writer = Cursor::new(&mut buffer[..]);
                    assert_matches!(header.write(&mut writer), Err(WriteError::IoError(_)));
                }
            }
        }

        #[test]
        fn is_verbose() {
            let mut header: DltHeader = Default::default();
            assert_eq!(false, header.is_verbose());
            //add an extended header without the verbose flag
            header.extended_header = Some(Default::default());
            assert_eq!(false, header.is_verbose());
            //set the verbose flag
            header
                .extended_header
                .as_mut()
                .unwrap()
                .set_is_verbose(true);
            assert_eq!(true, header.is_verbose());
        }

        #[test]
        fn header_len() {
            struct Test {
                expected: u16,
                ecu_id: Option<u32>,
                session_id: Option<u32>,
                timestamp: Option<u32>,
                extended_header: Option<DltExtendedHeader>,
            }

            let tests = [
                Test {
                    expected: 4,
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                },
                Test {
                    expected: 4 + 4 + 4 + 4 + 10,
                    ecu_id: Some(0),
                    session_id: Some(0),
                    timestamp: Some(0),
                    extended_header: Some(Default::default()),
                },
                Test {
                    expected: 4 + 4,
                    ecu_id: Some(0),
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                },
                Test {
                    expected: 4 + 4,
                    ecu_id: None,
                    session_id: Some(0),
                    timestamp: None,
                    extended_header: None,
                },
                Test {
                    expected: 4 + 4,
                    ecu_id: None,
                    session_id: None,
                    timestamp: Some(0),
                    extended_header: None,
                },
                Test {
                    expected: 4 + 10,
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: Some(Default::default()),
                },
            ];

            for test in tests {
                assert_eq!(
                    test.expected,
                    DltHeader {
                        is_big_endian: false,
                        version: MAX_VERSION,
                        message_counter: 123,
                        length: 123,
                        ecu_id: test.ecu_id,
                        session_id: test.session_id,
                        timestamp: test.timestamp,
                        extended_header: test.extended_header,
                    }
                    .header_len()
                );
            }
        }

        #[test]
        fn debug() {
            let header: DltHeader = Default::default();
            println!("{:?}", header);
        }

        proptest! {
            #[test]
            fn clone_eq(ref header in dlt_header_any()) {
                assert_eq!(*header, header.clone());
            }
        }

        #[test]
        fn default() {
            let header: DltHeader = Default::default();
            assert_eq!(header.is_big_endian, false);
            assert_eq!(header.version, 0);
            assert_eq!(header.message_counter, 0);
            assert_eq!(header.length, 0);
            assert_eq!(header.ecu_id, None);
            assert_eq!(header.session_id, None);
            assert_eq!(header.timestamp, None);
            assert_eq!(header.extended_header, None);
        }
    } // mod dlt_header

    /// Tests for `DltPacketSlice` methods
    mod dlt_packet_slice {

        use super::*;

        #[test]
        fn debug() {
            let mut header: DltHeader = Default::default();
            header.length = header.header_len() + 4;
            let mut buffer = Vec::with_capacity(usize::from(header.length));
            header.write(&mut buffer).unwrap();
            buffer.extend_from_slice(&[0, 0, 0, 0]);
            let slice = DltPacketSlice::from_slice(&buffer).unwrap();
            println!("{:?}", slice);
        }

        proptest! {
            #[test]
            fn clone_eq_debug(ref packet in dlt_header_with_payload_any()) {
                let mut buffer = Vec::with_capacity(
                    usize::from(packet.0.length)
                );
                packet.0.write(&mut buffer).unwrap();
                buffer.extend_from_slice(&packet.1);
                let slice = DltPacketSlice::from_slice(&buffer).unwrap();

                // clone & eq
                assert_eq!(slice, slice.clone());
            }
        }

        proptest! {
            #[test]
            fn from_slice(
                ref packet in dlt_header_with_payload_any()
            ) {
                let mut buffer = Vec::with_capacity(
                    packet.1.len() + usize::from(packet.0.header_len())
                );
                packet.0.write(&mut buffer).unwrap();
                buffer.write(&packet.1[..]).unwrap();
                //read the slice
                let slice = DltPacketSlice::from_slice(&buffer[..]).unwrap();
                //check the results are matching the input
                assert_eq!(slice.header(), packet.0);
                assert_eq!(slice.has_extended_header(), packet.0.extended_header.is_some());
                assert_eq!(slice.is_big_endian(), packet.0.is_big_endian);
                assert_eq!(slice.is_verbose(), packet.0.is_verbose());
                assert_eq!(slice.payload(), &packet.1[..]);
                assert_eq!(slice.extended_header(), packet.0.extended_header);
                assert_eq!(slice.non_verbose_payload(), &packet.1[4..]);

                if let Some(packet_ext_header) = packet.0.extended_header.as_ref() {
                    assert_eq!(slice.message_type(), packet_ext_header.message_type());
                    assert_eq!(slice.header().extended_header.unwrap().message_type(),
                               packet.0.extended_header.as_ref().unwrap().message_type());
                } else {
                    assert_eq!(slice.header().extended_header, None);
                    assert_eq!(slice.message_type(), None);
                }

                //check that a too small slice produces an error
                for len in 0..buffer.len() - 1 {
                    assert_matches!(
                        DltPacketSlice::from_slice(&buffer[..len]),
                        Err(
                            ReadError::UnexpectedEndOfSlice(
                                error::UnexpectedEndOfSliceError {
                                    layer: error::Layer::DltHeader,
                                    minimum_size: _,
                                    actual_size: _,
                                }
                            )
                        )
                    );
                }
            }
        }

        #[test]
        fn from_slice_header_len_eof_errors() {
            //too small for header
            {
                let buffer = [1, 2, 3];
                assert_matches!(
                    DltPacketSlice::from_slice(&buffer[..]),
                    Err(ReadError::UnexpectedEndOfSlice(
                        error::UnexpectedEndOfSliceError {
                            layer: error::Layer::DltHeader,
                            minimum_size: 4,
                            actual_size: 3,
                        }
                    ))
                );
            }
            //too small for the length
            {
                let mut header: DltHeader = Default::default();
                header.length = 5;
                let mut buffer = Vec::new();
                header.write(&mut buffer).unwrap();
                assert_matches!(
                    DltPacketSlice::from_slice(&buffer[..]),
                    Err(ReadError::UnexpectedEndOfSlice(
                        error::UnexpectedEndOfSliceError {
                            layer: error::Layer::DltHeader,
                            minimum_size: 5,
                            actual_size: 4,
                        }
                    ))
                );
            }
        }

        proptest! {
            #[test]
            fn from_slice_header_variable_len_eof_errors(ref input in dlt_header_any()) {
                let mut header = input.clone();
                header.length = header.header_len() + 3; //minimum payload size is 4
                let mut buffer = Vec::new();
                header.write(&mut buffer).unwrap();
                buffer.write(&[1,2,3]).unwrap();
                assert_matches!(DltPacketSlice::from_slice(&buffer[..]), Err(ReadError::LengthSmallerThenMinimum{required_length: _, length: _}));
            }
        }

        #[test]
        fn message_id() {
            //pairs of (header, expected_some)
            let tests = [
                //verbose (does not have message id)
                (
                    {
                        let mut header: DltHeader = Default::default();
                        header.extended_header = Some({
                            let mut ext: DltExtendedHeader = Default::default();
                            ext.set_is_verbose(true);
                            ext
                        });
                        header
                    },
                    false,
                ),
                //with extended header non-verbose
                (
                    {
                        let mut header: DltHeader = Default::default();
                        header.extended_header = Some({
                            let mut ext: DltExtendedHeader = Default::default();
                            ext.set_is_verbose(false);
                            ext
                        });
                        header
                    },
                    true,
                ),
                //without extended header (always non verbose)
                (
                    {
                        let mut header: DltHeader = Default::default();
                        header.extended_header = None;
                        header
                    },
                    true,
                ),
            ];
            //verbose (does not have message id)
            for t in tests.iter() {
                //big endian
                {
                    let header = {
                        let mut header = t.0.clone();
                        header.is_big_endian = true;
                        header.length = header.header_len() + 4;
                        header
                    };

                    //serialize
                    let mut buffer = Vec::<u8>::new();
                    header.write(&mut buffer).unwrap();
                    buffer.write_all(&0x1234_5678u32.to_be_bytes()).unwrap();

                    //slice
                    let slice = DltPacketSlice::from_slice(&buffer).unwrap();
                    assert_eq!(
                        slice.message_id(),
                        if t.1 { Some(0x1234_5678) } else { None }
                    );
                }

                //little endian
                {
                    let header = {
                        let mut header = t.0.clone();
                        header.is_big_endian = false;
                        header.length = header.header_len() + 4;
                        header
                    };

                    //serialize
                    let mut buffer = Vec::<u8>::new();
                    header.write(&mut buffer).unwrap();
                    buffer.write_all(&0x1234_5678u32.to_le_bytes()).unwrap();

                    //slice
                    let slice = DltPacketSlice::from_slice(&buffer).unwrap();
                    assert_eq!(
                        slice.message_id(),
                        if t.1 { Some(0x1234_5678) } else { None }
                    );
                }
            }
        }
    } // mod dlt_packet_slice

    /// Tests for `SliceIterator`
    mod slice_interator {

        use super::*;

        #[test]
        fn clone_eq() {
            let it = SliceIterator { slice: &[] };
            assert_eq!(it, it.clone());
        }

        #[test]
        fn debug() {
            let it = SliceIterator { slice: &[] };
            println!("{:?}", it);
        }

        proptest! {
            #[test]
            fn iterator(ref packets in prop::collection::vec(dlt_header_with_payload_any(), 1..5)) {
                //serialize the packets
                let mut buffer = Vec::with_capacity(
                    (*packets).iter().fold(0, |acc, x| acc + usize::from(x.0.header_len()) + x.1.len())
                );

                let mut offsets: Vec<(usize, usize)> = Vec::with_capacity(packets.len());

                for packet in packets {

                    //save the start for later processing
                    let start = buffer.len();

                    //header & payload
                    packet.0.write(&mut buffer).unwrap();
                    buffer.write_all(&packet.1).unwrap();

                    //safe the offset for later
                    offsets.push((start, buffer.len()));
                }

                //determine the expected output
                let mut expected: Vec<DltPacketSlice<'_>> = Vec::with_capacity(packets.len());
                for offset in &offsets {
                    //create the expected slice
                    let slice = &buffer[offset.0..offset.1];
                    let e = DltPacketSlice::from_slice(slice).unwrap();
                    assert_eq!(e.slice(), slice);
                    expected.push(e);
                }

                //iterate over packets
                assert_eq!(expected, SliceIterator::new(&buffer).map(|x| x.unwrap()).collect::<Vec<DltPacketSlice<'_>>>());

                //check for error return when the slice is too small
                //first entry
                {
                    let o = offsets.first().unwrap();
                    let mut it = SliceIterator::new(&buffer[..(o.1 - 1)]);

                    assert_matches!(it.next(), Some(Err(ReadError::UnexpectedEndOfSlice(_))));
                    //check that the iterator does not continue
                    assert_matches!(it.next(), None);
                }
                //last entry
                {
                    let o = offsets.last().unwrap();
                    let it = SliceIterator::new(&buffer[..(o.1 - 1)]);
                    let mut it = it.skip(offsets.len()-1);

                    assert_matches!(it.next(), Some(Err(ReadError::UnexpectedEndOfSlice(_))));
                    //check that the iterator does not continue
                    assert_matches!(it.next(), None);
                }
            }
        }
    } // mod slice_iterator

    /// Tests for `DltExtendedHeader` methods
    mod dlt_extended_header {

        use super::*;

        #[test]
        fn clone_eq() {
            let header: DltExtendedHeader = Default::default();
            assert_eq!(header, header.clone());
        }

        #[test]
        fn debug() {
            let header: DltExtendedHeader = Default::default();
            println!("{:?}", header);
        }

        #[test]
        fn default() {
            let header: DltExtendedHeader = Default::default();
            assert_eq!(header.message_info, 0);
            assert_eq!(header.number_of_arguments, 0);
            assert_eq!(header.application_id, [0, 0, 0, 0]);
            assert_eq!(header.context_id, [0, 0, 0, 0]);
        }

        proptest! {
            #[test]
            fn new_non_verbose_log(
                log_level in log_level_any(),
                application_id in any::<[u8;4]>(),
                context_id in any::<[u8;4]>())
            {
                use DltMessageType::Log;
                let header = DltExtendedHeader::new_non_verbose_log(log_level.clone(), application_id, context_id);
                assert_eq!(Log(log_level).to_byte().unwrap(), header.message_info);
                assert_eq!(0, header.number_of_arguments);
                assert_eq!(application_id, header.application_id);
                assert_eq!(context_id, header.context_id);
            }
        }

        proptest! {
            #[test]
            fn new_non_verbose(
                message_type in message_type_any(),
                application_id in any::<[u8;4]>(),
                context_id in any::<[u8;4]>(),
                invalid_user_defined in 0x10..0xffu8
            ) {
                // valid data
                {
                    let header = DltExtendedHeader::new_non_verbose(
                        message_type.clone(),
                        application_id,
                        context_id
                    ).unwrap();
                    assert_eq!(message_type.to_byte().unwrap(), header.message_info);
                    assert_eq!(0, header.number_of_arguments);
                    assert_eq!(application_id, header.application_id);
                    assert_eq!(context_id, header.context_id);
                }

                // invalid data
                {
                    use DltMessageType::NetworkTrace;
                    use DltNetworkType::UserDefined;
                    use RangeError::NetworkTypekUserDefinedOutsideOfRange;

                    let result = DltExtendedHeader::new_non_verbose(
                        NetworkTrace(UserDefined(invalid_user_defined)),
                        application_id,
                        context_id
                    ).unwrap_err();
                    assert_eq!(NetworkTypekUserDefinedOutsideOfRange(invalid_user_defined), result);
                }
            }
        }

        #[test]
        fn set_is_verbose() {
            let mut header: DltExtendedHeader = Default::default();
            let original = header.clone();
            header.set_is_verbose(true);
            assert_eq!(true, header.is_verbose());
            header.set_is_verbose(false);
            assert_eq!(false, header.is_verbose());
            assert_eq!(original, header);
        }

        proptest! {
            #[test]
            fn set_message_type(
                verbose in any::<bool>(),
                message_type0 in message_type_any(),
                message_type1 in message_type_any())
            {
                let mut header: DltExtendedHeader = Default::default();

                //set verbose (stored in same field, to ensure no side effects)
                header.set_is_verbose(verbose);
                assert_eq!(header.is_verbose(), verbose);

                //set to first message type
                header.set_message_type(message_type0.clone()).unwrap();
                assert_eq!(header.is_verbose(), verbose);
                assert_eq!(header.message_type(), Some(message_type0));

                //set to second message type (to make sure the old type is correctly cleaned)
                header.set_message_type(message_type1.clone()).unwrap();
                assert_eq!(header.is_verbose(), verbose);
                assert_eq!(header.message_type(), Some(message_type1));
            }
        }

        #[test]
        fn message_type() {
            use {
                DltControlMessageType::*, DltLogLevel::*, DltMessageType::*, DltNetworkType::*,
                DltTraceType::*,
            };

            //check that setting & resetting does correctly reset the values
            {
                let mut header = DltExtendedHeader::new_non_verbose_log(
                    Fatal,
                    Default::default(),
                    Default::default(),
                );

                header.set_message_type(NetworkTrace(SomeIp)).unwrap();
                assert_eq!(false, header.is_verbose());
                assert_eq!(Some(NetworkTrace(SomeIp)), header.message_type());

                //set to a different value with non overlapping bits (to make sure the values are reset)
                header.set_message_type(Trace(FunctionIn)).unwrap();
                assert_eq!(false, header.is_verbose());
                assert_eq!(Some(Trace(FunctionIn)), header.message_type());
            }

            //check None return type when a unknown value is presented
            //message type
            for message_type_id in 4..=0b111 {
                let mut header = DltExtendedHeader::new_non_verbose_log(
                    Fatal,
                    Default::default(),
                    Default::default(),
                );
                header.message_info = message_type_id << 1;
                assert_eq!(None, header.message_type());
            }

            //msin bad values
            let bad_values = [
                //bad log level 0 & everything above 6
                (Log(Fatal), (0u8..1).chain(7u8..=0xf)),
                //bad trace source (0 & everything above 5)
                (Trace(FunctionIn), (0u8..1).chain(6u8..=0xf)),
                //bad control message type (0 & everything above 2)
                (Control(Request), (0u8..1).chain(3u8..=0xf)),
            ];

            for t in bad_values.iter() {
                for value in t.1.clone() {
                    let mut header = DltExtendedHeader::new_non_verbose(
                        t.0.clone(),
                        Default::default(),
                        Default::default(),
                    )
                    .unwrap();
                    header.message_info &= 0b0000_1111;
                    header.message_info |= value << 4;
                    assert_eq!(None, header.message_type());
                }
            }

            //check set out of range error
            {
                use DltLogLevel::Fatal;
                use RangeError::*;
                for i in 0x10..=0xff {
                    let mut header = DltExtendedHeader::new_non_verbose_log(
                        Fatal,
                        Default::default(),
                        Default::default(),
                    );
                    assert_eq!(
                        Err(NetworkTypekUserDefinedOutsideOfRange(i)),
                        header.set_message_type(NetworkTrace(UserDefined(i)))
                    );
                }
            }
        }
    } // mod dlt_extended_header

    /// Tests for `ReadError` methods
    mod read_error {

        use super::*;

        #[test]
        fn debug() {
            use crate::ReadError::*;
            for value in [
                UnexpectedEndOfSlice(
                    error::UnexpectedEndOfSliceError {
                        minimum_size: 1,
                        actual_size: 2,
                        layer: error::Layer::DltHeader,
                    }
                ),
                LengthSmallerThenMinimum {
                    required_length: 3,
                    length: 4,
                },
                IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!")),
            ]
            .iter()
            {
                println!("{:?}", value);
            }
        }

        proptest! {
            #[test]
            fn display(
                usize0 in any::<usize>(),
                usize1 in any::<usize>(),
            ) {

                use ReadError::*;

                //UnexpectedEndOfSlice
                assert_eq!(
                    &format!("ReadError: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.", usize1, usize0),
                    &format!(
                        "{}",
                        UnexpectedEndOfSlice(
                            error::UnexpectedEndOfSliceError {
                                layer: error::Layer::DltHeader,
                                minimum_size: usize0,
                                actual_size: usize1,
                            }
                        )
                    )
                );

                //UnexpectedEndOfSlice
                assert_eq!(
                    &format!("ReadError: The length of {} in the dlt header is smaller then minimum required size of {} bytes.", usize1, usize0),
                    &format!("{}", LengthSmallerThenMinimum { required_length: usize0, length: usize1 })
                );

                //IoError
                {
                    let custom_error = std::io::Error::new(std::io::ErrorKind::Other, "some error");
                    assert_eq!(
                        &format!("{}", custom_error),
                        &format!("{}", IoError(custom_error))
                    );
                }
            }
        }

        #[test]
        fn source() {
            use crate::ReadError::*;
            use std::error::Error;

            assert!(UnexpectedEndOfSlice(
                error::UnexpectedEndOfSliceError {
                    layer: error::Layer::DltHeader,
                    minimum_size: 1,
                    actual_size: 2
                }
            )
            .source()
            .is_none());
            assert!(LengthSmallerThenMinimum {
                required_length: 3,
                length: 4
            }
            .source()
            .is_none());
            assert!(
                IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
                    .source()
                    .is_some()
            );
        }
    } // mod read_error

    /// Tests for `WriteError` methods
    mod write_error {

        use super::*;

        #[test]
        fn debug() {
            use WriteError::*;
            for value in [
                VersionTooLarge(123),
                IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!")),
            ]
            .iter()
            {
                println!("{:?}", value);
            }
        }

        proptest! {
            #[test]
            fn display(version in any::<u8>()) {
                use WriteError::*;

                // VersionTooLarge
                assert_eq!(
                    &format!("WriteError: DLT version {} is larger then the maximum supported value of {}", version, MAX_VERSION),
                    &format!("{}", VersionTooLarge(version))
                );

                //IoError
                {
                    let custom_error = std::io::Error::new(std::io::ErrorKind::Other, "some error");
                    assert_eq!(
                        &format!("{}", custom_error),
                        &format!("{}", IoError(custom_error))
                    );
                }
            }
        }

        #[test]
        fn source() {
            use std::error::Error;
            use WriteError::*;

            assert!(VersionTooLarge(123).source().is_none());
            assert!(
                IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
                    .source()
                    .is_some()
            );
        }
    } // mod write_error

    mod range_error {

        use super::*;

        #[test]
        fn clone_eq() {
            use RangeError::*;
            let v = NetworkTypekUserDefinedOutsideOfRange(123);
            assert_eq!(v, v.clone());
        }

        #[test]
        fn debug() {
            use RangeError::*;
            println!("{:?}", NetworkTypekUserDefinedOutsideOfRange(123));
        }

        proptest! {
            #[test]
            fn display(value in any::<u8>()) {
                use RangeError::*;

                // NetworkTypekUserDefinedOutsideOfRange
                assert_eq!(
                    &format!("RangeError: Message type info field user defined value of {} outside of the allowed range of 7-15.", value),
                    &format!("{}", NetworkTypekUserDefinedOutsideOfRange(value))
                );
            }
        }

        #[test]
        fn source() {
            use std::error::Error;
            use RangeError::*;

            assert!(NetworkTypekUserDefinedOutsideOfRange(123)
                .source()
                .is_none());
        }
    } // mod range_error

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
            use RangeError::NetworkTypekUserDefinedOutsideOfRange;
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
