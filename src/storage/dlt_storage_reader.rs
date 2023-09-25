use std::io::{BufRead, Read};
use std::vec::Vec;

use crate::error::{DltMessageLengthTooSmallError, ReadError, UnsupportedDltVersionError};
use crate::MAX_VERSION;
use crate::{storage::StorageHeader, DltPacketSlice};

use super::StorageSlice;

/// Reader to parse a dlt storage file.
///
/// # Example
/// ```no_run
/// # let dlt_file = "dummy.dlt";
/// use std::{fs::File, io::BufReader};
/// use dlt_parse::storage::DltStorageReader;
///
/// let dlt_file = File::open(dlt_file).expect("failed to open file");
/// let mut reader = DltStorageReader::new(BufReader::new(dlt_file));
///
/// while let Some(msg_result) = reader.next_packet() {
///     let msg = msg_result.expect("failed to parse dlt packet");
///
///     // the storage header contains the ecu id and the timestamp
///     println!("{:?}", msg.storage_header);
///
///     // the dlt packet
///     println!("{:?}", msg.packet);
/// }
/// ```
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct DltStorageReader<R: Read + BufRead> {
    reader: R,
    /// Continue search for next storage header if it is missing.
    is_seeking_storage_pattern: bool,
    last_packet: Vec<u8>,
    read_error: bool,
    num_read_packets: usize,
    num_pattern_seeks: usize,
}

#[cfg(feature = "std")]
impl<R: Read + BufRead> DltStorageReader<R> {
    /// Creates a new reader.
    pub fn new(reader: R) -> DltStorageReader<R> {
        DltStorageReader {
            reader,
            is_seeking_storage_pattern: true,
            last_packet: Vec::with_capacity(u16::MAX as usize),
            read_error: false,
            num_read_packets: 0,
            num_pattern_seeks: 0,
        }
    }

    /// Creates a new reader that does not allow corrupted data
    /// and does NOT seek to the next storage pattern whenever
    /// corrupted data is encountered.
    pub fn new_strict(reader: R) -> DltStorageReader<R> {
        DltStorageReader {
            reader,
            is_seeking_storage_pattern: false,
            last_packet: Vec::with_capacity(u16::MAX as usize),
            read_error: false,
            num_read_packets: 0,
            num_pattern_seeks: 0,
        }
    }

    /// Returns if the reader will seek storage headers if corrupted
    /// data is present between packets.
    #[inline]
    pub fn is_seeking_storage_pattern(&self) -> bool {
        self.is_seeking_storage_pattern
    }

    /// Returns the number of DLT packets read.
    #[inline]
    pub fn num_read_packets(&self) -> usize {
        self.num_read_packets
    }

    /// Returns the number of times corrupt data was encountered and the
    /// next "storage pattern" ([`crate::storage::StorageHeader::PATTERN_AT_START`])
    /// had to be searched in the data stream.
    #[inline]
    pub fn num_pattern_seeks(&self) -> usize {
        self.num_pattern_seeks
    }

    /// Returns the next DLT packet.
    pub fn next_packet(&mut self) -> Option<Result<StorageSlice<'_>, ReadError>> {
        // check if iteration is done based as
        if self.read_error {
            return None;
        }

        // goto & read storage header
        let storage_header = if self.num_read_packets == 0
            || false == self.is_seeking_storage_pattern
        {
            // check if there is data left in the reader
            match self.reader.fill_buf() {
                Ok(slice) => {
                    if slice.is_empty() {
                        return None;
                    }
                }
                Err(err) => {
                    self.read_error = true;
                    return Some(Err(err.into()));
                }
            }

            // in the non seeking version a storage header is expected to be directly present
            let mut storage_header_data = [0u8; StorageHeader::BYTE_LEN];
            if let Err(err) = self.reader.read_exact(&mut storage_header_data) {
                self.read_error = true;
                return Some(Err(err.into()));
            }
            match StorageHeader::from_bytes(storage_header_data) {
                Ok(value) => value,
                Err(err) => {
                    self.read_error = true;
                    return Some(Err(err.into()));
                }
            }
        } else {
            // seek the next storage header pattern
            let mut pattern_elements_found = 0;
            let mut storage_pattern_error = false;

            while pattern_elements_found < StorageHeader::PATTERN_AT_START.len() {
                // load data
                let slice = match self.reader.fill_buf() {
                    Ok(slice) => {
                        if slice.is_empty() {
                            return None;
                        }
                        slice
                    }
                    Err(err) => {
                        self.read_error = true;
                        return Some(Err(err.into()));
                    }
                };

                // check for the pattern
                let mut consumed_len = 0;
                for d in slice {
                    if *d == StorageHeader::PATTERN_AT_START[pattern_elements_found] {
                        pattern_elements_found += 1;
                    } else {
                        storage_pattern_error = true;
                        pattern_elements_found = 0;
                    }
                    consumed_len += 1;
                    if pattern_elements_found >= StorageHeader::PATTERN_AT_START.len() {
                        break;
                    }
                }
                self.reader.consume(consumed_len);
            }
            if storage_pattern_error {
                self.num_pattern_seeks += 1;
            }

            // read the rest of the storage header
            let mut bytes = [0u8; StorageHeader::BYTE_LEN - StorageHeader::PATTERN_AT_START.len()];
            if let Err(err) = self.reader.read_exact(&mut bytes) {
                self.read_error = true;
                return Some(Err(err.into()));
            }

            StorageHeader {
                timestamp_seconds: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                timestamp_microseconds: u32::from_le_bytes([
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]),
                ecu_id: [bytes[8], bytes[9], bytes[10], bytes[11]],
            }
        };

        // read the start
        let mut header_start = [0u8; 4];
        if let Err(err) = self.reader.read_exact(&mut header_start) {
            self.read_error = true;
            return Some(Err(err.into()));
        }

        // check version
        let version = (header_start[0] >> 5) & MAX_VERSION;
        if 0 != version && 1 != version {
            self.read_error = true;
            return Some(Err(ReadError::UnsupportedDltVersion(
                UnsupportedDltVersionError {
                    unsupported_version: version,
                },
            )));
        }

        // check length to be at least 4
        let length = u16::from_be_bytes([header_start[2], header_start[3]]) as usize;
        if length < 4 {
            self.read_error = true;
            return Some(Err(ReadError::DltMessageLengthTooSmall(
                DltMessageLengthTooSmallError {
                    required_length: 4,
                    actual_length: length,
                },
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

        // packet successfully read
        self.num_read_packets += 1;

        Some(Ok(StorageSlice {
            storage_header,
            packet,
        }))
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod dlt_storage_reader_tests {
    use super::*;
    use crate::{
        error::ReadError,
        storage::{DltStorageReader, StorageHeader, StorageSlice},
        DltHeader, DltPacketSlice,
    };
    use std::format;
    use std::io::{BufRead, BufReader, Cursor};

    /// Reader that returns an error when buffer_fill is called.
    struct BufferFillErrorReader {}

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
        let r = DltStorageReader::new(BufReader::new(Cursor::new(&[])));
        assert!(format!("{:?}", r).len() > 0);
    }

    #[test]
    fn next_packet() {
        use std::vec::Vec;

        // empty reader
        {
            let mut r = DltStorageReader::new(BufReader::new(Cursor::new(&[])));
            assert!(r.next_packet().is_none());
            assert!(r.is_seeking_storage_pattern());
            assert_eq!(0, r.num_read_packets());
            assert_eq!(0, r.num_pattern_seeks());
        }

        // reader with working packets
        {
            // build two packets
            let storage_header0 = StorageHeader {
                timestamp_seconds: 1,
                timestamp_microseconds: 2,
                ecu_id: [0, 0, 0, 0],
            };
            let packet0 = {
                let mut packet = Vec::new();
                let mut header = DltHeader {
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
                // set version to 0
                packet[0] = packet[0] & 0b0001_1111;
                packet.extend_from_slice(&[1, 2, 3, 4]);
                packet
            };

            let storage_header1 = StorageHeader {
                timestamp_seconds: 3,
                timestamp_microseconds: 4,
                ecu_id: [5, 6, 7, 8],
            };
            let packet1 = {
                let mut packet = Vec::new();
                let mut header = DltHeader {
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
            // add some dummy data to test the seeking of the storage pattern
            v.extend_from_slice(&[0, 0, 0]);
            v.extend_from_slice(&storage_header1.to_bytes());
            v.extend_from_slice(&packet1);
            v.extend_from_slice(&[0, 0, 0, 0, 0, 0]);

            // check result
            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&v[..])));
            assert!(reader.is_seeking_storage_pattern());
            assert_eq!(0, reader.num_read_packets());
            assert_eq!(0, reader.num_pattern_seeks());

            assert_eq!(
                reader.next_packet().unwrap().unwrap(),
                StorageSlice {
                    storage_header: storage_header0,
                    packet: DltPacketSlice::from_slice(&packet0).unwrap()
                }
            );
            assert_eq!(1, reader.num_read_packets());
            assert_eq!(0, reader.num_pattern_seeks());

            assert_eq!(
                reader.next_packet().unwrap().unwrap(),
                StorageSlice {
                    storage_header: storage_header1.clone(),
                    packet: DltPacketSlice::from_slice(&packet1).unwrap()
                }
            );
            assert_eq!(2, reader.num_read_packets());
            assert_eq!(0, reader.num_pattern_seeks());

            assert_eq!(
                reader.next_packet().unwrap().unwrap(),
                StorageSlice {
                    storage_header: storage_header1,
                    packet: DltPacketSlice::from_slice(&packet1).unwrap()
                }
            );
            assert_eq!(3, reader.num_read_packets());
            assert_eq!(1, reader.num_pattern_seeks());

            assert!(reader.next_packet().is_none());
        }

        // reader with working packets (strict)
        {
            // build two packets
            let storage_header0 = StorageHeader {
                timestamp_seconds: 1,
                timestamp_microseconds: 2,
                ecu_id: [0, 0, 0, 0],
            };
            let packet0 = {
                let mut packet = Vec::new();
                let mut header = DltHeader {
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
                // set version to 0
                packet[0] = packet[0] & 0b0001_1111;
                packet.extend_from_slice(&[1, 2, 3, 4]);
                packet
            };

            let storage_header1 = StorageHeader {
                timestamp_seconds: 3,
                timestamp_microseconds: 4,
                ecu_id: [5, 6, 7, 8],
            };
            let packet1 = {
                let mut packet = Vec::new();
                let mut header = DltHeader {
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
            // add some dummy data to test that an error gets triggered
            v.extend_from_slice(&[0u8; StorageHeader::BYTE_LEN]);

            // check result
            let mut reader = DltStorageReader::new_strict(BufReader::new(Cursor::new(&v[..])));
            assert!(false == reader.is_seeking_storage_pattern());
            assert_eq!(0, reader.num_read_packets());
            assert_eq!(0, reader.num_pattern_seeks());

            assert_eq!(
                reader.next_packet().unwrap().unwrap(),
                StorageSlice {
                    storage_header: storage_header0,
                    packet: DltPacketSlice::from_slice(&packet0).unwrap()
                }
            );
            assert_eq!(1, reader.num_read_packets());
            assert_eq!(0, reader.num_pattern_seeks());

            assert_eq!(
                reader.next_packet().unwrap().unwrap(),
                StorageSlice {
                    storage_header: storage_header1.clone(),
                    packet: DltPacketSlice::from_slice(&packet1).unwrap()
                }
            );
            assert_eq!(2, reader.num_read_packets());
            assert_eq!(0, reader.num_pattern_seeks());

            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::StorageHeaderStartPattern(_)))
            );

            assert!(reader.next_packet().is_none());
        }

        // reader with error during buffering (non seeking)
        {
            let mut buf = BufferFillErrorReader {};
            buf.consume(0);
            buf.read(&mut []).unwrap();

            let mut reader = DltStorageReader::new(buf);
            assert_matches!(reader.next_packet(), Some(Err(ReadError::IoError(_))));
            assert!(reader.next_packet().is_none());
        }

        // reader with error during buffering (seeking)
        {
            let mut buf = BufferFillErrorReader {};
            buf.consume(0);
            buf.read(&mut []).unwrap();

            let mut reader = DltStorageReader::new(buf);
            reader.num_read_packets = 1;
            assert_matches!(reader.next_packet(), Some(Err(ReadError::IoError(_))));
            assert!(reader.next_packet().is_none());
        }

        // storage header read error at start (non seeking)
        {
            let bytes = [0u8; StorageHeader::BYTE_LEN - 1];
            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&bytes[..])));
            assert_matches!(reader.next_packet(), Some(Err(ReadError::IoError(_))));
            assert!(reader.next_packet().is_none());
        }

        // storage header read error at start (seeking)
        {
            let mut bytes = [0u8; StorageHeader::BYTE_LEN - 1];
            bytes[0] = StorageHeader::PATTERN_AT_START[0];
            bytes[1] = StorageHeader::PATTERN_AT_START[1];
            bytes[2] = StorageHeader::PATTERN_AT_START[2];
            bytes[3] = StorageHeader::PATTERN_AT_START[3];
            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&bytes[..])));
            reader.num_read_packets = 1;
            assert_matches!(reader.next_packet(), Some(Err(ReadError::IoError(_))));
            assert!(reader.next_packet().is_none());
        }

        // storage header pattern error
        {
            let mut bytes = StorageHeader {
                timestamp_seconds: 0,
                timestamp_microseconds: 0,
                ecu_id: [0u8; 4],
            }
            .to_bytes();
            bytes[0] = 0;
            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&bytes[..])));
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::StorageHeaderStartPattern(_)))
            );
            assert!(reader.next_packet().is_none());
        }

        // start read error
        {
            let mut v = Vec::new();
            v.extend_from_slice(
                &StorageHeader {
                    timestamp_seconds: 0,
                    timestamp_microseconds: 0,
                    ecu_id: [0u8; 4],
                }
                .to_bytes(),
            );
            v.extend_from_slice(&[1, 2, 3]);

            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&v[..])));
            assert_matches!(reader.next_packet(), Some(Err(ReadError::IoError(_))));
            assert!(reader.next_packet().is_none());
        }

        // dlt header version error
        {
            let mut v = Vec::new();
            {
                v.extend_from_slice(
                    &StorageHeader {
                        timestamp_seconds: 1,
                        timestamp_microseconds: 2,
                        ecu_id: [0, 0, 0, 0],
                    }
                    .to_bytes(),
                );

                let mut header = DltHeader {
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
                v.extend_from_slice(&[1, 2, 3, 4]);
            }

            // change the version to 2
            v[StorageHeader::BYTE_LEN] = 2 << 5;

            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&v[..])));
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
                v.extend_from_slice(
                    &StorageHeader {
                        timestamp_seconds: 1,
                        timestamp_microseconds: 2,
                        ecu_id: [0, 0, 0, 0],
                    }
                    .to_bytes(),
                );

                let mut header = DltHeader {
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
                v.extend_from_slice(&[1, 2, 3]); // missing one byte
            }
            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&v[..])));
            assert_matches!(reader.next_packet(), Some(Err(ReadError::IoError(_))));
            assert!(reader.next_packet().is_none());
        }

        // length size error
        {
            let mut v = Vec::new();
            {
                v.extend_from_slice(
                    &StorageHeader {
                        timestamp_seconds: 1,
                        timestamp_microseconds: 2,
                        ecu_id: [0, 0, 0, 0],
                    }
                    .to_bytes(),
                );

                DltHeader {
                    is_big_endian: true,
                    message_counter: 1,
                    length: 3, // trigger error
                    ecu_id: None,
                    session_id: None,
                    timestamp: None,
                    extended_header: None,
                }
                .write(&mut v)
                .unwrap();
                v.extend_from_slice(&[1, 2, 3]); // missing one byte
            }
            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&v[..])));
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
                v.extend_from_slice(
                    &StorageHeader {
                        timestamp_seconds: 1,
                        timestamp_microseconds: 2,
                        ecu_id: [0, 0, 0, 0],
                    }
                    .to_bytes(),
                );

                // setup a header that needs more then 4 bytes
                // so the slicing method triggers an error
                let mut header = DltHeader {
                    is_big_endian: true,
                    message_counter: 1,
                    length: 0, // set later
                    ecu_id: Some([0u8; 4]),
                    session_id: Some(1234),
                    timestamp: None,
                    extended_header: None,
                };
                header.length = header.header_len() - 1;
                header.write(&mut v).unwrap();
                v.extend_from_slice(&[]); // missing one byte
            }
            let mut reader = DltStorageReader::new(BufReader::new(Cursor::new(&v[..])));
            assert_matches!(
                reader.next_packet(),
                Some(Err(ReadError::DltMessageLengthTooSmall(_)))
            );
            assert!(reader.next_packet().is_none());
        }
    }
}
