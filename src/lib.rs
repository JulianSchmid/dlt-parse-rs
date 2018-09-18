use std::io;

extern crate byteorder;
use self::byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

///A dlt message header
#[derive(Debug, PartialEq, Eq, Clone, Default)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ExtendedDltHeader {
    pub message_info: u8,
    pub number_of_arguments: u8,
    pub application_id: u32,
    pub context_id: u32
}

///A slice containing an dlt header & payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltPacketSlice<'a> {
    slice: &'a [u8],
    header_size: usize
}

#[derive(Debug)]
pub enum ReadError {
    ///Error if the slice is smaller then dlt length field or minimal size.
    UnexpectedEndOfSlice { minimum_size: usize, actual_size: usize},
    ///Error if the dlt length is smaller then the header the calculated header size based on the flags
    LengthSmallerThenHeader { required_header_length: usize, length: usize },
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

    ///Returns if the package is a verbose package
    pub fn verbose(&self) -> bool {
        match &self.extended_header {
            None => false, //only packages with extended headers can be verbose
            Some(ext) => ext.verbose() 
        }
    }

    ///Return the byte/octed size of the serialized header
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

impl<'a> DltPacketSlice<'a> {

    ///Read the dlt header and create a slice containing the dlt header & payload.
    pub fn from_slice(slice: &'a [u8]) -> Result<DltPacketSlice, ReadError> {

        if slice.len() < 4 {
            return Err(ReadError::UnexpectedEndOfSlice{ minimum_size: 4, actual_size: slice.len()})
        }
        
        let length = BigEndian::read_u16(&slice[2..4]) as usize;
        if slice.len() < length {
            return Err(ReadError::UnexpectedEndOfSlice { minimum_size: length, actual_size: slice.len() });
        }

        //calculate the minimum size based on the header flags
        let header_type = slice[0];
        let mut header_size = 4;
        if 0 != header_type & EXTDENDED_HEADER_FLAG {
            header_size += 10;
        }
        if 0 != header_type & ECU_ID_FLAG {
            header_size += 4;
        }
        if 0 != header_type & SESSION_ID_FLAG {
            header_size += 4;
        }
        if 0 != header_type & TIMESTAMP_FLAG {
            header_size += 4;
        }
        if length < header_size {

            return Err(ReadError::LengthSmallerThenHeader { 
                required_header_length: header_size, 
                length: length 
            });
        }

        //looks ok -> create the DltPacketSlice
        Ok(DltPacketSlice {
            slice: &slice[..length],
            header_size: header_size
        })
    }

    ///Returns a slice containing the payload of the dlt message
    pub fn payload(&self) -> &'a [u8] {
        &self.slice[self.header_size..]
    }

    ///Deserialize the dlt header
    pub fn header(&self) -> DltHeader {
        let mut offset = 4;
        let header_type = self.slice[0];
        DltHeader {
            big_endian: 0 != header_type & BIG_ENDIAN_FLAG,
            version: (header_type >> 5) & MAX_VERSION,
            message_counter: self.slice[1],
            length: BigEndian::read_u16(&self.slice[2..4]),
            ecu_id: if 0 != header_type & ECU_ID_FLAG {
                let start = offset;
                offset += 4;
                Some(BigEndian::read_u32(&self.slice[start..offset]))
            } else {
                None
            },
            session_id: if 0 != header_type & SESSION_ID_FLAG {
                let start = offset;
                offset += 4;
                Some(BigEndian::read_u32(&self.slice[start..offset]))
            } else {
                None
            },
            timestamp: if 0 != header_type & TIMESTAMP_FLAG {
                let start = offset;
                offset += 4;
                Some(BigEndian::read_u32(&self.slice[start..offset]))
            } else {
                None
            },
            extended_header: if 0 != header_type & EXTDENDED_HEADER_FLAG {
                Some(ExtendedDltHeader {
                    message_info: self.slice[offset],
                    number_of_arguments: self.slice[offset + 1],
                    application_id: BigEndian::read_u32(&self.slice[offset + 2 .. offset + 6]),
                    context_id: BigEndian::read_u32(&self.slice[offset + 6 .. offset + 10])
                })
            } else {
                None
            }
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
        fn dlt_header_with_payload_any()(
            payload_length in 0u32..1234 //limit it a bit so that not too much memory is allocated during testing
        )(
            big_endian in any::<bool>(),
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
                        big_endian: big_endian,
                        version: version,
                        message_counter: message_counter,
                        length: payload.len() as u16,
                        ecu_id: ecu_id,
                        session_id: session_id,
                        timestamp: timestamp,
                        extended_header: extended_header
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
            assert_eq!(slice.payload(), &packet.1[..]);
        }
    }
    #[test]
    fn test_debug() {
        {
            use ReadError::*;
            for value in [
                IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
            ].iter() {
                println!("{:?}", value);
            }
        }
        {
            use WriteError::*;
            for value in [
                VersionTooLarge(123),
                IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))].iter()
            {
                println!("{:?}", value);
            }
        }
    }
    #[test]
    fn ext_set_verbose() {
        let mut header: ExtendedDltHeader = Default::default();
        let original = header.clone();
        header.set_verbose(true);
        assert_eq!(true, header.verbose());
        header.set_verbose(false);
        assert_eq!(false, header.verbose());
        assert_eq!(original, header);
    }
    #[test]
    fn verbose() {

        let mut header: DltHeader = Default::default();
        assert_eq!(false, header.verbose());
        //add an extended header without the verbose flag
        header.extended_header = Some(Default::default());
        assert_eq!(false, header.verbose());
        //set the verbose flag
        header.extended_header.as_mut().unwrap().set_verbose(true);
        assert_eq!(true, header.verbose());
    }
}
