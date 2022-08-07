use super::*;
#[cfg(feature = "std")]
use std::io;

use core::str::Utf8Error;

/// Header present before a `DltHeader` if a DLT packet is
/// stored in .dlt file or database.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct StorageHeader {
    pub timestamp_seconds: u32,
    pub timestamp_microseconds: u32,
    pub ecu_id: [u8;4],
}

impl StorageHeader {

    /// Pattern/Magic Number that must be present at the start of a storage header.
    pub const PATTERN_AT_START: [u8;4] = [0x44, 0x4C, 0x54, 0x01];

    /// Returns the serialized from of the header.
    pub fn to_bytes(&self) -> [u8;16] {
        let ts = self.timestamp_seconds.to_be_bytes();
        let tms = self.timestamp_microseconds.to_be_bytes();
        [
            StorageHeader::PATTERN_AT_START[0],
            StorageHeader::PATTERN_AT_START[1],
            StorageHeader::PATTERN_AT_START[2],
            StorageHeader::PATTERN_AT_START[3],
            ts[0],
            ts[1],
            ts[2],
            ts[3],
            tms[0],
            tms[1],
            tms[2],
            tms[3],
            self.ecu_id[0],
            self.ecu_id[1],
            self.ecu_id[2],
            self.ecu_id[3],
        ]
    }

    /// Tries to decode a storage header.
    pub fn from_bytes(bytes: [u8;16]) -> Result<StorageHeader, error::StorageHeaderStartPatternError> {
        let start_pattern = [
            bytes[0], bytes[1], bytes[2], bytes[3],
        ];
        if start_pattern != StorageHeader::PATTERN_AT_START {
            Err(
                error::StorageHeaderStartPatternError{
                    actual_pattern: start_pattern,
                }
            )
        } else {
            Ok(StorageHeader{
                timestamp_seconds: u32::from_le_bytes([
                    bytes[4], bytes[5], bytes[6], bytes[7]
                ]),
                timestamp_microseconds: u32::from_le_bytes([
                    bytes[8], bytes[9], bytes[10], bytes[11]
                ]),
                ecu_id: [
                    bytes[12], bytes[13], bytes[14], bytes[15]
                ],
            })
        }
    }

    ///Deserialize a DltHeader & TpHeader from the given reader.
    #[cfg(feature = "std")]
    pub fn read<T: io::Read + Sized>(reader: &mut T) -> Result<StorageHeader, ReadError> {
        let mut bytes: [u8;16] = [0;16];
        reader.read_exact(&mut bytes)?;
        Ok(StorageHeader::from_bytes(bytes)?)
    }

    ///Serializes the header to the given writer.
    #[cfg(feature = "std")]
    pub fn write<T: io::Write + Sized>(&self, writer: &mut T) -> Result<(), WriteError> {
        writer.write_all(&self.to_bytes())?;
        Ok(())
    }

    /// Returns the ecu id decoded as an UTF8 string or an error if
    /// decoding was not possible.
    pub fn ecu_id_str<'a>(&'a self) -> Result<&str, Utf8Error> {
        core::str::from_utf8(&self.ecu_id)
    }
}


