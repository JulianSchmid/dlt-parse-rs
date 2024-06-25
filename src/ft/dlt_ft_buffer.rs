#[cfg(feature = "std")]
use crate::{ft::*, error::FtReassembleError};
#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DltFtBuffer {
    /// Buffer containing the recostructed data
    data: Vec<u8>,

    /// Contains the ranges filled with data.
    sections: Vec<DltFtRange>,

    /// Set to the extended end size.
    file_size: u64,

    /// Number of expected packets.
    number_of_packets: u64,

    /// Buffer size.
    buffer_size: u64,

    /// File serial number (usually inode).
    file_serial_number: DltFtUInt,

    /// Absolute path to the file.
    file_name: String,

    /// File creaton date.
    creation_date: String,

    /// True if an end packet was received.
    end_received: bool,
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl DltFtBuffer {
    /// File serial number (usually inode).
    #[inline]
    pub fn file_serial_number(&self) -> DltFtUInt {
        self.file_serial_number
    }

    /// Absolute path to the file.
    #[inline]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// File creaton date.
    #[inline]
    pub fn creation_date(&self) -> &str {
        &self.creation_date
    }

    /// True if an end packet was received.
    #[inline]
    pub fn end_received(&self) -> bool {
        self.end_received
    }

    pub fn new(header: &DltFtHeaderPkg) -> Result<DltFtBuffer, FtReassembleError> {
        let mut result = DltFtBuffer {
            data: Vec::new(),
            sections: Vec::with_capacity(4),
            file_size: 0,
            number_of_packets: 0,
            buffer_size: 0,
            file_serial_number: header.file_serial_number,
            file_name: String::with_capacity(header.file_name.len()),
            creation_date: String::with_capacity(header.creation_date.len()),
            end_received: false,
        };
        result.reinit_from_header_pkg(header)?;
        Ok(result)
    }

    /// Reset buffer to starting state.
    pub fn clear(&mut self) {
        self.data.clear();
        self.sections.clear();
        self.file_size = 0;
        self.number_of_packets = 0;
        self.buffer_size = 0;
        self.file_serial_number = DltFtUInt::U64(0);
        self.file_name.clear();
        self.creation_date.clear();
        self.end_received = false;
    }

    /// Setup all the buffers based on a received header package.
    pub fn reinit_from_header_pkg(
        &mut self,
        header: &DltFtHeaderPkg,
    ) -> Result<(), FtReassembleError> {
        // validate that the file is not too big
        let file_size: u64 = header.file_size.into();
        {
            let max_allowed: u64 = usize::MAX as u64;
            if file_size > max_allowed {
                return Err(FtReassembleError::FileSizeTooBig { file_size, max_allowed });
            }
        }
        // validate that the header is consistant
        let number_of_packages: u64 = header.number_of_packages.into();
        let buffer_size: u64 = header.buffer_size.into();
        use core::ops::RangeInclusive;
        let value_err = FtReassembleError::InconsitantHeaderLenValues {
            file_size: header.file_size.into(),
            number_of_packages,
            buffer_size,
        };
        if (number_of_packages == 0 || buffer_size == 0) && file_size != 0 {
            return Err(value_err.clone());
        }
        let max_expected_size = u64::from(header.buffer_size)
            .checked_mul(header.number_of_packages.into())
            .ok_or_else(|| value_err.clone())?;
        let min_expected_size = if number_of_packages > 0 {
            (max_expected_size - buffer_size) + 1
        } else {
            0
        };

        if !RangeInclusive::new(min_expected_size, max_expected_size)
            .contains(&header.file_size.into())
        {
            return Err(value_err);
        }

        // reset the buffer
        self.data.clear();
        self.data
            .try_reserve(file_size as usize)
            .map_err(|_| FtReassembleError::AllocationFailure {
                len: file_size as usize,
            })?;
        self.sections.clear();

        // set values
        self.file_size = header.file_size.into();
        self.number_of_packets = header.number_of_packages.into();
        self.buffer_size = header.buffer_size.into();
        self.file_serial_number = header.file_serial_number;
        self.file_name.clear();
        self.file_name.push_str(header.file_name);
        self.creation_date.clear();
        self.creation_date.push_str(header.creation_date);
        self.end_received = false;

        Ok(())
    }

    /// Sets that the end packet was received.
    pub fn set_end_received(&mut self) {
        self.end_received = true;
    }

    /// Consume a DLT file transfer data package, the caller is responsible to ensure the
    /// [`DltFtDataPkg::file_serial_number`] of the data package match the
    /// [`Self::file_serial_number`] of the buffer.
    pub fn consume_data_pkg(&mut self, data: &DltFtDataPkg) -> Result<(), FtReassembleError> {
        // validate the package number
        let package_nr: u64 = data.package_nr.into();
        if package_nr == 0 || package_nr > self.number_of_packets {
            return Err(FtReassembleError::UnexpectedPackageNrInDataPkg {
                expected_nr_of_packages: self.number_of_packets,
                package_nr,
            });
        }

        // determine insertion start
        let insertion_start: usize = (package_nr as usize - 1) * (self.buffer_size as usize);

        // validate the data len of the package
        let expected_len = if package_nr < self.number_of_packets {
            self.buffer_size
        } else {
            // the last package only contains the left overs
            let rest = self.file_size % self.buffer_size;
            if rest > 0 {
                rest
            } else {
                self.buffer_size
            }
        };

        if (data.data.len() as u64) != expected_len {
            return Err(FtReassembleError::DataLenNotMatchingBufferSize {
                header_buffer_len: self.buffer_size,
                data_pkt_len: data.data.len() as u64,
                data_pkt_nr: package_nr,
                number_of_packages: self.number_of_packets,
            });
        }

        // insert the data
        // check if it is possible to grow & append the data
        let insert_end = insertion_start + (expected_len as usize);
        if self.data.len() <= insertion_start {
            // fill until insertion point if needed
            if self.data.len() < insertion_start {
                self.data.resize(insertion_start, 0);
            }
            // add data
            self.data.extend_from_slice(data.data);
        } else {
            // overwrite the existing data
            let overwrite_end = std::cmp::min(self.data.len(), insert_end);
            let overwrite_len = overwrite_end - insertion_start;
            self.data.as_mut_slice()[insertion_start..overwrite_end]
                .clone_from_slice(&data.data[..overwrite_len]);

            // in case some data still needs to appended do that as well
            if overwrite_end < insert_end {
                self.data.extend_from_slice(&data.data[overwrite_len..]);
            }
        }

        // update sections
        let mut new_section = DltFtRange {
            start: insertion_start as u64,
            end: insert_end as u64,
        };

        // merge overlapping section into new section and remove them
        self.sections.retain(|it| -> bool {
            if let Some(merged) = new_section.merge(*it) {
                new_section = merged;
                false
            } else {
                true
            }
        });
        self.sections.push(new_section);

        Ok(())
    }

    /// Returns true if the data has been completed and the end received.
    pub fn is_complete(&self) -> bool {
        self.end_received
            && 1 == self.sections.len()
            && 0 == self.sections[0].start
            && self.sections[0].end == self.file_size
    }

    /// Try finalizing the reconstructed file data and return a reference to it
    /// if the stream reconstruction was completed.
    pub fn try_finalize(&self) -> Option<DltFtCompleteInMemFile<'_>> {
        if false == self.is_complete() {
            None
        } else {
            Some(DltFtCompleteInMemFile {
                file_serial_number: self.file_serial_number,
                file_name: &self.file_name,
                creation_date: &self.creation_date,
                data: &self.data,
            })
        }
    }
}

#[cfg(all(feature = "std", test))]
mod test {

    use crate::{ft::*, error::FtReassembleError};
    use alloc::{borrow::ToOwned, vec::Vec, format};

    #[test]
    fn debug_clone_eq() {
        let buf = DltFtBuffer::new(&DltFtHeaderPkg {
            file_serial_number: DltFtUInt::U32(0),
            file_name: "a.txt",
            file_size: DltFtUInt::U32(0),
            creation_date: "",
            number_of_packages: DltFtUInt::U32(0),
            buffer_size: DltFtUInt::U32(0),
        });
        let _ = format!("{:?}", buf);
        assert_eq!(buf, buf.clone());
        assert_eq!(buf.cmp(&buf), core::cmp::Ordering::Equal);
        assert_eq!(buf.partial_cmp(&buf), Some(core::cmp::Ordering::Equal));

        use core::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let h1 = {
            let mut h = DefaultHasher::new();
            buf.hash(&mut h);
            h.finish()
        };
        let h2 = {
            let mut h = DefaultHasher::new();
            buf.clone().hash(&mut h);
            h.finish()
        };
        assert_eq!(h1, h2);
    }

    #[test]
    fn new() {
        // ok case
        {
            let buf = DltFtBuffer::new(&DltFtHeaderPkg {
                file_serial_number: DltFtUInt::U32(1234),
                file_name: "a.txt",
                file_size: DltFtUInt::U32(20),
                creation_date: "2024-06-25",
                number_of_packages: DltFtUInt::U32(2),
                buffer_size: DltFtUInt::U32(10),
            });
            assert_eq!(
                DltFtBuffer {
                    data: Vec::new(),
                    sections: Vec::new(),
                    file_size: 20,
                    number_of_packets: 2,
                    buffer_size: 10,
                    file_serial_number: DltFtUInt::U32(1234),
                    file_name: "a.txt".to_owned(),
                    creation_date: "2024-06-25".to_owned(),
                    end_received: false,
                },
                buf.unwrap()
            );
        }
        // error case
        {
            let buf = DltFtBuffer::new(&DltFtHeaderPkg {
                file_serial_number: DltFtUInt::U32(1234),
                file_name: "a.txt",
                file_size: DltFtUInt::U32(20),
                creation_date: "2024-06-25",
                // bad number of packages
                number_of_packages: DltFtUInt::U32(0),
                buffer_size: DltFtUInt::U32(10),
            });
            assert_eq!(
                FtReassembleError::InconsitantHeaderLenValues {
                    file_size: 20,
                    number_of_packages: 0,
                    buffer_size: 10,
                },
                buf.unwrap_err()
            );
        }
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn reinit_from_header_pkg_size_err_32() {
        // setup base
        let mut base = DltFtBuffer::new(&DltFtHeaderPkg {
            file_serial_number: DltFtUInt::U32(0),
            file_name: "base.txt",
            file_size: DltFtUInt::U32(4),
            creation_date: "0-0-0",
            number_of_packages: DltFtUInt::U32(1),
            buffer_size: DltFtUInt::U32(4),
        }).unwrap();

        // check for error if the file is bigger then representable
        // in 32 bits
        assert_eq!(
            FtReassembleError::FileSizeTooBig {
                file_size: usize::MAX as u64 + 1,
                max_allowed: usize::MAX as u64,
            },
            buf.reinit_from_header_pkg(&DltFtHeaderPkg {
                file_serial_number: DltFtUInt::U32(1234),
                file_name: "file.txt",
                file_size: DltFtUInt::U64(usize::MAX as u64 + 1),
                creation_date: "2024-06-25",
                number_of_packages: DltFtUInt::U32(2),
                buffer_size: DltFtUInt::U32(5),
            }).unwrap_err()
        );
    }

    #[test]
    fn reinit_from_header_pkg() {
        let base = {
            let mut base = DltFtBuffer::new(&DltFtHeaderPkg {
                file_serial_number: DltFtUInt::U32(0),
                file_name: "base.txt",
                file_size: DltFtUInt::U32(4),
                creation_date: "0-0-0",
                number_of_packages: DltFtUInt::U32(1),
                buffer_size: DltFtUInt::U32(4),
            }).unwrap();
            base.consume_data_pkg(&DltFtDataPkg{
                file_serial_number: DltFtUInt::U32(0),
                package_nr: DltFtUInt::U32(1),
                data: &[1,2,3,4],
            }).unwrap();
            base.set_end_received();
            base
        };

        // ok init
        {
            let mut buf = base.clone();
            buf.reinit_from_header_pkg(&DltFtHeaderPkg {
                file_serial_number: DltFtUInt::U32(1234),
                file_name: "file.txt",
                file_size: DltFtUInt::U32(10),
                creation_date: "2024-06-25",
                number_of_packages: DltFtUInt::U32(2),
                buffer_size: DltFtUInt::U32(5),
            }).unwrap();
            assert_eq!(
                DltFtBuffer {
                    data: Vec::new(),
                    sections: Vec::new(),
                    file_size: 10,
                    number_of_packets: 2,
                    buffer_size: 5,
                    file_serial_number: DltFtUInt::U32(1234),
                    file_name: "file.txt".to_owned(),
                    creation_date: "2024-06-25".to_owned(),
                    end_received: false,
                },
                buf
            );
        }

        // empty file
        {
            let mut buf = base.clone();
            buf.reinit_from_header_pkg(&DltFtHeaderPkg {
                file_serial_number: DltFtUInt::U32(1234),
                file_name: "file.txt",
                file_size: DltFtUInt::U32(0),
                creation_date: "2024-06-25",
                number_of_packages: DltFtUInt::U32(0),
                buffer_size: DltFtUInt::U32(0),
            }).unwrap();
            assert_eq!(
                DltFtBuffer {
                    data: Vec::new(),
                    sections: Vec::new(),
                    file_size: 0,
                    number_of_packets: 0,
                    buffer_size: 0,
                    file_serial_number: DltFtUInt::U32(1234),
                    file_name: "file.txt".to_owned(),
                    creation_date: "2024-06-25".to_owned(),
                    end_received: false,
                },
                buf
            );
        }

        // tests for consistency error checks
        {
            use FtReassembleError::*;
            let tests = [
                // file_size, number_of_packages, buffer_size
                // basic checks for zero values
                (1, 0, 0),
                (1, 1, 0),
                (1, 0, 1),
                // out of range errors
                (5, 1, 4),
                (3, 2, 4),
                (9, 2, 4),
                // overflow errors
                (9, u64::MAX, 2),
                (9, 2, u64::MAX),
            ];
            for (file_size, number_of_packages, buffer_size) in tests {
                let mut buf = base.clone();
                let err = buf.reinit_from_header_pkg(&DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(1234),
                    file_name: "file.txt",
                    file_size: DltFtUInt::U64(file_size),
                    creation_date: "2024-06-25",
                    number_of_packages: DltFtUInt::U64(number_of_packages),
                    buffer_size: DltFtUInt::U64(buffer_size),
                }).unwrap_err();
                assert_eq!(
                    err,
                    InconsitantHeaderLenValues {
                        file_size,
                        number_of_packages,
                        buffer_size,
                    }
                );
            }
        }

        // test allocation error
        {
            // TODO
        }
    }

    /*
    struct TestPacket {
        offset: u32,
        more_segments: bool,
        payload: Vec<u8>,
    }

    impl TestPacket {
        fn new(offset: u32, more_segments: bool, payload: &[u8]) -> TestPacket {
            TestPacket {
                offset,
                more_segments,
                payload: payload.iter().copied().collect(),
            }
        }

        fn send_to_buffer(&self, buffer: &mut DltFtBuffer) -> Result<(), err::TpReassembleError> {
            let packet = self.to_vec();
            let slice = SomeipMsgSlice::from_slice(&packet).unwrap();
            buffer.consume_tp(slice)
        }

        fn to_vec(&self) -> Vec<u8> {
            let header = SomeipHeader {
                message_id: 1234,
                length: 8 + 4 + self.payload.len() as u32,
                request_id: 23,
                interface_version: 1,
                message_type: MessageType::Notification,
                return_code: 0,
                tp_header: {
                    let mut tp = TpHeader::new(self.more_segments);
                    tp.set_offset(self.offset).unwrap();
                    Some(tp)
                },
            };
            let mut result = Vec::with_capacity(SOMEIP_HEADER_LENGTH + 4 + self.payload.len());
            result.extend_from_slice(&header.base_to_bytes());
            result.extend_from_slice(&header.tp_header.as_ref().unwrap().to_bytes());
            result.extend_from_slice(&self.payload);
            result
        }

        fn result_header(payload_length: u32) -> SomeipHeader {
            SomeipHeader {
                message_id: 1234,
                length: payload_length + 8,
                request_id: 23,
                interface_version: 1,
                message_type: MessageType::Notification,
                return_code: 0,
                tp_header: None,
            }
        }
    }

    #[test]
    fn new() {
        let actual = DltFtBuffer::new(DltFtBufferConfig::new(1024, 2048).unwrap());
        assert!(actual.data.is_empty());
        assert!(actual.sections.is_empty());
        assert!(actual.end.is_none());
        assert_eq!(1024, actual.config.tp_buffer_start_payload_alloc_len);
        assert_eq!(2048, actual.config.tp_max_payload_len());
    }

    #[test]
    fn clear() {
        let mut actual = DltFtBuffer::new(DltFtBufferConfig::new(1024, 2048).unwrap());

        actual.data.push(1);
        actual.sections.push(TpRange { start: 2, end: 3 });
        actual.end = Some(123);

        actual.clear();

        assert!(actual.data.is_empty());
        assert!(actual.sections.is_empty());
        assert!(actual.end.is_none());
        assert_eq!(1024, actual.config.tp_buffer_start_payload_alloc_len);
        assert_eq!(2048, actual.config.tp_max_payload_len());
    }

    /// Returns a u8 vec counting up from "start" until len is reached (truncating bits greater then u8).
    fn sequence(start: usize, len: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(len);
        for i in start..start + len {
            result.push((i & 0xff) as u8);
        }
        result
    }

    #[rustfmt::skip]
    #[test]
    fn consume() {
        use err::TpReassembleError::*;

        // normal reconstruction
        {
            let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(1024, 2048).unwrap());

            let actions = [
                (false, TestPacket::new(0, true, &sequence(0,16))),
                (false, TestPacket::new(16, true, &sequence(16,32))),
                (true, TestPacket::new(48, false, &sequence(48,16))),
            ];
            for a in actions {
                a.1.send_to_buffer(&mut buffer).unwrap();
                assert_eq!(a.0, buffer.is_complete());
            }
            let result = buffer.try_finalize().unwrap();
            assert_eq!(result.to_header(), TestPacket::result_header(16*4));
            assert_eq!(result.payload(), &sequence(0,16*4));
        }

        // overlapping reconstruction
        {
            let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(1024, 2048).unwrap());

            let actions = [
                (false, TestPacket::new(0, true, &sequence(0,16))),
                // will be overwritten
                (false, TestPacket::new(32, true, &sequence(0,16))),
                // overwrites
                (false, TestPacket::new(32, false, &sequence(32,16))),
                // completes
                (true, TestPacket::new(16, true, &sequence(16,16))),
            ];
            for a in actions {
                a.1.send_to_buffer(&mut buffer).unwrap();
                assert_eq!(a.0, buffer.is_complete());
            }
            let result = buffer.try_finalize().unwrap();
            assert_eq!(result.to_header(), TestPacket::result_header(16*3));
            assert_eq!(result.payload(), &sequence(0,16*3));
        }

        // reverse order
        {
            let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(1024, 2048).unwrap());

            let actions = [
                (false, TestPacket::new(48, false, &sequence(48,16))),
                (false, TestPacket::new(16, true, &sequence(16,32))),
                (true, TestPacket::new(0, true, &sequence(0,16))),
            ];
            for a in actions {
                a.1.send_to_buffer(&mut buffer).unwrap();
                assert_eq!(a.0, buffer.is_complete());
            }
            let result = buffer.try_finalize().unwrap();
            assert_eq!(result.to_header(), TestPacket::result_header(16*4));
            assert_eq!(result.payload(), &sequence(0,16*4));
        }

        // error tp packet bigger then max (offset only)
        {
            let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(32, 32).unwrap());
            assert_eq!(
                SegmentTooBig { offset: 32 + 16, payload_len: 16, max: 32 },
                TestPacket::new(32 + 16, true, &sequence(0,16)).send_to_buffer(&mut buffer).unwrap_err()
            );
        }

        // error tp packet bigger then max (offset + payload)
        {
            let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(32, 32).unwrap());
            assert_eq!(
                SegmentTooBig { offset: 16, payload_len: 32, max: 32 },
                TestPacket::new(16, true, &sequence(0,32)).send_to_buffer(&mut buffer).unwrap_err()
            );
        }

        // check packets that fill exactly to the max work
        {
            let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(32, 32).unwrap());
            let test_packet = TestPacket::new(16, false, &sequence(0,16));

            let packet = test_packet.to_vec();
            let slice = SomeipMsgSlice::from_slice(&packet).unwrap();

            assert_eq!(Ok(()), buffer.consume_tp(slice));
        }

        // packets conflicting with previously seen end
        for bad_offset in 1..16 {
            let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(16*100, 16*100).unwrap());
            let test_packet = TestPacket::new(48, true, &sequence(0,32 + bad_offset));

            let packet = test_packet.to_vec();
            let slice = SomeipMsgSlice::from_slice(&packet).unwrap();

            assert_eq!(
                UnalignedTpPayloadLen { offset: 48, payload_len: 32 + bad_offset },
                buffer.consume_tp(slice).unwrap_err()
            );
        }

        // test that conflicting ends trigger errors (received a different end)
        {
            let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(1024, 2048).unwrap());

            // setup an end (aka no more segements)
            TestPacket::new(32, false, &sequence(32,16)).send_to_buffer(&mut buffer).unwrap();

            // test that a "non end" going over the end package triggers an error
            assert_eq!(
                ConflictingEnd { previous_end: 32 + 16, conflicting_end: 48 + 16 },
                TestPacket::new(48, true, &sequence(48,16)).send_to_buffer(&mut buffer).unwrap_err()
            );

            // test that a new end at an earlier position triggers an error
            assert_eq!(
                ConflictingEnd { previous_end: 32 + 16, conflicting_end: 16 + 16 },
                TestPacket::new(16, false, &sequence(16,16)).send_to_buffer(&mut buffer).unwrap_err()
            );
        }
    }

    #[test]
    fn try_finalize() {
        let mut buffer = DltFtBuffer::new(DltFtBufferConfig::new(1024, 2048).unwrap());

        // not ended
        assert_eq!(buffer.try_finalize(), None);
        TestPacket::new(0, true, &sequence(0, 16))
            .send_to_buffer(&mut buffer)
            .unwrap();
        assert_eq!(buffer.try_finalize(), None);

        // ended
        TestPacket::new(16, false, &sequence(16, 16))
            .send_to_buffer(&mut buffer)
            .unwrap();
        let result = buffer.try_finalize().unwrap();
        assert_eq!(result.to_header(), TestPacket::result_header(16 * 2));
        assert_eq!(result.payload(), &sequence(0, 16 * 2));
    }
     */
}
