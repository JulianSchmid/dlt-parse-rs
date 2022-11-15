#[cfg(feature = "std")]
use std::io::{Write, Error};

use crate::{DltPacketSlice, storage::StorageHeader};

#[cfg(feature = "std")]
#[derive(Debug)]
pub struct DltStorageWriter<W: Write> {
    writer: W
}

#[cfg(feature = "std")]
impl<W: Write> DltStorageWriter<W> {

    /// Creates a new writer that allows writing dlt packets to a storage file.
    pub fn new(writer: W) -> DltStorageWriter<W> {
        DltStorageWriter{
            writer
        }
    }

    /// Writes a sliced packet into a storage file.
    pub fn write_slice<'a>(&mut self, storage_header: StorageHeader, dlt_slice: DltPacketSlice<'a>) -> Result<(), Error> {
        storage_header.write(&mut self.writer)?;
        self.writer.write_all(dlt_slice.slice())
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod dlt_storage_writer_tests {
    use super::*;
    use crate::DltHeader;
    use std::vec::Vec;
    use std::format;

    #[test]
    fn debug() {
        let mut buffer = Vec::<u8>::new();
        let writer = DltStorageWriter::new(&mut buffer);
        assert!(format!("{:?}", writer).len() > 0);
    }

    #[test]
    fn new() {
        let mut buffer = Vec::<u8>::new();
        let _writer = DltStorageWriter::new(&mut buffer);
        assert_eq!(0, buffer.len());
    }

    #[test]
    fn write_slice() {
        // ok 
        {
            let mut buffer = Vec::<u8>::new();
            let mut writer = DltStorageWriter::new(&mut buffer);

            let packet0 = {
                let mut packet = Vec::<u8>::new();
                let mut header = DltHeader{
                    is_big_endian: true,
                    message_counter: 0,
                    length: 0,
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() + 4;
                header.write(&mut packet).unwrap();
                packet.write_all(&[1,2,3,4]).unwrap();
                packet
            };
            let header0 = StorageHeader {
                timestamp_seconds: 1234,
                timestamp_microseconds: 2345,
                ecu_id: [b'A', b'B', b'C' ,b'D'],
            };
            writer.write_slice(
                header0.clone(), 
                DltPacketSlice::from_slice(&packet0).unwrap()
            ).unwrap();

            // add a secondary packet
            let packet1 = {
                let mut packet = Vec::<u8>::new();
                let mut header = DltHeader{
                    is_big_endian: false,
                    message_counter: 0,
                    length: 0,
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() + 4;
                header.write(&mut packet).unwrap();
                packet.write_all(&[9,0,1,2]).unwrap();
                packet
            };
            let header1 = StorageHeader {
                timestamp_seconds: 3456,
                timestamp_microseconds: 4567,
                ecu_id: [b'B', b'C', b'D' ,b'E'],
            };
            writer.write_slice(
                header1.clone(), 
                DltPacketSlice::from_slice(&packet1).unwrap()
            ).unwrap();

            // check contents
            {
                let mut expected = Vec::new();
                expected.extend_from_slice(&header0.to_bytes());
                expected.extend_from_slice(&packet0);
                expected.extend_from_slice(&header1.to_bytes());
                expected.extend_from_slice(&packet1);
                assert_eq!(expected, buffer);
            }
        }

        // check write error because of size error
        {
            let packet = {
                let mut packet = Vec::<u8>::new();
                let mut header = DltHeader{
                    is_big_endian: true,
                    message_counter: 0,
                    length: 0,
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() + 4;
                header.write(&mut packet).unwrap();
                packet.write_all(&[1,2,3,4]).unwrap();
                packet
            };
            let header = StorageHeader {
                timestamp_seconds: 1234,
                timestamp_microseconds: 2345,
                ecu_id: [b'A', b'B', b'C' ,b'D'],
            };
            
            // writer with not enough memory for the storage header
            {
                let mut buffer = [0u8;StorageHeader::BYTE_LEN - 1];
                let mut cursor = std::io::Cursor::new(&mut buffer[..]);
                let mut writer = DltStorageWriter::new(&mut cursor);
                assert!(writer.write_slice(
                    header.clone(),
                    DltPacketSlice::from_slice(&packet).unwrap()
                ).is_err());
            }
            // write with not enough memory for the packet
            {
                let mut buffer = [0u8;StorageHeader::BYTE_LEN + 1];
                let mut cursor = std::io::Cursor::new(&mut buffer[..]);
                let mut writer = DltStorageWriter::new(&mut cursor);
                assert!(writer.write_slice(
                    header,
                    DltPacketSlice::from_slice(&packet).unwrap()
                ).is_err());
            }
        }

    }
}