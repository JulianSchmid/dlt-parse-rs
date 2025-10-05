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
    number_of_packages: u64,

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

    /// Returns the reconstructed section ranges.
    #[inline]
    pub fn sections(&self) -> &Vec<DltFtRange> {
        &self.sections
    }

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
            number_of_packages: 0,
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
        self.number_of_packages = 0;
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
        self.number_of_packages = header.number_of_packages.into();
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
        if package_nr == 0 || package_nr > self.number_of_packages {
            return Err(FtReassembleError::UnexpectedPackageNrInDataPkg {
                expected_nr_of_packages: self.number_of_packages,
                package_nr,
            });
        }

        // determine insertion start
        let insertion_start: usize = (package_nr as usize - 1) * (self.buffer_size as usize);

        // validate the data len of the package
        let expected_len = if package_nr < self.number_of_packages {
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
                number_of_packages: self.number_of_packages,
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

            // there should not be anything left to overwrite
            // as all packets have a fixed size
            debug_assert!(overwrite_end >= insert_end);
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
        self.end_received && (
            (
                1 == self.sections.len()
                && 0 == self.sections[0].start
                && self.sections[0].end == self.file_size
            ) || (
                0 == self.file_size &&
                0 == self.sections.len()
            )
        )
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
    fn getters() {
        let buf = DltFtBuffer{
            data: Vec::new(),
            sections: Vec::new(),
            file_size: 1,
            number_of_packages: 2,
            buffer_size: 3,
            file_serial_number: DltFtUInt::U32(4),
            file_name: "5".to_owned(),
            creation_date: "6".to_owned(),
            end_received: true,
        };
        assert_eq!(buf.sections(), &Vec::new());
        assert_eq!(buf.file_serial_number(), DltFtUInt::U32(4));
        assert_eq!(buf.file_name(), "5".to_owned());
        assert_eq!(buf.creation_date(), "6".to_owned());
        assert_eq!(buf.end_received(), true);
    }

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
                    number_of_packages: 2,
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
        let mut buf = DltFtBuffer::new(&DltFtHeaderPkg {
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
            let tests = [
                // file_size, number_of_packages, buffer_size
                (0, 0, 0),
                (0, 0, 4),
                (1, 1, 4),
                (2, 1, 4),
                (3, 1, 4),
                (4, 1, 4),
                (5, 2, 4),
                (6, 2, 4),
                (7, 2, 4),
                (8, 2, 4),
                (9, 3, 4),
                (10, 3, 4),
                (11, 3, 4),
                (12, 3, 4),
                (13, 4, 4),
            ];
            for (file_size, number_of_packages, buffer_size) in tests {
                let mut buf = base.clone();
                buf.reinit_from_header_pkg(&DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(1234),
                    file_name: "file.txt",
                    file_size: DltFtUInt::U64(file_size),
                    creation_date: "2024-06-25",
                    number_of_packages: DltFtUInt::U64(number_of_packages),
                    buffer_size: DltFtUInt::U64(buffer_size),
                }).unwrap();
                assert_eq!(
                    DltFtBuffer {
                        data: Vec::new(),
                        sections: Vec::new(),
                        file_size,
                        number_of_packages,
                        buffer_size,
                        file_serial_number: DltFtUInt::U32(1234),
                        file_name: "file.txt".to_owned(),
                        creation_date: "2024-06-25".to_owned(),
                        end_received: false,
                    },
                    buf
                );
            }
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
                    number_of_packages: 0,
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
            let mut buf = base.clone();
            let err = buf.reinit_from_header_pkg(&DltFtHeaderPkg {
                file_serial_number: DltFtUInt::U32(1234),
                file_name: "file.txt",
                file_size: DltFtUInt::U64(usize::MAX as u64),
                creation_date: "2024-06-25",
                number_of_packages: DltFtUInt::U64(1),
                buffer_size: DltFtUInt::U64(usize::MAX as u64),
            }).unwrap_err();
            assert_eq!(
                err,
                FtReassembleError::AllocationFailure {
                    len: usize::MAX
                }
            );
        }
    }

    #[test]
    fn set_end_received() {
        let mut buf = DltFtBuffer::new(&DltFtHeaderPkg {
            file_serial_number: DltFtUInt::U32(0),
            file_name: "base.txt",
            file_size: DltFtUInt::U32(4),
            creation_date: "0-0-0",
            number_of_packages: DltFtUInt::U32(1),
            buffer_size: DltFtUInt::U32(4),
        }).unwrap();
        buf.consume_data_pkg(&DltFtDataPkg{
            file_serial_number: DltFtUInt::U32(0),
            package_nr: DltFtUInt::U32(1),
            data: &[1,2,3,4],
        }).unwrap();

        assert_eq!(false, buf.end_received());
        buf.set_end_received();
        assert!(buf.end_received());
    }

    #[test]
    fn clear() {
        let mut buf = DltFtBuffer::new(&DltFtHeaderPkg {
            file_serial_number: DltFtUInt::U32(123),
            file_name: "base.txt",
            file_size: DltFtUInt::U64(20*12),
            creation_date: "0-0-0",
            number_of_packages: DltFtUInt::U64(12),
            buffer_size: DltFtUInt::U64(20),
        }).unwrap();
        buf.consume_data_pkg(&DltFtDataPkg{
            file_serial_number: DltFtUInt::U32(123),
            package_nr: DltFtUInt::U64(1),
            data: &[0u8;20],
        }).unwrap();
        buf.set_end_received();

        assert_eq!(20, buf.data.len());
        assert_eq!(1, buf.sections.len());
        assert_eq!(20*12, buf.file_size);
        assert_eq!(12, buf.number_of_packages);
        assert_eq!(20, buf.buffer_size);
        assert_eq!(DltFtUInt::U32(123), buf.file_serial_number);
        assert_eq!("base.txt", buf.file_name.as_str());
        assert_eq!("0-0-0", buf.creation_date.as_str());
        assert!(buf.end_received);

        buf.clear();

        assert_eq!(0, buf.data.len());
        assert_eq!(0, buf.sections.len());
        assert_eq!(0, buf.file_size);
        assert_eq!(0, buf.number_of_packages);
        assert_eq!(0, buf.buffer_size);
        assert_eq!(DltFtUInt::U64(0), buf.file_serial_number);
        assert_eq!("", buf.file_name.as_str());
        assert_eq!("", buf.creation_date.as_str());
        assert_eq!(false, buf.end_received);
    }

    #[test]
    #[rustfmt::skip]
    fn consume_data_pkg() {
        let new_buf = |file_size: u64, number_of_packages: u64, buffer_size: u64| -> DltFtBuffer {
            DltFtBuffer::new(&DltFtHeaderPkg {
                file_serial_number: DltFtUInt::U32(0),
                file_name: "base.txt",
                file_size: DltFtUInt::U64(file_size),
                creation_date: "0-0-0",
                number_of_packages: DltFtUInt::U64(number_of_packages),
                buffer_size: DltFtUInt::U64(buffer_size),
            }).unwrap()
        };
        let assert_sequence = |buf: &DltFtBuffer| {
            assert!(buf.is_complete());
            let data = buf.try_finalize().unwrap();
            assert_eq!(data.file_serial_number, buf.file_serial_number);
            assert_eq!(data.file_name, buf.file_name);
            assert_eq!(data.creation_date, buf.creation_date);
            for (i, value) in data.data.iter().enumerate() {
                assert_eq!(i as u8, *value);
            }
        };

        // ok reconstruction
        {
            let tests: [((u64, u64, u64), &'static [(u64, &'static [u8])]);12] = [
                ((0, 0, 4), &[]),
                (
                    (1, 1, 4), 
                    &[(1, &[0]),]
                ),
                (
                    (2, 1, 4), 
                    &[(1, &[0,1]),]
                ),
                (
                    (3, 1, 4), 
                    &[(1, &[0,1,2]),]
                ),
                (
                    (4, 1, 4), 
                    &[(1, &[0,1,2,3]),]
                ),
                (
                    (5, 2, 4), 
                    &[
                        (1, &[0,1,2,3]),
                        (2, &[4]),
                    ]
                ),
                (
                    (7, 2, 4), 
                    &[
                        (1, &[0,1,2,3]),
                        (2, &[4,5,6]),
                    ]
                ),
                (
                    (8, 2, 4), 
                    &[
                        (1, &[0,1,2,3]),
                        (2, &[4,5,6,7]),
                    ]
                ),
                (
                    (9, 3, 4), 
                    &[
                        (1, &[0,1,2,3]),
                        (2, &[4,5,6,7]),
                        (3, &[8]),
                    ]
                ),
                (
                    (10, 3, 4), 
                    &[
                        (1, &[0,1,2,3]),
                        (2, &[4,5,6,7]),
                        (3, &[8,9]),
                    ]
                ),
                // out of order
                (
                    (10, 3, 4), 
                    &[
                        (3, &[8,9]),
                        (1, &[0,1,2,3]),
                        (2, &[4,5,6,7]),
                    ]
                ),
                (
                    (10, 3, 4), 
                    &[
                        (1, &[0,1,2,3]),
                        (3, &[8,9]),
                        (2, &[4,5,6,7]),
                    ]
                ),
            ];

            // run with end at the end
            for ((file_size, number_of_packages, buffer_size), consumes) in tests {
                let mut buf = new_buf(file_size, number_of_packages, buffer_size);
                for (package_nr, data) in consumes {
                    assert_eq!(false, buf.is_complete());
                    assert_eq!(None, buf.try_finalize());
                    buf.consume_data_pkg(&DltFtDataPkg{
                        file_serial_number: DltFtUInt::U32(0),
                        package_nr: DltFtUInt::U64(*package_nr),
                        data,
                    }).unwrap();
                }
                buf.set_end_received();
                assert_sequence(&buf);
            }

            // end at the start
            for ((file_size, number_of_packages, buffer_size), consumes) in tests {
                let mut buf = new_buf(file_size, number_of_packages, buffer_size);
                buf.set_end_received();
                for (package_nr, data) in consumes {
                    assert_eq!(false, buf.is_complete());
                    assert_eq!(None, buf.try_finalize());
                    buf.consume_data_pkg(&DltFtDataPkg{
                        file_serial_number: DltFtUInt::U32(0),
                        package_nr: DltFtUInt::U64(*package_nr),
                        data,
                    }).unwrap();
                }
                assert_sequence(&buf);
            }
        }

        // package number error
        {
            // zero
            {
                let mut buf = new_buf(12*20, 12, 20);
                let err = buf.consume_data_pkg(&DltFtDataPkg{
                    file_serial_number: DltFtUInt::U32(0),
                    package_nr: DltFtUInt::U64(0),
                    data: &[0u8;20],
                });
                assert_eq!(
                    Err(FtReassembleError::UnexpectedPackageNrInDataPkg {
                        expected_nr_of_packages: 12,
                        package_nr: 0
                    }),
                    err
                );
            }

            // above start
            {
                let mut buf = new_buf(12*20, 12, 20);
                let err = buf.consume_data_pkg(&DltFtDataPkg{
                    file_serial_number: DltFtUInt::U32(0),
                    package_nr: DltFtUInt::U64(13),
                    data: &[0u8;20],
                });
                assert_eq!(
                    Err(FtReassembleError::UnexpectedPackageNrInDataPkg {
                        expected_nr_of_packages: 12,
                        package_nr: 13
                    }),
                    err
                );
            }
        }

        // data len error
        {
            // middle package not matching buffer size (too small)
            {
                let mut buf = new_buf(12*20, 12, 20);
                let err = buf.consume_data_pkg(&DltFtDataPkg{
                    file_serial_number: DltFtUInt::U32(0),
                    package_nr: DltFtUInt::U64(1),
                    data: &[0u8;19],
                });
                assert_eq!(
                    Err(FtReassembleError::DataLenNotMatchingBufferSize {
                        header_buffer_len: 20,
                        data_pkt_len: 19,
                        data_pkt_nr: 1,
                        number_of_packages: 12
                    }),
                    err
                );
            }
            // middle package not matching buffer size (too big)
            {
                let mut buf = new_buf(12*20, 12, 20);
                let err = buf.consume_data_pkg(&DltFtDataPkg{
                    file_serial_number: DltFtUInt::U32(0),
                    package_nr: DltFtUInt::U64(1),
                    data: &[0u8;21],
                });
                assert_eq!(
                    Err(FtReassembleError::DataLenNotMatchingBufferSize {
                        header_buffer_len: 20,
                        data_pkt_len: 21,
                        data_pkt_nr: 1,
                        number_of_packages: 12
                    }),
                    err
                );
            }
            // end package not matching end size (too big)
            {
                let mut buf = new_buf(11*20 + 15, 12, 20);
                let err = buf.consume_data_pkg(&DltFtDataPkg{
                    file_serial_number: DltFtUInt::U32(0),
                    package_nr: DltFtUInt::U64(1),
                    data: &[0u8;16],
                });
                assert_eq!(
                    Err(FtReassembleError::DataLenNotMatchingBufferSize {
                        header_buffer_len: 20,
                        data_pkt_len: 16,
                        data_pkt_nr: 1,
                        number_of_packages: 12
                    }),
                    err
                );
            }
            // end package not matching end size (too small)
            {
                let mut buf = new_buf(11*20 + 15, 12, 20);
                let err = buf.consume_data_pkg(&DltFtDataPkg{
                    file_serial_number: DltFtUInt::U32(0),
                    package_nr: DltFtUInt::U64(1),
                    data: &[0u8;14],
                });
                assert_eq!(
                    Err(FtReassembleError::DataLenNotMatchingBufferSize {
                        header_buffer_len: 20,
                        data_pkt_len: 14,
                        data_pkt_nr: 1,
                        number_of_packages: 12
                    }),
                    err
                );
            }
        }
    }
}
