use std::io;

extern crate byteorder;
use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

///A dlt message header
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DltHeader {
    pub big_endian: bool,
    pub version: u8,
    pub message_counter: u8,
    pub length: u16,
    pub ecu_id: Option<u32>,
    pub session_id: Option<u32>,
    pub timestamp: Option<u32>,
    pub extended_header: Option<ExtendedDltHeader>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtendedDltHeader {
    pub message_info: u8,
    pub number_of_arguments: u8,
    pub application_id: u32,
    pub context_id: u32
}

#[derive(Debug)]
pub enum ReadError {
    IoError(io::Error)
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::IoError(err)
    }
}

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

const MAX_VERSION: u8 = 0b111;

const EXTDENDED_HEADER_FLAG: u8 = 0b1;
const BIG_ENDIAN_FLAG: u8 = 0b10;
const ECU_ID_FLAG: u8     = 0b100;
const SESSION_ID_FLAG: u8 = 0b1000;
const TIMESTAMP_FLAG: u8  = 0b10000;

impl DltHeader {
    pub fn read<T: io::Read + Sized>(reader: &mut T) -> Result<DltHeader, ReadError> {
        //first lets read the header type
        let header_type = reader.read_u8()?;
        //let extended_header = 0 != header_type & EXTDENDED_HEADER_FLAG;
        Ok(DltHeader{
            big_endian: 0 != header_type & BIG_ENDIAN_FLAG,
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
                Some(ExtendedDltHeader{
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
            if self.big_endian {
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
            result |= (self.version << 5) & 0b11100000;
            result
        })?;
        //write the rest of the standard header fields
        writer.write_u8(self.message_counter)?;
        writer.write_u16::<BigEndian>(self.length)?;
        match self.ecu_id {
            Some(value) => writer.write_u32::<BigEndian>(value)?,
            None => {}
        }
        match self.session_id {
            Some(value) => writer.write_u32::<BigEndian>(value)?,
            None => {}
        }
        match self.timestamp {
            Some(value) => writer.write_u32::<BigEndian>(value)?,
            None => {}
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
}

impl ExtendedDltHeader {
    pub fn verbose(&self) -> bool {
        0 != self.message_info & 0b1 
    }

    pub fn set_verbose(&mut self, is_verbose: bool) {
        if is_verbose {
            self.message_info |= 0b1;
        } else {
            self.message_info &= 0b1111_1110;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::*;
    use proptest::prelude::*;
    use std::io::Cursor;

    prop_compose! {
        fn extended_dlt_header_any()(message_info in any::<u8>(),
                                     number_of_arguments in any::<u8>(),
                                     application_id in any::<u32>(),
                                     context_id in any::<u32>()) -> ExtendedDltHeader
        {
            ExtendedDltHeader {
                message_info: message_info,
                number_of_arguments: number_of_arguments,
                application_id: application_id,
                context_id: context_id
            }
        }
    }

    prop_compose! {
        fn dlt_header_any()(big_endian in any::<bool>(),
                            version in prop::bits::u8::between(0,3),
                            message_counter in any::<u8>(),
                            length in any::<u16>(),
                            ecu_id in any::<Option<u32>>(),
                            session_id in any::<Option<u32>>(),
                            timestamp in any::<Option<u32>>(),
                            extended_header in option::of(extended_dlt_header_any())) -> DltHeader
        {
            DltHeader {
                big_endian: big_endian,
                version: version,
                message_counter: message_counter,
                length: length,
                ecu_id: ecu_id,
                session_id: session_id,
                timestamp: timestamp,
                extended_header: extended_header
            }
        }
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
}
