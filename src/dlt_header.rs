use super::*;

///A dlt message header
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DltHeader {
    ///If true the payload is encoded in big endian. This does not influence the fields of the dlt header, which is always encoded in big endian.
    pub is_big_endian: bool,
    pub message_counter: u8,
    pub length: u16,
    pub ecu_id: Option<[u8; 4]>,
    pub session_id: Option<u32>,
    pub timestamp: Option<u32>,
    pub extended_header: Option<DltExtendedHeader>,
}

impl DltHeader {
    /// Versions of the DLT header that can be decoded by the decoding
    /// functions in this library.
    pub const SUPPORTED_DECODABLE_VERSIONS: [u8; 2] = [0, 1];

    /// The maximum size in bytes/octets a V1 DLT header can be when encoded.
    ///
    /// The number is calculated by adding
    ///
    /// * 4 bytes for the base header
    /// * 4 bytes for the ECU id
    /// * 4 bytes for the session id
    /// * 4 bytes for the timestamp
    /// * 10 bytes for the extended header
    pub const MAX_SERIALIZED_SIZE: usize = 4 + 4 + 4 + 4 + 10;

    /// Version that will be written into the DLT header version field when writing this header.
    pub const VERSION: u8 = 1;

    pub fn from_slice(slice: &[u8]) -> Result<DltHeader, error::PacketSliceError> {
        use error::{PacketSliceError::*, *};

        if slice.len() < 4 {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: error::Layer::DltHeader,
                minimum_size: 4,
                actual_size: slice.len(),
            }));
        }

        // SAFETY:
        // Safe as it is checked beforehand that the slice
        // has at least 4 bytes.
        let header_type = unsafe { *slice.get_unchecked(0) };

        // check version
        let version = (header_type >> 5) & MAX_VERSION;
        if 0 != version && 1 != version {
            return Err(UnsupportedDltVersion(UnsupportedDltVersionError {
                unsupported_version: version,
            }));
        }

        // calculate the minimum size based on the header flags
        // the header size has at least 4 bytes
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

        // check that enough data based on the header size is available
        if slice.len() < header_len {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: error::Layer::DltHeader,
                minimum_size: header_len,
                actual_size: slice.len(),
            }));
        }

        // SAFETY: Safe as the slice lenght has been verfied to be long
        // enough for the optional header parts.
        let mut next_option_ptr = unsafe { slice.as_ptr().add(4) };

        let ecu_id = if 0 != header_type & ECU_ID_FLAG {
            // SAFETY: Safe as header_len was extended by 4 if the ECU_ID_FLAG
            // is set & the slice len is verfied to be at least as long as
            // the header_len.
            unsafe {
                let ecu_id_ptr = next_option_ptr;
                next_option_ptr = next_option_ptr.add(4);
                Some([
                    *ecu_id_ptr,
                    *ecu_id_ptr.add(1),
                    *ecu_id_ptr.add(2),
                    *ecu_id_ptr.add(3),
                ])
            }
        } else {
            None
        };

        let session_id = if 0 != header_type & SESSION_ID_FLAG {
            // SAFETY: Safe as header_len was extended by 4 if the SESSION_ID_FLAG
            // is set & the slice len is verfied to be at least as long as
            // the header_len.
            unsafe {
                let session_id_ptr = next_option_ptr;
                next_option_ptr = next_option_ptr.add(4);
                Some(u32::from_be_bytes([
                    *session_id_ptr,
                    *session_id_ptr.add(1),
                    *session_id_ptr.add(2),
                    *session_id_ptr.add(3),
                ]))
            }
        } else {
            None
        };

        let timestamp = if 0 != header_type & TIMESTAMP_FLAG {
            // SAFETY: Safe as header_len was extended by 4 if the TIMESTAMP_FLAG
            // is set & the slice len is verfied to be at least as long as
            // the header_len.
            unsafe {
                let timestamp_id_ptr = next_option_ptr;
                next_option_ptr = next_option_ptr.add(4);
                Some(u32::from_be_bytes([
                    *timestamp_id_ptr,
                    *timestamp_id_ptr.add(1),
                    *timestamp_id_ptr.add(2),
                    *timestamp_id_ptr.add(3),
                ]))
            }
        } else {
            None
        };

        let extended_header = if 0 != header_type & EXTDENDED_HEADER_FLAG {
            Some(DltExtendedHeader {
                // SAFETY: Safe as header_len was extended by 4 if the EXTDENDED_HEADER_FLAG
                // is set & the slice len is verfied to be at least as long as
                // the header_len.
                message_info: DltMessageInfo(unsafe { *next_option_ptr }),
                number_of_arguments: unsafe { *next_option_ptr.add(1) },
                application_id: unsafe {
                    [
                        *next_option_ptr.add(2),
                        *next_option_ptr.add(3),
                        *next_option_ptr.add(4),
                        *next_option_ptr.add(5),
                    ]
                },
                context_id: unsafe {
                    [
                        *next_option_ptr.add(6),
                        *next_option_ptr.add(7),
                        *next_option_ptr.add(8),
                        *next_option_ptr.add(9),
                    ]
                },
            })
        } else {
            None
        };

        Ok(DltHeader {
            // If true the payload is encoded in big endian. This does not influence the fields of the dlt header, which is always encoded in big endian.
            is_big_endian: 0 != header_type & BIG_ENDIAN_FLAG,
            // SAFETY:
            // Safe, as the slice length was checked at the start of the function
            // to be at least 4.
            message_counter: unsafe { *slice.get_unchecked(1) },
            length: u16::from_be_bytes(
                // SAFETY:
                // Safe, as the slice length was checked at the start of the function
                // to be at least 4.
                unsafe { [*slice.get_unchecked(2), *slice.get_unchecked(3)] },
            ),
            ecu_id,
            session_id,
            timestamp,
            extended_header,
        })
    }

    /// Encodes the header to the on the wire format.
    pub fn to_bytes(&self) -> ArrayVec<u8, { DltHeader::MAX_SERIALIZED_SIZE }> {
        // encode values
        let length_be = self.length.to_be_bytes();
        let mut bytes: [u8; 26] = [
            //header type bitfield
            {
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
                result |= (DltHeader::VERSION << 5) & 0b1110_0000;
                result
            },
            self.message_counter,
            length_be[0],
            length_be[1],
            // 4 bytes ECU id
            0,
            0,
            0,
            0,
            // 4 bytes for session id
            0,
            0,
            0,
            0,
            // 4 bytes for timestamp
            0,
            0,
            0,
            0,
            // 10 bytes for extension header
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];

        let mut offset = 4;
        let mut add_4bytes = |data: [u8; 4]| {
            // SAFETY: add_4bytes not called more then 4 times
            // as and the
            unsafe {
                let ptr = bytes.as_mut_slice().as_mut_ptr().add(offset);
                *ptr = data[0];
                *ptr.add(1) = data[1];
                *ptr.add(2) = data[2];
                *ptr.add(3) = data[3];
            }
            offset += 4;
        };

        // insert optional headers
        if let Some(value) = self.ecu_id {
            add_4bytes(value);
        }

        if let Some(value) = self.session_id {
            add_4bytes(value.to_be_bytes());
        }

        if let Some(value) = self.timestamp {
            add_4bytes(value.to_be_bytes());
        }

        if let Some(value) = &self.extended_header {
            // SAFETY: 10 bytes are guranteed to be left over.
            unsafe {
                let ptr = bytes.as_mut_slice().as_mut_ptr().add(offset);
                *ptr = value.message_info.0;
                *ptr.add(1) = value.number_of_arguments;
                *ptr.add(2) = value.application_id[0];
                *ptr.add(3) = value.application_id[1];
                *ptr.add(4) = value.application_id[2];
                *ptr.add(5) = value.application_id[3];
                *ptr.add(6) = value.context_id[0];
                *ptr.add(7) = value.context_id[1];
                *ptr.add(8) = value.context_id[2];
                *ptr.add(9) = value.context_id[3];
            }
            offset += 10;
        }
        let mut result = ArrayVec::from(bytes);
        unsafe {
            result.set_len(offset);
        }
        result
    }

    ///Deserialize a DltHeader & TpHeader from the given reader.
    #[cfg(feature = "std")]
    pub fn read<T: io::Read + Sized>(reader: &mut T) -> Result<DltHeader, error::ReadError> {
        use crate::error::UnsupportedDltVersionError;

        // read the standard header that is always present
        let standard_header_start = {
            let mut standard_header_start: [u8; 4] = [0; 4];
            reader.read_exact(&mut standard_header_start)?;
            standard_header_start
        };

        //first lets read the header type
        let header_type = standard_header_start[0];

        // check version
        let version = (header_type >> 5) & MAX_VERSION;
        if 0 != version && 1 != version {
            return Err(error::ReadError::UnsupportedDltVersion(
                UnsupportedDltVersionError {
                    unsupported_version: version,
                },
            ));
        }

        //let extended_header = 0 != header_type & EXTDENDED_HEADER_FLAG;
        Ok(DltHeader {
            is_big_endian: 0 != header_type & BIG_ENDIAN_FLAG,
            message_counter: standard_header_start[1],
            length: u16::from_be_bytes([standard_header_start[2], standard_header_start[3]]),
            ecu_id: if 0 != header_type & ECU_ID_FLAG {
                Some({
                    let mut buffer: [u8; 4] = [0; 4];
                    reader.read_exact(&mut buffer)?;
                    buffer
                })
            } else {
                None
            },
            session_id: if 0 != header_type & SESSION_ID_FLAG {
                Some({
                    let mut buffer: [u8; 4] = [0; 4];
                    reader.read_exact(&mut buffer)?;
                    u32::from_be_bytes(buffer)
                })
            } else {
                None
            },
            timestamp: if 0 != header_type & TIMESTAMP_FLAG {
                Some({
                    let mut buffer: [u8; 4] = [0; 4];
                    reader.read_exact(&mut buffer)?;
                    u32::from_be_bytes(buffer)
                })
            } else {
                None
            },
            extended_header: if 0 != header_type & EXTDENDED_HEADER_FLAG {
                Some({
                    let mut buffer: [u8; 10] = [0; 10];
                    reader.read_exact(&mut buffer)?;

                    DltExtendedHeader {
                        message_info: DltMessageInfo(buffer[0]),
                        number_of_arguments: buffer[1],
                        application_id: [buffer[2], buffer[3], buffer[4], buffer[5]],
                        context_id: [buffer[6], buffer[7], buffer[8], buffer[9]],
                    }
                })
            } else {
                None
            },
        })
    }

    ///Serializes the header to the given writer.
    #[cfg(feature = "std")]
    pub fn write<T: io::Write + Sized>(&self, writer: &mut T) -> Result<(), std::io::Error> {
        {
            let length_be = self.length.to_be_bytes();
            let standard_header_start: [u8; 4] = [
                //header type bitfield
                {
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
                    result |= (DltHeader::VERSION << 5) & 0b1110_0000;
                    result
                },
                self.message_counter,
                length_be[0],
                length_be[1],
            ];

            writer.write_all(&standard_header_start)?;
        }

        if let Some(value) = self.ecu_id {
            writer.write_all(&value)?;
        }

        if let Some(value) = self.session_id {
            writer.write_all(&value.to_be_bytes())?;
        }

        if let Some(value) = self.timestamp {
            writer.write_all(&value.to_be_bytes())?;
        }

        //write the extended header if it exists
        match &self.extended_header {
            Some(value) => {
                let bytes: [u8; 10] = [
                    value.message_info.0,
                    value.number_of_arguments,
                    value.application_id[0],
                    value.application_id[1],
                    value.application_id[2],
                    value.application_id[3],
                    value.context_id[0],
                    value.context_id[1],
                    value.context_id[2],
                    value.context_id[3],
                ];
                writer.write_all(&bytes)?;
            }
            None => {}
        }
        Ok(())
    }

    ///Returns if the package is a verbose package
    #[inline]
    pub fn is_verbose(&self) -> bool {
        match &self.extended_header {
            None => false, //only packages with extended headers can be verbose
            Some(ext) => ext.is_verbose(),
        }
    }

    ///Return the byte/octed size of the serialized header (including extended header)
    #[inline]
    pub fn header_len(&self) -> u16 {
        4 + match self.ecu_id {
            Some(_) => 4,
            None => 0,
        } + match self.session_id {
            Some(_) => 4,
            None => 0,
        } + match self.timestamp {
            Some(_) => 4,
            None => 0,
        } + match self.extended_header {
            Some(_) => 10,
            None => 0,
        }
    }
}

#[cfg(test)]
mod dlt_header_tests {

    use super::*;
    use crate::proptest_generators::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn to_bytes_from_slice(
            version in 0..=1u8,
            ref dlt_header in dlt_header_any(),
            unsupported_version in (0u8..0b111u8).prop_filter(
                "version must be unknown",
                |v| !DltHeader::SUPPORTED_DECODABLE_VERSIONS.iter().any(|&x| v == &x)
            )
        ) {
            use error::PacketSliceError::*;
            // ok case
            {
                let bytes = {
                    let mut bytes = dlt_header.to_bytes();
                    // inject the supported version number
                    bytes[0] = (bytes[0] & 0b0001_1111) | ((version << 5) & 0b1110_0000);
                    bytes
                };
                assert_eq!(
                    dlt_header.clone(),
                    DltHeader::from_slice(&bytes[..]).unwrap()
                );
            }
            // from_slice unexpected end of slice error
            {
                for l in 0..dlt_header.header_len() as usize {
                    let bytes = dlt_header.to_bytes();
                    assert_eq!(
                        UnexpectedEndOfSlice(
                            error::UnexpectedEndOfSliceError{
                                minimum_size: if l < 4 {
                                    4
                                } else {
                                    dlt_header.header_len() as usize
                                },
                                actual_size: l,
                                layer: error::Layer::DltHeader,
                            }
                        ),
                        DltHeader::from_slice(&bytes[..l]).unwrap_err()
                    );
                }
            }
            // from_slice unsupported version
            {
                let mut bytes = dlt_header.to_bytes();
                // modify the version in the encoded version
                // directly
                bytes[0] = (bytes[0] & 0b0001_1111) | ((unsupported_version << 5) & 0b1110_0000);
                assert_eq!(
                    UnsupportedDltVersion(
                        error::UnsupportedDltVersionError{
                            unsupported_version,
                        }
                    ),
                    DltHeader::from_slice(&bytes[..]).unwrap_err()
                );
            }
        }
    }

    proptest! {
        #[test]
        #[cfg(feature = "std")]
        fn write_read(ref dlt_header in dlt_header_any()) {
            use std::io::Cursor;

            let mut buffer = Vec::new();
            dlt_header.write(&mut buffer).unwrap();
            let mut reader = Cursor::new(&buffer[..]);
            let result = DltHeader::read(&mut reader).unwrap();
            assert_eq!(dlt_header, &result);
        }
    }

    proptest! {
        #[test]
        #[cfg(feature = "std")]
        fn read_length_error(ref dlt_header in dlt_header_any()) {
            use std::io::Cursor;

            let mut buffer = Vec::new();
            dlt_header.write(&mut buffer).unwrap();
            let reduced_len = buffer.len() - 1;
            let mut reader = Cursor::new(&buffer[..reduced_len]);
            assert_matches!(DltHeader::read(&mut reader), Err(error::ReadError::IoError(_)));
        }
    }

    proptest! {
        #[test]
        #[cfg(feature = "std")]
        fn write_io_error(ref header in dlt_header_any()) {
            use std::io::Cursor;

            let mut buffer: Vec<u8> = Vec::with_capacity(
                header.header_len().into()
            );
            for len in 0..header.header_len() {
                buffer.resize(len.into(), 0);
                let mut writer = Cursor::new(&mut buffer[..]);
                assert_matches!(header.write(&mut writer), Err(_));
            }
        }
    }

    #[test]
    fn is_verbose() {
        let mut header: DltHeader = Default::default();
        assert_eq!(false, header.is_verbose());
        //add an extended header without the verbose flag
        header.extended_header = Some(Default::default());
        assert_eq!(false, header.is_verbose());
        //set the verbose flag
        header
            .extended_header
            .as_mut()
            .unwrap()
            .set_is_verbose(true);
        assert_eq!(true, header.is_verbose());
    }

    #[test]
    fn header_len() {
        struct Test {
            expected: u16,
            ecu_id: Option<[u8; 4]>,
            session_id: Option<u32>,
            timestamp: Option<u32>,
            extended_header: Option<DltExtendedHeader>,
        }

        let tests = [
            Test {
                expected: 4,
                ecu_id: None,
                session_id: None,
                timestamp: None,
                extended_header: None,
            },
            Test {
                expected: 4 + 4 + 4 + 4 + 10,
                ecu_id: Some([0; 4]),
                session_id: Some(0),
                timestamp: Some(0),
                extended_header: Some(Default::default()),
            },
            Test {
                expected: 4 + 4,
                ecu_id: Some([0; 4]),
                session_id: None,
                timestamp: None,
                extended_header: None,
            },
            Test {
                expected: 4 + 4,
                ecu_id: None,
                session_id: Some(0),
                timestamp: None,
                extended_header: None,
            },
            Test {
                expected: 4 + 4,
                ecu_id: None,
                session_id: None,
                timestamp: Some(0),
                extended_header: None,
            },
            Test {
                expected: 4 + 10,
                ecu_id: None,
                session_id: None,
                timestamp: None,
                extended_header: Some(Default::default()),
            },
        ];

        for test in tests {
            assert_eq!(
                test.expected,
                DltHeader {
                    is_big_endian: false,
                    message_counter: 123,
                    length: 123,
                    ecu_id: test.ecu_id,
                    session_id: test.session_id,
                    timestamp: test.timestamp,
                    extended_header: test.extended_header,
                }
                .header_len()
            );
        }
    }

    #[test]
    fn debug() {
        let header: DltHeader = Default::default();
        assert_eq!(
            format!(
                "DltHeader {{ is_big_endian: {}, message_counter: {}, length: {}, ecu_id: {:?}, session_id: {:?}, timestamp: {:?}, extended_header: {:?} }}",
                header.is_big_endian,
                header.message_counter,
                header.length,
                header.ecu_id,
                header.session_id,
                header.timestamp,
                header.extended_header,
            ),
            format!("{:?}", header)
        );
    }

    proptest! {
        #[test]
        fn clone_eq(ref header in dlt_header_any()) {
            assert_eq!(*header, header.clone());
        }
    }

    #[test]
    fn default() {
        let header: DltHeader = Default::default();
        assert_eq!(header.is_big_endian, false);
        assert_eq!(header.message_counter, 0);
        assert_eq!(header.length, 0);
        assert_eq!(header.ecu_id, None);
        assert_eq!(header.session_id, None);
        assert_eq!(header.timestamp, None);
        assert_eq!(header.extended_header, None);
    }
} // mod dlt_header_tests
