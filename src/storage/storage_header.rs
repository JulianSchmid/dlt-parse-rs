use crate::*;
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

    /// Serialized length of the header in bytes.
    pub const BYTE_LEN: usize = 16;

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
                timestamp_seconds: u32::from_be_bytes([
                    bytes[4], bytes[5], bytes[6], bytes[7]
                ]),
                timestamp_microseconds: u32::from_be_bytes([
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
    pub fn read<T: io::Read + Sized>(reader: &mut T) -> Result<StorageHeader, error::ReadError> {
        let mut bytes: [u8;16] = [0;16];
        reader.read_exact(&mut bytes)?;
        Ok(StorageHeader::from_bytes(bytes)?)
    }

    ///Serializes the header to the given writer.
    #[cfg(feature = "std")]
    pub fn write<T: io::Write + Sized>(&self, writer: &mut T) -> Result<(), std::io::Error> {
        writer.write_all(&self.to_bytes())?;
        Ok(())
    }

    /// Returns the ecu id decoded as an UTF8 string or an error if
    /// decoding was not possible.
    pub fn ecu_id_str<'a>(&'a self) -> Result<&str, Utf8Error> {
        core::str::from_utf8(&self.ecu_id)
    }
}

#[cfg(test)]
mod storage_header_tests{

    use super::*;
    use std::format;
    use proptest::prelude::*;
    use crate::proptest_generators::storage_header_any;

    proptest!{
        #[test]
        fn debug(
            header in storage_header_any()
        ) {
            prop_assert_eq!(
                format!(
                    "StorageHeader {{ timestamp_seconds: {}, timestamp_microseconds: {}, ecu_id: {:?} }}",
                    header.timestamp_seconds,
                    header.timestamp_microseconds,
                    header.ecu_id
                ),
                format!("{:?}", header)
            );
        }
    }

    proptest!{
        #[test]
        fn to_bytes(
            header in storage_header_any()
        ) {
            let secs_be = header.timestamp_seconds.to_be_bytes();
            let us_be = header.timestamp_microseconds.to_be_bytes();

            prop_assert_eq!(
                header.to_bytes(),
                [
                    0x44, 0x4C, 0x54, 0x01,
                    secs_be[0], secs_be[1], secs_be[2], secs_be[3],
                    us_be[0], us_be[1], us_be[2], us_be[3],
                    header.ecu_id[0], header.ecu_id[1], header.ecu_id[2], header.ecu_id[3], 
                ]
            );
        }
    }

    proptest!{
        #[test]
        fn from_bytes(
            header in storage_header_any(),
            bad_pattern in any::<[u8;4]>().prop_filter(
                "pattern must not match the expected pattern",
                |v| *v != StorageHeader::PATTERN_AT_START
            )
        ) {
            // ok case
            prop_assert_eq!(
                Ok(header.clone()),
                StorageHeader::from_bytes(header.to_bytes())
            );
            
            // start partern error
            {
                let mut bytes = header.to_bytes();
                bytes[0] = bad_pattern[0];
                bytes[1] = bad_pattern[1];
                bytes[2] = bad_pattern[2];
                bytes[3] = bad_pattern[3];
                prop_assert_eq!(
                    Err(error::StorageHeaderStartPatternError{
                        actual_pattern: bad_pattern.clone(),
                    }),
                    StorageHeader::from_bytes(bytes)
                );
            }
        }
    }

    proptest!{
        #[cfg(feature = "std")]
        #[test]
        fn read(
            header in storage_header_any(),
            bad_len in 0..StorageHeader::BYTE_LEN,
            bad_pattern in any::<[u8;4]>().prop_filter(
                "pattern must not match the expected pattern",
                |v| *v != StorageHeader::PATTERN_AT_START
            )
        ) {
            // ok read
            {
                let bytes = header.to_bytes();
                let mut cursor = std::io::Cursor::new(&bytes[..]);
                prop_assert_eq!(
                    header.clone(),
                    StorageHeader::read(&mut cursor).unwrap()
                );
            }

            // unexpected eof
            {
                let bytes = header.to_bytes();
                let mut cursor = std::io::Cursor::new(&bytes[..bad_len]);
                prop_assert!(StorageHeader::read(&mut cursor).is_err());
            }

            // start pattern error
            {
                let mut bytes = header.to_bytes();
                bytes[0] = bad_pattern[0];
                bytes[1] = bad_pattern[1];
                bytes[2] = bad_pattern[2];
                bytes[3] = bad_pattern[3];
                let mut cursor = std::io::Cursor::new(&bytes[..]);
                prop_assert!(StorageHeader::read(&mut cursor).is_err());
            }
        }
    }

    proptest!{
        #[cfg(feature = "std")]
        #[test]
        fn write(
            header in storage_header_any()
        ) {
            // ok write
            {
                let mut buffer = [0u8; StorageHeader::BYTE_LEN];
                let mut cursor = std::io::Cursor::new(&mut buffer[..]);
                header.write(&mut cursor).unwrap();
                prop_assert_eq!(&buffer, &header.to_bytes());
            }

            // trigger and error as there is not enough memory to write the complete header
            {
                let mut buffer = [0u8; StorageHeader::BYTE_LEN - 1];
                let mut cursor = std::io::Cursor::new(&mut buffer[..]);
                prop_assert!(header.write(&mut cursor).is_err());
            }
        }
    }

    proptest!{
        #[test]
        fn ecu_id_str(
            header in storage_header_any()
        ) {
            prop_assert_eq!(
                header.ecu_id_str(),
                core::str::from_utf8(&header.ecu_id)
            );
        }
    }

}