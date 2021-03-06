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
//! dlt_parse = "0.2.1"
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
//! * Methods for parsing verbose DLT packets are still missing (but maybe implemented in future versions).
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
//!             123,//application id
//!             1,//context id
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
//!     //byteorder crate is used for writing the message id with  the correct endianess
//!     use byteorder::{BigEndian, WriteBytesExt};
//!
//!     //write the message id & payload
//!     buffer.write_u32::<BigEndian>(1234).unwrap(); //message id
//!     buffer.write_all(&[1,2,3,4]); //payload
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

use std::io;

use byteorder;
use self::byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(test)]
extern crate proptest;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

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
    pub extended_header: Option<DltExtendedHeader>
}

///Errors that can occure on reading a dlt header.
#[derive(Debug)]
pub enum ReadError {
    ///Error if the slice is smaller then dlt length field or minimal size.
    UnexpectedEndOfSlice { minimum_size: usize, actual_size: usize},
    ///Error if the dlt length is smaller then the header the calculated header size based on the flags (+ minimum payload size of 4 bytes/octetets)
    LengthSmallerThenMinimum { required_length: usize, length: usize },
    ///Standard io error.
    IoError(io::Error)
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::IoError(err)
    }
}

///Errors that can occur when serializing a dlt header.
#[derive(Debug)]
pub enum WriteError {
    VersionTooLarge(u8),
    IoError(io::Error)
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> WriteError {
        WriteError::IoError(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RangeError {
    ///Error if the user defined network type is greater then 0xf
    NetworkTypekUserDefinedTooLarge(u8)
}

const MAX_VERSION: u8 = 0b111;

const EXTDENDED_HEADER_FLAG: u8 = 0b1;
const BIG_ENDIAN_FLAG: u8 = 0b10;
const ECU_ID_FLAG: u8     = 0b100;
const SESSION_ID_FLAG: u8 = 0b1000;
const TIMESTAMP_FLAG: u8  = 0b10000;

///Shifted value in the msin extended header field for dlt "log" messages.
const EXT_MSIN_MSTP_TYPE_LOG: u8 = 0x0 << 1;
///Shifted value in the msin extended header field for dlt "trace" messages.
const EXT_MSIN_MSTP_TYPE_TRACE: u8 = 0x1 << 1;
///Shifted value in the msin extended header field for dlt "network trace" messages.
const EXT_MSIN_MSTP_TYPE_NW_TRACE: u8 = 0x2 << 1;
///Shifted value in the msin extended header field for dlt "control" messages.
const EXT_MSIN_MSTP_TYPE_CONTROL: u8 = 0x3 << 1;

impl DltHeader {

    ///Deserialize a DltHeader & TpHeader from the given reader.
    pub fn read<T: io::Read + Sized>(reader: &mut T) -> Result<DltHeader, ReadError> {
        //first lets read the header type
        let header_type = reader.read_u8()?;
        //let extended_header = 0 != header_type & EXTDENDED_HEADER_FLAG;
        Ok(DltHeader{
            is_big_endian: 0 != header_type & BIG_ENDIAN_FLAG,
            version: (header_type >> 5) & MAX_VERSION,
            message_counter: reader.read_u8()?,
            length: reader.read_u16::<BigEndian>()?,
            ecu_id: if 0 != header_type & ECU_ID_FLAG {
                Some(reader.read_u32::<BigEndian>()?)
            } else {
                None
            },
            session_id: if 0 != header_type & SESSION_ID_FLAG {
                Some(reader.read_u32::<BigEndian>()?)
            } else {
                None
            },
            timestamp: if 0 != header_type & TIMESTAMP_FLAG {
                Some(reader.read_u32::<BigEndian>()?)
            } else {
                None
            },
            extended_header: if 0 != header_type & EXTDENDED_HEADER_FLAG {
                Some(DltExtendedHeader{
                    message_info: reader.read_u8()?,
                    number_of_arguments: reader.read_u8()?,
                    application_id: reader.read_u32::<BigEndian>()?,
                    context_id: reader.read_u32::<BigEndian>()?
                })
            } else {
                None
            }
        })
    }

    ///Serializes the header to the given writer.
    pub fn write<T: io::Write + Sized>(&self, writer: &mut T) -> Result<(), WriteError> {
        //pre check if the ranges of all fields are valid
        if self.version > MAX_VERSION {
            return Err(WriteError::VersionTooLarge(self.version))
        }

        //create the header type bitfield
        writer.write_u8({
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
        })?;
        //write the rest of the standard header fields
        writer.write_u8(self.message_counter)?;
        writer.write_u16::<BigEndian>(self.length)?;

        if let Some(value) = self.ecu_id { 
            writer.write_u32::<BigEndian>(value)?;
        }

        if let Some(value) = self.session_id {
            writer.write_u32::<BigEndian>(value)?;
        }

        if let Some(value) = self.timestamp {
            writer.write_u32::<BigEndian>(value)?;
        }

        //write the extended header if it exists
        match &self.extended_header {
            Some(value) => {
                writer.write_u8(value.message_info)?;
                writer.write_u8(value.number_of_arguments)?;
                writer.write_u32::<BigEndian>(value.application_id)?;
                writer.write_u32::<BigEndian>(value.context_id)?;
            },
            None => {}
        }
        Ok(())
    }

    ///Returns if the package is a verbose package
    pub fn is_verbose(&self) -> bool {
        match &self.extended_header {
            None => false, //only packages with extended headers can be verbose
            Some(ext) => ext.is_verbose() 
        }
    }

    ///Return the byte/octed size of the serialized header (including extended header)
    pub fn header_len(&self) -> u16 {
        4 + match self.ecu_id {
            Some(_) => 4,
            None => 0
        } + match self.session_id {
            Some(_) => 4,
            None => 0
        } + match self.timestamp {
            Some(_) => 4,
            None => 0
        } + match self.extended_header {
            Some(_) => 10,
            None => 0
        }
    }
}

///Extended dlt header (optional header in the dlt header)
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DltExtendedHeader {
    pub message_info: u8,
    pub number_of_arguments: u8,
    pub application_id: u32,
    pub context_id: u32
}

impl DltExtendedHeader {

    ///Create a extended header for a non verbose log message with given application id & context id.
    pub fn new_non_verbose_log(log_level: DltLogLevel, application_id: u32, context_id: u32) -> DltExtendedHeader {
        DltExtendedHeader {
            message_info: DltMessageType::Log(log_level).encode_to_message_info_byte().unwrap(),
            number_of_arguments: 0,
            application_id,
            context_id
        }
    }

    ///Create a extended header for a non verbose message with given message type, application id & context id.
    pub fn new_non_verbose(message_type: DltMessageType, application_id: u32, context_id: u32) -> Result<DltExtendedHeader, RangeError> {
        Ok(DltExtendedHeader {
            message_info: message_type.encode_to_message_info_byte()?,
            number_of_arguments: 0,
            application_id,
            context_id
        })
    }

    ///Returns true if the extended header flags the message as a verbose message.
    pub fn is_verbose(&self) -> bool {
        0 != self.message_info & 0b1 
    }

    ///Sets or unsets the is_verbose bit in the DltExtendedHeader.
    pub fn set_is_verbose(&mut self, is_verbose: bool) {
        if is_verbose {
            self.message_info |= 0b1;
        } else {
            self.message_info &= 0b1111_1110;
        }
    }

    ///Returns message type info or `Option::None` for reserved values.
    pub fn message_type(&self) -> Option<DltMessageType> {
        DltMessageType::from_message_info_encoded(self.message_info)
    }

    ///Set message type info and based on that the message type.
    pub fn set_message_type(&mut self, value: DltMessageType) -> Result<(),RangeError> {
        
        let encoded = value.encode_to_message_info_byte()?;

        //unset old message type & set the new one
        self.message_info &= 0b0000_0001;
        self.message_info |= encoded;

        //all good
        Ok(())
    }
}

///Log level for dlt log messages.
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DltControlMessageType {
    ///Request control message.
    Request = 0x1,
    ///Respond control message.
    Response = 0x2
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

    ///Attempts to read the message type from the first byte of 
    ///the dlt extended message header.
    fn from_message_info_encoded(value: u8) -> Option<DltMessageType> {
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
            },
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
            },
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
                    other => Some(NetworkTrace(UserDefined(other)))
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
            },
            _ => None
        }
    }

    ///Set message type info and based on that the message type.
    pub fn encode_to_message_info_byte(&self) -> Result<u8,RangeError> {
        use DltMessageType::*;
        use DltNetworkType::UserDefined;
        use RangeError::NetworkTypekUserDefinedTooLarge;
        
        //check ranges
        if let NetworkTrace(UserDefined(user_defined_value)) = *self {
            if user_defined_value > 0xf {
                return Err(NetworkTypekUserDefinedTooLarge(user_defined_value));
            }
        }

        //determine message type & message type info
        let (message_type, message_type_info) = match self {
            Log(ref level) => (
                EXT_MSIN_MSTP_TYPE_LOG,
                level.clone() as u8
            ),
            Trace(ref trace_type) => (
                EXT_MSIN_MSTP_TYPE_TRACE,
                trace_type.clone() as u8
            ),
            NetworkTrace(ref nw_trace_type) => {
                use DltNetworkType::*;

                (EXT_MSIN_MSTP_TYPE_NW_TRACE,
                 match *nw_trace_type {
                    Ipc => 0x1,
                    Can => 0x2,
                    Flexray => 0x3,
                    Most => 0x4,
                    Ethernet => 0x5,
                    SomeIp => 0x6,
                    UserDefined(value) => value,
                })
            },
            Control(ref control_msg_type) => (
                EXT_MSIN_MSTP_TYPE_CONTROL,
                control_msg_type.clone() as u8
            ),
        };

        Ok(message_type | ((message_type_info << 4) & 0b1111_0000))
    }
}

///A slice containing an dlt header & payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltPacketSlice<'a> {
    slice: &'a [u8],
    header_len: usize
}

impl<'a> DltPacketSlice<'a> {

    ///Read the dlt header and create a slice containing the dlt header & payload.
    pub fn from_slice(slice: &'a [u8]) -> Result<DltPacketSlice<'_>, ReadError> {

        if slice.len() < 4 {
            return Err(ReadError::UnexpectedEndOfSlice{ minimum_size: 4, actual_size: slice.len()})
        }
        
        let length = BigEndian::read_u16(&slice[2..4]) as usize;
        if slice.len() < length {
            return Err(ReadError::UnexpectedEndOfSlice { minimum_size: length, actual_size: slice.len() });
        }

        //calculate the minimum size based on the header flags
        let header_type = slice[0];
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
                length 
            });
        }

        //looks ok -> create the DltPacketSlice
        Ok(DltPacketSlice {
            slice: &slice[..length],
            header_len
        })
    }

    ///Returns if an extended header is present.
    pub fn has_extended_header(&self) -> bool {
        0 != self.slice[0] & 0b1
    }

    ///Returns if the numbers in the payload are encoded in big endian.
    pub fn is_big_endian(&self) -> bool {
        0 != self.slice[0] & 0b10
    }

    ///Returns if the dlt package is verbose or non verbose.
    pub fn is_verbose(&self) -> bool {
        if self.has_extended_header() {
            0 != self.slice[self.header_len - 10] & 0b1
        } else {
            false
        }
    }

    ///Returns the dlt extended header if present
    pub fn extended_header(&self) -> Option<DltExtendedHeader> {
        if self.has_extended_header() {
            let slice = &self.slice[self.header_len - 10..];
            Some(DltExtendedHeader {
                message_info: slice[0],
                number_of_arguments: slice[1],
                application_id: BigEndian::read_u32(&slice[2..6]),
                context_id: BigEndian::read_u32(&slice[6..10])
            })
        } else {
            None
        }
    }

    ///Returns the message type if a parsable message type is present
    pub fn message_type(&self) -> Option<DltMessageType> {
        if self.has_extended_header() {
            DltMessageType::from_message_info_encoded(self.slice[self.header_len - 10])
        } else {
            None
        }
    }

    ///Returns the message id if the message is a non verbose message otherwise None is returned.
    pub fn message_id(&self) -> Option<u32> {
        if self.is_verbose() {
            None
        } else {
            let id_slice = &self.slice[self.header_len .. self.header_len + 4];
            if self.is_big_endian() {
                Some(BigEndian::read_u32(id_slice))
            } else {
                Some(LittleEndian::read_u32(id_slice))
            }
        }
    }

    ///Returns the slice containing the dlt header + payload.
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }

    ///Returns a slice containing the payload of the dlt message
    pub fn payload(&self) -> &'a [u8] {
        &self.slice[self.header_len..]
    }

    ///Returns a slice containing the payload of a non verbose message (after the message id).
    pub fn non_verbose_payload(&self) -> &'a [u8] {
        &self.slice[self.header_len + 4..]
    }

    ///Deserialize the dlt header
    pub fn header(&self) -> DltHeader {
        let header_type = self.slice[0];
        let (is_big_endian, version) = {
            let header_type = self.slice[0];

            (0 != header_type & BIG_ENDIAN_FLAG, 
             (header_type >> 5) & MAX_VERSION)
        };
        let message_counter = self.slice[1];
        let length = BigEndian::read_u16(&self.slice[2..4]);

        let (ecu_id, slice) = if 0 != header_type & ECU_ID_FLAG {
            (Some(BigEndian::read_u32(&self.slice[4..8])), &self.slice[8..])
        } else {
            (None, &self.slice[4..])
        };

        let (session_id, slice) = if 0 != header_type & SESSION_ID_FLAG {
            (Some(BigEndian::read_u32(&slice[..4])), &slice[4..])
        } else {
            (None, slice)
        };

        let (timestamp, slice) = if 0 != header_type & TIMESTAMP_FLAG {
            (Some(BigEndian::read_u32(&slice[..4])), &slice[4..])
        } else {
            (None, slice)
        };

        let extended_header = if 0 != header_type & EXTDENDED_HEADER_FLAG {
            Some(DltExtendedHeader {
                message_info: slice[0],
                number_of_arguments: slice[1],
                application_id: BigEndian::read_u32(&slice[2..6]),
                context_id: BigEndian::read_u32(&slice[6..10])
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
            extended_header
        }
    }
}

///Allows iterating over the someip message in a udp or tcp payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceIterator<'a> {
    slice: &'a [u8]
}

impl<'a> SliceIterator<'a> {
    pub fn new(slice: &'a [u8]) -> SliceIterator<'a> {
        SliceIterator {
            slice
        }
    }
}

impl<'a> Iterator for SliceIterator<'a> {
    type Item = Result<DltPacketSlice<'a>, ReadError>;

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

    use super::*;
    use proptest::*;
    use proptest::prelude::*;
    use std::io::Cursor;
    use std::io::Write;

    prop_compose! {
        fn extended_dlt_header_any()(message_info in any::<u8>(),
                                     number_of_arguments in any::<u8>(),
                                     application_id in any::<u32>(),
                                     context_id in any::<u32>()) -> DltExtendedHeader
        {
            DltExtendedHeader {
                message_info: message_info,
                number_of_arguments: number_of_arguments,
                application_id: application_id,
                context_id: context_id
            }
        }
    }

    prop_compose! {
        fn dlt_header_with_payload_any()(
            payload_length in 4u32..1234 //limit it a bit so that not too much memory is allocated during testing
        )(
            is_big_endian in any::<bool>(),
            version in prop::bits::u8::between(0,3),
            message_counter in any::<u8>(),
            ecu_id in any::<Option<u32>>(),
            session_id in any::<Option<u32>>(),
            timestamp in any::<Option<u32>>(),
            extended_header in option::of(extended_dlt_header_any()),
            payload in proptest::collection::vec(any::<u8>(), payload_length as usize)
        ) -> (DltHeader, Vec<u8>)
        {
            (
                {
                    let mut header = DltHeader {
                        is_big_endian,
                        version,
                        message_counter,
                        length: payload.len() as u16,
                        ecu_id,
                        session_id,
                        timestamp,
                        extended_header
                    };
                    let header_size = header.header_len();
                    header.length = header_size + (payload.len() as u16);
                    header
                },
                payload
            )
        }
    }

    prop_compose! {
        fn dlt_header_any()(is_big_endian in any::<bool>(),
                            version in prop::bits::u8::between(0,3),
                            message_counter in any::<u8>(),
                            length in any::<u16>(),
                            ecu_id in any::<Option<u32>>(),
                            session_id in any::<Option<u32>>(),
                            timestamp in any::<Option<u32>>(),
                            extended_header in option::of(extended_dlt_header_any())) -> DltHeader
        {
            DltHeader {
                is_big_endian,
                version,
                message_counter,
                length,
                ecu_id,
                session_id,
                timestamp,
                extended_header
            }
        }
    }

    fn log_level_any() -> impl Strategy<Value = DltLogLevel> {
        use DltLogLevel::*;
        prop_oneof![
            Just(Fatal),
            Just(Error),
            Just(Warn),
            Just(Info),
            Just(Debug),
            Just(Verbose),
        ]
    }

    fn message_type_any() -> impl Strategy<Value = DltMessageType> {
        use DltMessageType::*;
        use DltLogLevel::*;
        use DltNetworkType::*;
        use DltTraceType::*;
        use DltControlMessageType::*;
        prop_oneof![
            Just(Log(Fatal)),
            Just(Log(Error)),
            Just(Log(Warn)),
            Just(Log(Info)),
            Just(Log(Debug)),
            Just(Log(Verbose)),

            Just(Trace(Variable)),
            Just(Trace(FunctionIn)),
            Just(Trace(FunctionOut)),
            Just(Trace(State)),
            Just(Trace(Vfb)),

            Just(NetworkTrace(Ipc)),
            Just(NetworkTrace(Can)),
            Just(NetworkTrace(Flexray)),
            Just(NetworkTrace(Most)),
            Just(NetworkTrace(Ethernet)),
            Just(NetworkTrace(SomeIp)),
            Just(NetworkTrace(UserDefined(0x7))),
            Just(NetworkTrace(UserDefined(0x8))),
            Just(NetworkTrace(UserDefined(0x9))),
            Just(NetworkTrace(UserDefined(0xA))),
            Just(NetworkTrace(UserDefined(0xB))),
            Just(NetworkTrace(UserDefined(0xC))),
            Just(NetworkTrace(UserDefined(0xD))),
            Just(NetworkTrace(UserDefined(0xE))),
            Just(NetworkTrace(UserDefined(0xF))),

            Just(Control(Request)),
            Just(Control(Response)),
        ]
    }

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
        fn write_io_error(ref dlt_header in dlt_header_any()) {
            let mut buffer: [u8;1] = [0];
            let mut writer = Cursor::new(&mut buffer[..]);
            assert_matches!(dlt_header.write(&mut writer), Err(WriteError::IoError(_)));
        }
    }

    proptest! {
        #[test]
        fn packet_from_slice(ref packet in dlt_header_with_payload_any()) {
            let mut buffer = Vec::new();
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

            if let Some(packet_ext_header) = packet.0.extended_header.as_ref() {
                assert_eq!(slice.message_type(), packet_ext_header.message_type());
                assert_eq!(slice.header().extended_header.unwrap().message_type(), 
                           packet.0.extended_header.as_ref().unwrap().message_type());
            } else {
                assert_eq!(slice.header().extended_header, None);
                assert_eq!(slice.message_type(), None);
            }

            //check that a too small slice produces an error
            {
                let len = buffer.len();
                assert_matches!(DltPacketSlice::from_slice(&buffer[..len-1]), Err(ReadError::UnexpectedEndOfSlice{ 
                    minimum_size: _, 
                    actual_size: _
                }));
            }
        }
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

                assert_matches!(it.next(), Some(Err(ReadError::UnexpectedEndOfSlice{minimum_size: _, actual_size: _})));
                //check that the iterator does not continue
                assert_matches!(it.next(), None);
            }
            //last entry
            {
                let o = offsets.last().unwrap();
                let it = SliceIterator::new(&buffer[..(o.1 - 1)]);
                let mut it = it.skip(offsets.len()-1);

                assert_matches!(it.next(), Some(Err(ReadError::UnexpectedEndOfSlice{minimum_size: _, actual_size: _})));
                //check that the iterator does not continue
                assert_matches!(it.next(), None);
            }
        }
    }

    proptest! {
        #[test]
        fn ext_header_message_type(
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
    fn packet_from_slice_header_len_eof_errors() {
        //too small for header
        {
            let buffer = [1,2,3];
            assert_matches!(DltPacketSlice::from_slice(&buffer[..]), Err(ReadError::UnexpectedEndOfSlice{ 
                minimum_size: 4, 
                actual_size: 3
            }));
        }
        //too small for the length
        {
            let mut header: DltHeader = Default::default();
            header.length = 5;
            let mut buffer = Vec::new();
            header.write(&mut buffer).unwrap();
            assert_matches!(DltPacketSlice::from_slice(&buffer[..]), Err(ReadError::UnexpectedEndOfSlice{ 
                minimum_size: 5, 
                actual_size: 4
            }));
        }
    }

    proptest! {
        #[test]
        fn packet_from_slice_header_variable_len_eof_errors(ref input in dlt_header_any()) {
            let mut header = input.clone();
            header.length = header.header_len() + 3; //minimum payload size is 4
            let mut buffer = Vec::new();
            header.write(&mut buffer).unwrap();
            buffer.write(&[1,2,3]).unwrap();
            assert_matches!(DltPacketSlice::from_slice(&buffer[..]), Err(ReadError::LengthSmallerThenMinimum{required_length: _, length: _}));
        }
    }

    #[test]
    fn test_debug() {
        {
            use crate::ReadError::*;
            for value in [
                UnexpectedEndOfSlice { minimum_size: 1, actual_size: 2},
                LengthSmallerThenMinimum { required_length: 3, length: 4 },
                IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
            ].iter() {
                println!("{:?}", value);
            }
        }
        {
            use crate::WriteError::*;
            for value in [
                VersionTooLarge(123),
                IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))].iter()
            {
                println!("{:?}", value);
            }
        }
        {
            let header: DltHeader = Default::default();
            let mut buffer = Vec::new();
            header.write(&mut buffer).unwrap();
            let slice = DltPacketSlice::from_slice(&buffer);
            println!("{:?}", slice);
        }
    }

    proptest! {
        #[test]
        fn new_non_verbose(log_level in log_level_any(),
                           application_id in any::<u32>(),
                           context_id in any::<u32>())
        {
            use DltMessageType::Log;
            let header = DltExtendedHeader::new_non_verbose_log(log_level.clone(), application_id, context_id);
            assert_eq!(Log(log_level).encode_to_message_info_byte().unwrap(), header.message_info);
            assert_eq!(0, header.number_of_arguments);
            assert_eq!(application_id, header.application_id);
            assert_eq!(context_id, header.context_id);
        }
    }

    #[test]
    fn ext_set_is_verbose() {
        let mut header: DltExtendedHeader = Default::default();
        let original = header.clone();
        header.set_is_verbose(true);
        assert_eq!(true, header.is_verbose());
        header.set_is_verbose(false);
        assert_eq!(false, header.is_verbose());
        assert_eq!(original, header);
    }

    #[test]
    fn is_verbose() {
        let mut header: DltHeader = Default::default();
        assert_eq!(false, header.is_verbose());
        //add an extended header without the verbose flag
        header.extended_header = Some(Default::default());
        assert_eq!(false, header.is_verbose());
        //set the verbose flag
        header.extended_header.as_mut().unwrap().set_is_verbose(true);
        assert_eq!(true, header.is_verbose());
    }

    #[test]
    fn message_type() {
        use {DltMessageType::*, DltNetworkType::*, DltLogLevel::*, DltTraceType::*, DltControlMessageType::*};

        //check that setting & resetting does correctly reset the values
        {
            let mut header = DltExtendedHeader::new_non_verbose_log(Fatal, 0, 0);

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
        for message_type_id in 4 ..=0b111 {
            let mut header = DltExtendedHeader::new_non_verbose_log(Fatal, 0, 0);
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
            (Control(Request), (0u8..1).chain(3u8..=0xf))
        ];

        for t in bad_values.iter() {
            for value in t.1.clone() {
                let mut header = DltExtendedHeader::new_non_verbose(t.0.clone(), 0, 0).unwrap();
                println!("{:?}", t.0);
                header.message_info &= 0b0000_1111;
                header.message_info |= value << 4;
                assert_eq!(None, header.message_type());
            }
        }

        //check set out of range error
        {
            use RangeError::*;
            use DltLogLevel::Fatal;
            for i in 0x10..=0xff {
                let mut header = DltExtendedHeader::new_non_verbose_log(Fatal, 0, 0);
                assert_eq!(Err(NetworkTypekUserDefinedTooLarge(i)), 
                           header.set_message_type(NetworkTrace(UserDefined(i))));
            }
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
                    header.extended_header = Some(
                        {
                            let mut ext: DltExtendedHeader = Default::default();
                            ext.set_is_verbose(true);
                            ext
                        }
                    );
                    header
                },
                false
            ),
            
            //with extended header non-verbose
            (
                {
                    let mut header: DltHeader = Default::default();
                    header.extended_header = Some(
                        {
                            let mut ext: DltExtendedHeader = Default::default();
                            ext.set_is_verbose(false);
                            ext
                        }
                    );
                    header
                },
                true
            ),

            //without extended header (always non verbose)
            (
                {
                    let mut header: DltHeader = Default::default();
                    header.extended_header = None;
                    header
                },
                true
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
                buffer.write_u32::<BigEndian>(0x1234_5678).unwrap();

                //slice
                let slice = DltPacketSlice::from_slice(&buffer).unwrap();
                assert_eq!(
                    slice.message_id(),
                    if t.1 {
                        Some(0x1234_5678)
                    } else {
                        None
                    }
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
                buffer.write_u32::<LittleEndian>(0x1234_5678).unwrap();

                //slice
                let slice = DltPacketSlice::from_slice(&buffer).unwrap();
                assert_eq!(
                    slice.message_id(),
                    if t.1 {
                        Some(0x1234_5678)
                    } else {
                        None
                    }
                );
            }
        }
    }
}
