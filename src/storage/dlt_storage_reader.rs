use std::io::{Read, BufRead};
use std::vec::Vec;

use crate::{DltHeader, MAX_VERSION};
use crate::error::{ReadError, UnsupportedDltVersionError, DltMessageLengthTooSmallError};
use crate::{DltPacketSlice, storage::StorageHeader};

use super::StorageSlice;

#[cfg(feature = "std")]
#[derive(Debug)]
pub struct DltStorageReader<R: Read + BufRead> {
    reader: R,
    last_packet: Vec<u8>,
    read_error: bool,
}

#[cfg(feature = "std")]
impl<R: Read + BufRead> DltStorageReader<R> {

    /// Creates a new reader.
    pub fn new(reader: R) -> DltStorageReader<R> {
        DltStorageReader{
            reader,
            last_packet: Vec::with_capacity(u16::MAX as usize),
            read_error: false,
        }
    }

    /// Returns the next DLT packet.
    pub fn next_packet(&mut self) -> Option<Result<StorageSlice<'_>, ReadError>> {

        // check if iteration is done based as
        if self.read_error {
            return None;
        }

        // check if there is data left in the reader
        match self.reader.fill_buf() {
            Ok(slice) => if slice.is_empty() {
                return None;
            },
            Err(err) => {
                self.read_error = true;
                return Some(Err(err.into()));
            }
        }

        // get the data from the storage header bytes
        let mut storage_header_data = [0u8;StorageHeader::BYTE_LEN];
        if let Err(err) = self.reader.read_exact(&mut storage_header_data) {
            self.read_error = true;
            return Some(Err(err.into()));
        }
        let storage_header = match StorageHeader::from_bytes(storage_header_data) {
            Ok(value) => value,
            Err(err) => {
                self.read_error = true;
                return Some(Err(err.into()));
            }
        };

        // read the start
        let mut header_start = [0u8;4];
        if let Err(err) = self.reader.read_exact(&mut header_start) {
            self.read_error = true;
            return Some(Err(err.into()));
        }

        // check version
        let version = (header_start[0] >> 5) & MAX_VERSION;
        if DltHeader::VERSION != version {
            self.read_error = true;
            return Some(Err(
                ReadError::UnsupportedDltVersion(
                    UnsupportedDltVersionError{
                        unsupported_version: version,
                    }
                )
            ));
        }
        
        // check length to be at least 4
        let length = u16::from_be_bytes([header_start[2], header_start[3]]) as usize;
        if length < 4 {
            self.read_error = true;
            return Some(Err(ReadError::DltMessageLengthTooSmall(
                DltMessageLengthTooSmallError {
                    required_length: 4,
                    actual_length: length,
                }
            )));
        }

        // read the complete packet
        self.last_packet.clear();
        self.last_packet.reserve(length);
        self.last_packet.extend_from_slice(&header_start);
        if length > 4 {
            self.last_packet.resize(length, 0);
            if let Err(err) = self.reader.read_exact(&mut self.last_packet[4..]) {
                self.read_error = true;
                return Some(Err(err.into()));
            }
        }

        let packet = match DltPacketSlice::from_slice(&self.last_packet) {
            Ok(packet) => packet,
            Err(err) => {
                self.read_error = true;
                return Some(Err(err.into()));
            }
        };

        Some(Ok(StorageSlice { storage_header, packet }))
    } 
}

#[cfg(test)]
#[cfg(feature = "std")]
mod dlt_storage_reader_tests {
    use super::*;
    use crate::{storage::{DltStorageReader, StorageHeader, StorageSlice}, DltHeader, DltPacketSlice, error::ReadError};
    use std::io::{Cursor, BufReader, BufRead};
    use std::format;

    /// Reader that returns an error when buffer_fill is called.
    struct BufferFillErrorReader{}

    impl Read for BufferFillErrorReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Ok(0)
        }
    }

    impl BufRead for BufferFillErrorReader {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, ""))
        }

        fn consume(&mut self, _amt: usize) {}
    }

    #[test]
    fn debug() {
        let r = DltStorageReader::new(
            BufReader::new(Cursor::new(&[]))
        );
        assert!(format!("{:?}", r).len() > 0);
    }

    #[test]
    fn next_packet() {
        use std::vec::Vec;

        // empty reader
        {
            let mut r = DltStorageReader::new(
                BufReader::new(Cursor::new(&[]))
            );
            assert!(r.next_packet().is_none());
        }

        // reader with working packets
        {
            // build two packets
            let storage_header0 = StorageHeader{
                timestamp_seconds: 1,
                timestamp_microseconds: 2,
                ecu_id: [0, 0, 0, 0],
            };
            let packet0 = {
                let mut packet = Vec::new();
                let mut header = DltHeader{
                    is_big_endian: true,
                    message_counter: 1,
                    length: 0, // set afterwords
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() + 4;
                header.write(&mut packet).unwrap();
                packet.extend_from_slice(&[1,2,3,4]);
                packet
            };

            let storage_header1 = StorageHeader{
                timestamp_seconds: 3,
                timestamp_microseconds: 4,
                ecu_id: [5, 6, 7, 8],
            };
            let packet1 = {
                let mut packet = Vec::new();
                let mut header = DltHeader{
                    is_big_endian: true,
                    message_counter: 2,
                    length: 0, // set afterwords
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() + 6;
                header.write(&mut packet).unwrap();
                packet.extend_from_slice(&[10, 11, 12, 13, 14, 15]);
                packet
            };

            // compose data
            let mut v = Vec::new();
            v.extend_from_slice(&storage_header0.to_bytes());
            v.extend_from_slice(&packet0);
            v.extend_from_slice(&storage_header1.to_bytes());
            v.extend_from_slice(&packet1);

            // check result
            let mut reader = DltStorageReader::new(
                BufReader::new(Cursor::new(&v[..]))
            );
            assert_eq!(
                reader.next_packet().unwrap().unwrap(),
                StorageSlice{
                    storage_header: storage_header0,
                    packet: DltPacketSlice::from_slice(&packet0).unwrap()
                }
            );
            assert_eq!(
                reader.next_packet().unwrap().unwrap(),
                StorageSlice{
                    storage_header: storage_header1,
                    packet: DltPacketSlice::from_slice(&packet1).unwrap()
                }
            );
            assert!(reader.next_packet().is_none());
        }

        // reader with error during buffering
        {
            let mut buf = BufferFillErrorReader{};
            buf.consume(0);
            buf.read(&mut []).unwrap();

            let mut reader = DltStorageReader::new(buf);
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::IoError(_)))
            );
            assert!(reader.next_packet().is_none());
        }

        // storage header read error
        {
            let bytes = [0u8;StorageHeader::BYTE_LEN - 1];
            let mut reader = DltStorageReader::new(
                BufReader::new(Cursor::new(&bytes[..]))
            );
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::IoError(_)))
            );
            assert!(reader.next_packet().is_none());
        }

        // storage header pattern error
        {
            let mut bytes = StorageHeader{
                timestamp_seconds: 0,
                timestamp_microseconds: 0,
                ecu_id: [0u8;4],
            }.to_bytes();
            bytes[0] = 0;
            let mut reader = DltStorageReader::new(
                BufReader::new(Cursor::new(&bytes[..]))
            );
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::StorageHeaderStartPattern(_)))
            );
            assert!(reader.next_packet().is_none());
        }

        // start read error
        {
            let mut v = Vec::new();
            v.extend_from_slice(&StorageHeader{
                timestamp_seconds: 0,
                timestamp_microseconds: 0,
                ecu_id: [0u8;4],
            }.to_bytes());
            v.extend_from_slice(&[1,2,3]);

            let mut reader = DltStorageReader::new(
                BufReader::new(Cursor::new(&v[..]))
            );
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::IoError(_)))
            );
            assert!(reader.next_packet().is_none());
        }

        // dlt header version error
        {
            let mut v = Vec::new();
            {
                v.extend_from_slice(&StorageHeader{
                    timestamp_seconds: 1,
                    timestamp_microseconds: 2,
                    ecu_id: [0, 0, 0, 0],
                }.to_bytes());
                
                let mut header = DltHeader{
                    is_big_endian: true,
                    message_counter: 1,
                    length: 0, // set afterwords
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() + 4;
                header.write(&mut v).unwrap();
                v.extend_from_slice(&[1,2,3,4]);
            }

            // change the version to 0
            v[StorageHeader::BYTE_LEN] = 0;

            let mut reader = DltStorageReader::new(
                BufReader::new(Cursor::new(&v[..]))
            );
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::UnsupportedDltVersion(_)))
            );
            assert!(reader.next_packet().is_none());
        }

        // read error of complete packet
        {
            let mut v = Vec::new();
            {
                v.extend_from_slice(&StorageHeader{
                    timestamp_seconds: 1,
                    timestamp_microseconds: 2,
                    ecu_id: [0, 0, 0, 0],
                }.to_bytes());
                
                let mut header = DltHeader{
                    is_big_endian: true,
                    message_counter: 1,
                    length: 0, // set afterwords
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() + 4;
                header.write(&mut v).unwrap();
                v.extend_from_slice(&[1,2,3]); // missing one byte
            }
            let mut reader = DltStorageReader::new(
                BufReader::new(Cursor::new(&v[..]))
            );
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::IoError(_)))
            );
            assert!(reader.next_packet().is_none());
        }

        // length size error
        {
            let mut v = Vec::new();
            {
                v.extend_from_slice(&StorageHeader{
                    timestamp_seconds: 1,
                    timestamp_microseconds: 2,
                    ecu_id: [0, 0, 0, 0],
                }.to_bytes());
                
                DltHeader{
                    is_big_endian: true,
                    message_counter: 1,
                    length: 3, // trigger error
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                }.write(&mut v).unwrap();
                v.extend_from_slice(&[1,2,3]); // missing one byte
            }
            let mut reader = DltStorageReader::new(
                BufReader::new(Cursor::new(&v[..]))
            );
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::DltMessageLengthTooSmall(_)))
            );
            assert!(reader.next_packet().is_none());
        }

        // dlt slice error
        {
            let mut v = Vec::new();
            {
                v.extend_from_slice(&StorageHeader{
                    timestamp_seconds: 1,
                    timestamp_microseconds: 2,
                    ecu_id: [0, 0, 0, 0],
                }.to_bytes());
                
                // setup a header that needs more then 4 bytes
                // so the slicing method triggers an error
                let mut header = DltHeader{
                    is_big_endian: true,
                    message_counter: 1,
                    length: 0, // set later
                    ecu_id: Some([0u8;4]),
                    session_id: Some(1234),
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() - 1;
                header.write(&mut v).unwrap();
                v.extend_from_slice(&[]); // missing one byte
            }
            let mut reader = DltStorageReader::new(
                BufReader::new(Cursor::new(&v[..]))
            );
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::DltMessageLengthTooSmall(_)))
            );
            assert!(reader.next_packet().is_none());
        }
    }

}