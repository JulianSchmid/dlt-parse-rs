use super::*;

///A slice containing an dlt header & payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltPacketSlice<'a> {
    slice: &'a [u8],
    header_len: usize,
}

impl<'a> DltPacketSlice<'a> {
    ///Read the dlt header and create a slice containing the dlt header & payload.
    pub fn from_slice(slice: &'a [u8]) -> Result<DltPacketSlice<'_>, error::PacketSliceError> {
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

        let length = u16::from_be_bytes(
            // SAFETY:
            // Safe as it is checked beforehand that the slice
            // has at least 4 bytes.
            unsafe { [*slice.get_unchecked(2), *slice.get_unchecked(3)] },
        ) as usize;

        if slice.len() < length {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: error::Layer::DltHeader,
                minimum_size: length,
                actual_size: slice.len(),
            }));
        }

        // calculate the minimum size based on the header flags
        //
        // SAFETY:
        // Safe as it is checked beforehand that the slice
        // has at least 4 bytes.
        let header_type = unsafe { slice.get_unchecked(0) };

        //the header size has at least 4 bytes
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

        // check there is enough data to at least contain the dlt header
        if length < header_len {
            return Err(MessageLengthTooSmall(DltMessageLengthTooSmallError {
                required_length: header_len,
                actual_length: length,
            }));
        }

        //looks ok -> create the DltPacketSlice
        Ok(DltPacketSlice {
            // SAFETY:
            // Safe as it is checked beforehand that the slice
            // has at least length bytes.
            slice: unsafe { from_raw_parts(slice.as_ptr(), length) },
            header_len,
        })
    }

    ///Returns if an extended header is present.
    #[inline]
    pub fn has_extended_header(&self) -> bool {
        // SAFETY:
        // Safe as it is checked in from_slice that the slice
        // has at least a length of 4 bytes.
        0 != unsafe { self.slice.get_unchecked(0) } & 0b1
    }

    ///Returns if the numbers in the payload are encoded in big endian.
    #[inline]
    pub fn is_big_endian(&self) -> bool {
        // SAFETY:
        // Safe as it is checked in from_slice that the slice
        // has at least a length of 4 bytes.
        0 != unsafe { self.slice.get_unchecked(0) } & 0b10
    }

    ///Returns if the dlt package is verbose or non verbose.
    #[inline]
    pub fn is_verbose(&self) -> bool {
        if self.has_extended_header() {
            // SAFETY:
            // Safe as if the extended header is present the
            // header_len is checked in from_slice to be at least
            // 10 bytes.
            0 != unsafe { self.slice.get_unchecked(self.header_len - 10) } & 0b1
        } else {
            false
        }
    }

    ///Returns the dlt extended header if present
    #[inline]
    pub fn extended_header(&self) -> Option<DltExtendedHeader> {
        if self.has_extended_header() {
            // SAFETY:
            // Safe as if the extended header is present the
            // header_len is set in from_slice to be at least
            // 10 bytes and also checked against the slice length.
            unsafe {
                let ext_slice = from_raw_parts(self.slice.as_ptr().add(self.header_len - 10), 10);
                Some(DltExtendedHeader {
                    message_info: *ext_slice.get_unchecked(0),
                    number_of_arguments: *ext_slice.get_unchecked(1),
                    application_id: [
                        *ext_slice.get_unchecked(2),
                        *ext_slice.get_unchecked(3),
                        *ext_slice.get_unchecked(4),
                        *ext_slice.get_unchecked(5),
                    ],
                    context_id: [
                        *ext_slice.get_unchecked(6),
                        *ext_slice.get_unchecked(7),
                        *ext_slice.get_unchecked(8),
                        *ext_slice.get_unchecked(9),
                    ],
                })
            }
        } else {
            None
        }
    }

    ///Returns the message type if a parsable message type is present
    #[inline]
    pub fn message_type(&self) -> Option<DltMessageType> {
        if self.has_extended_header() {
            DltMessageType::from_byte(
                // SAFETY:
                // Safe as if the extended header is present the
                // header_len is set in from_slice to be at least
                // 10 bytes and also checked against the slice length.
                unsafe { *self.slice.get_unchecked(self.header_len - 10) },
            )
        } else {
            None
        }
    }

    /// Returns the message id if the message is a non verbose message
    /// and enough data for a message is present. Otherwise None is returned.
    #[inline]
    pub fn message_id(&self) -> Option<u32> {
        if self.is_verbose() || self.header_len + 4 > self.slice.len() {
            None
        } else {
            // SAFETY:
            // Safe as the slice len is checked to be at least
            // header_len + 4 in the if branch above.
            let id_bytes = unsafe {
                [
                    *self.slice.get_unchecked(self.header_len),
                    *self.slice.get_unchecked(self.header_len + 1),
                    *self.slice.get_unchecked(self.header_len + 2),
                    *self.slice.get_unchecked(self.header_len + 3),
                ]
            };
            if self.is_big_endian() {
                Some(u32::from_be_bytes(id_bytes))
            } else {
                Some(u32::from_le_bytes(id_bytes))
            }
        }
    }

    ///Returns the slice containing the dlt header + payload.
    #[inline]
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }

    ///Returns a slice containing the payload of the dlt message
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        // SAFETY:
        // Safe as the slice len is checked to be at least
        // header_len + 4 in from_slice.
        unsafe {
            from_raw_parts(
                self.slice.as_ptr().add(self.header_len),
                self.slice.len() - self.header_len,
            )
        }
    }

    /// Returns the message id and a slice containing the payload (after the
    /// message id) if the dlt message is a non verbose message.
    /// 
    /// If the message is not a non verbose message or does not have enough
    /// memory for the message id `None` is returned.
    pub fn message_id_and_payload(&self) -> Option<(u32, &'a [u8])> {
        if self.is_verbose() || self.header_len + 4 > self.slice.len() {
            None
        } else {
            // SAFETY:
            // Safe as the slice len is checked to be at least
            // header_len + 4 in the if branch above.
            let id_bytes = unsafe {
                [
                    *self.slice.get_unchecked(self.header_len),
                    *self.slice.get_unchecked(self.header_len + 1),
                    *self.slice.get_unchecked(self.header_len + 2),
                    *self.slice.get_unchecked(self.header_len + 3),
                ]
            };
            let message_id = if self.is_big_endian() {
                u32::from_be_bytes(id_bytes)
            } else {
                u32::from_le_bytes(id_bytes)
            };
            // SAFETY:
            // Safe as the slice len is checked to be at least
            // header_len + 4 in the if check above.
            let non_verbose_payload = unsafe {
                from_raw_parts(
                    self.slice.as_ptr().add(self.header_len + 4),
                    self.slice.len() - self.header_len - 4,
                )
            };
            Some((message_id, non_verbose_payload))
        }
    }

    /// Returns a slice containing the payload of a non verbose message (after the message id).
    pub fn non_verbose_payload(&self) -> Option<&'a [u8]> {
        if self.is_verbose() || self.header_len + 4 > self.slice.len() {
            None
        } else {
            // SAFETY:
            // Safe as the slice len is checked to be at least
            // header_len + 4 in the if check above.
            Some(unsafe {
                from_raw_parts(
                    self.slice.as_ptr().add(self.header_len + 4),
                    self.slice.len() - self.header_len - 4,
                )
            })
        }
    }

    ///Deserialize the dlt header
    pub fn header(&self) -> DltHeader {
        // SAFETY:
        // Safe as it is checked in from_slice that the slice
        // has at least a length of 4 bytes.
        let header_type = unsafe { *self.slice.get_unchecked(0) };
        let is_big_endian = 0 != header_type & BIG_ENDIAN_FLAG;

        // SAFETY:
        // Safe as it is checked in from_slice that the slice
        // has at least a length of 4 bytes.
        let message_counter = unsafe { *self.slice.get_unchecked(1) };
        let length = u16::from_be_bytes(
            // SAFETY:
            // Safe as it is checked in from_slice that the slice
            // has at least the length of 4 bytes.
            unsafe { [*self.slice.get_unchecked(2), *self.slice.get_unchecked(3)] },
        );

        let (ecu_id, slice) = if 0 != header_type & ECU_ID_FLAG {
            (
                Some(
                    // SAFETY:
                    // Safe as it is checked in from_slice that the slice
                    // has the length to contain the standard & extended header
                    // based on the flags contained in the standard header.
                    unsafe {
                        [
                            *self.slice.get_unchecked(4),
                            *self.slice.get_unchecked(5),
                            *self.slice.get_unchecked(6),
                            *self.slice.get_unchecked(7),
                        ]
                    },
                ),
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has the length to contain the standard & extended header
                // based on the flags contained in the standard header.
                unsafe { from_raw_parts(self.slice.as_ptr().add(8), self.slice.len() - 8) },
            )
        } else {
            (
                None,
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has at least the length of 4 bytes.
                unsafe {
                    // go after the standard header base
                    from_raw_parts(self.slice.as_ptr().add(4), self.slice.len() - 4)
                },
            )
        };

        let (session_id, slice) = if 0 != header_type & SESSION_ID_FLAG {
            (
                Some(u32::from_be_bytes(
                    // SAFETY:
                    // Safe as it is checked in from_slice that the slice
                    // has the length to contain the standard & extended header
                    // based on the flags contained in the standard header.
                    unsafe {
                        [
                            *slice.get_unchecked(0),
                            *slice.get_unchecked(1),
                            *slice.get_unchecked(2),
                            *slice.get_unchecked(3),
                        ]
                    },
                )),
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has the length to contain the standard & extended header
                // based on the flags contained in the standard header.
                unsafe { from_raw_parts(slice.as_ptr().add(4), slice.len() - 4) },
            )
        } else {
            (None, slice)
        };

        let (timestamp, slice) = if 0 != header_type & TIMESTAMP_FLAG {
            (
                Some(u32::from_be_bytes(
                    // SAFETY:
                    // Safe as it is checked in from_slice that the slice
                    // has the length to contain the standard & extended header
                    // based on the flags contained in the standard header.
                    unsafe {
                        [
                            *slice.get_unchecked(0),
                            *slice.get_unchecked(1),
                            *slice.get_unchecked(2),
                            *slice.get_unchecked(3),
                        ]
                    },
                )),
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has the length to contain the standard & extended header
                // based on the flags contained in the standard header.
                unsafe { from_raw_parts(slice.as_ptr().add(4), slice.len() - 4) },
            )
        } else {
            (None, slice)
        };

        let extended_header = if 0 != header_type & EXTDENDED_HEADER_FLAG {
            Some(DltExtendedHeader {
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has the length to contain the standard & extended header
                // based on the flags contained in the standard header.
                message_info: unsafe { *slice.get_unchecked(0) },
                number_of_arguments: unsafe { *slice.get_unchecked(1) },
                application_id: unsafe {
                    [
                        *slice.get_unchecked(2),
                        *slice.get_unchecked(3),
                        *slice.get_unchecked(4),
                        *slice.get_unchecked(5),
                    ]
                },
                context_id: unsafe {
                    [
                        *slice.get_unchecked(6),
                        *slice.get_unchecked(7),
                        *slice.get_unchecked(8),
                        *slice.get_unchecked(9),
                    ]
                },
            })
        } else {
            None
        };

        DltHeader {
            is_big_endian,
            message_counter,
            length,
            ecu_id,
            session_id,
            timestamp,
            extended_header,
        }
    }
}

/// Tests for `DltPacketSlice` methods
#[cfg(test)]
mod dlt_packet_slice_tests {

    use super::*;
    use crate::proptest_generators::*;
    use proptest::prelude::*;

    #[test]
    fn debug() {
        let mut header: DltHeader = Default::default();
        header.length = header.header_len() + 4;
        let mut buffer = Vec::with_capacity(usize::from(header.length));
        buffer.extend_from_slice(&header.to_bytes());
        buffer.extend_from_slice(&[0, 0, 0, 0]);
        let slice = DltPacketSlice::from_slice(&buffer).unwrap();
        assert_eq!(
            format!(
                "DltPacketSlice {{ slice: {:?}, header_len: {} }}",
                &buffer[..],
                header.header_len(),
            ),
            format!("{:?}", slice)
        );
    }

    proptest! {
        #[test]
        fn clone_eq_debug(ref packet in dlt_header_with_payload_any()) {
            let mut buffer = Vec::with_capacity(
                usize::from(packet.0.length)
            );
            buffer.extend_from_slice(&packet.0.to_bytes());
            buffer.extend_from_slice(&packet.1);
            let slice = DltPacketSlice::from_slice(&buffer).unwrap();

            // clone & eq
            assert_eq!(slice, slice.clone());
        }
    }

    proptest! {
        #[test]
        fn from_slice(
            ref packet in dlt_header_with_payload_any(),
            version in 0..=1u8,
        ) {
            use error::PacketSliceError::*;

            let mut buffer = Vec::with_capacity(
                packet.1.len() + usize::from(packet.0.header_len())
            );
            buffer.extend_from_slice(&{
                let mut bytes = packet.0.to_bytes();
                // inject the supported version number
                bytes[0] = (bytes[0] & 0b0001_1111) | ((version << 5) & 0b1110_0000);
                bytes
            });
            buffer.extend_from_slice(&packet.1[..]);
            //read the slice
            let slice = DltPacketSlice::from_slice(&buffer[..]).unwrap();
            //check the results are matching the input
            assert_eq!(slice.header(), packet.0);
            assert_eq!(slice.has_extended_header(), packet.0.extended_header.is_some());
            assert_eq!(slice.is_big_endian(), packet.0.is_big_endian);
            assert_eq!(slice.is_verbose(), packet.0.is_verbose());
            assert_eq!(slice.payload(), &packet.1[..]);
            assert_eq!(slice.extended_header(), packet.0.extended_header);

            if let Some(packet_ext_header) = packet.0.extended_header.as_ref() {
                assert_eq!(slice.message_type(), packet_ext_header.message_type());
                assert_eq!(slice.header().extended_header.unwrap().message_type(),
                            packet.0.extended_header.as_ref().unwrap().message_type());
            } else {
                assert_eq!(slice.header().extended_header, None);
                assert_eq!(slice.message_type(), None);
            }

            //check that a too small slice produces an error
            for len in 0..buffer.len() - 1 {
                assert_matches!(
                    DltPacketSlice::from_slice(&buffer[..len]),
                    Err(
                        UnexpectedEndOfSlice(
                            error::UnexpectedEndOfSliceError {
                                layer: error::Layer::DltHeader,
                                minimum_size: _,
                                actual_size: _,
                            }
                        )
                    )
                );
            }
        }
    }

    #[test]
    fn from_slice_header_len_eof_errors() {
        use error::{PacketSliceError::*, *};
        //too small for header
        {
            let buffer = [1, 2, 3];
            assert_matches!(
                DltPacketSlice::from_slice(&buffer[..]),
                Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                    layer: error::Layer::DltHeader,
                    minimum_size: 4,
                    actual_size: 3,
                }))
            );
        }
        //too small for the length
        {
            let mut header: DltHeader = Default::default();
            header.length = 5;
            assert_matches!(
                DltPacketSlice::from_slice(&header.to_bytes()),
                Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                    layer: error::Layer::DltHeader,
                    minimum_size: 5,
                    actual_size: 4,
                }))
            );
        }
    }

    proptest! {
        #[test]
        fn from_slice_version_errors(
            ref packet in dlt_header_with_payload_any(),
            unsupported_version in (0u8..0b111u8).prop_filter(
                "version must be unknown",
                |v| !DltHeader::SUPPORTED_DECODABLE_VERSIONS.iter().any(|&x| v == &x)
            )
        ) {
            use error::{PacketSliceError::*, *};

            let mut buffer = Vec::with_capacity(
                packet.1.len() + usize::from(packet.0.header_len())
            );
            buffer.extend_from_slice(&packet.0.to_bytes());
            buffer.extend_from_slice(&packet.1[..]);

            // inject unsupported version number
            buffer[0] = (buffer[0] & 0b0001_1111) | ((unsupported_version << 5) & 0b1110_0000);

            //read the slice
            assert_eq!(
                DltPacketSlice::from_slice(&buffer[..]),
                Err(UnsupportedDltVersion(UnsupportedDltVersionError{ unsupported_version }))
            );
        }
    }

    proptest! {
        #[test]
        fn from_slice_header_variable_len_eof_errors(ref input in dlt_header_any()) {
            use error::PacketSliceError::*;
            let mut header = input.clone();
            header.length = header.header_len() - 1; // length must contain the header

            let mut buffer = ArrayVec::<u8, {DltHeader::MAX_SERIALIZED_SIZE}>::new();
            buffer.try_extend_from_slice(&header.to_bytes()).unwrap();
            assert_eq!(
                DltPacketSlice::from_slice(&buffer[..]),
                Err(MessageLengthTooSmall(error::DltMessageLengthTooSmallError{
                    required_length: header.header_len().into(),
                    actual_length: header.header_len() as usize - 1usize,
                }))
            );
        }
    }

    #[test]
    fn message_id_and_payload() {
        //pairs of (header, expected_some)
        let tests = [
            //verbose (does not have message id)
            (
                {
                    let mut header: DltHeader = Default::default();
                    header.extended_header = Some({
                        let mut ext: DltExtendedHeader = Default::default();
                        ext.set_is_verbose(true);
                        ext
                    });
                    header
                },
                false,
            ),
            //with extended header non-verbose
            (
                {
                    let mut header: DltHeader = Default::default();
                    header.extended_header = Some({
                        let mut ext: DltExtendedHeader = Default::default();
                        ext.set_is_verbose(false);
                        ext
                    });
                    header
                },
                true,
            ),
            //without extended header (always non verbose)
            (
                {
                    let mut header: DltHeader = Default::default();
                    header.extended_header = None;
                    header
                },
                true,
            ),
        ];
        // run tests
        for t in tests.iter() {
            // big endian
            {
                let header = {
                    let mut header = t.0.clone();
                    header.is_big_endian = true;
                    header.length = header.header_len() + 6;
                    header
                };

                // serialize
                let mut buffer = ArrayVec::<u8, { DltHeader::MAX_SERIALIZED_SIZE + 4 }>::new();
                buffer.try_extend_from_slice(&header.to_bytes()).unwrap();
                buffer
                    .try_extend_from_slice(&0x1234_5678u32.to_be_bytes())
                    .unwrap();
                buffer
                    .try_extend_from_slice(&[0x10, 0x11])
                    .unwrap();

                // slice
                let slice = DltPacketSlice::from_slice(&buffer).unwrap();
                if t.1 {
                    assert_eq!(Some(0x1234_5678), slice.message_id());
                    assert_eq!(
                        Some((0x1234_5678u32, &[0x10u8, 0x11][..])),
                        slice.message_id_and_payload()
                    );
                    assert_eq!(Some(&[0x10u8, 0x11][..]), slice.non_verbose_payload());
                } else {
                    assert_eq!(None, slice.message_id());
                    assert_eq!(None, slice.message_id_and_payload());
                    assert_eq!(None, slice.non_verbose_payload());
                }
            }

            // little endian
            {
                let header = {
                    let mut header = t.0.clone();
                    header.is_big_endian = false;
                    header.length = header.header_len() + 6;
                    header
                };

                // serialize
                let mut buffer = ArrayVec::<u8, { DltHeader::MAX_SERIALIZED_SIZE + 4 }>::new();
                buffer.try_extend_from_slice(&header.to_bytes()).unwrap();
                buffer
                    .try_extend_from_slice(&0x1234_5678u32.to_le_bytes())
                    .unwrap();
                buffer
                    .try_extend_from_slice(&[0x10, 0x11])
                    .unwrap();

                // slice
                let slice = DltPacketSlice::from_slice(&buffer).unwrap();
                if t.1 {
                    assert_eq!(Some(0x1234_5678), slice.message_id());
                    assert_eq!(
                        Some((0x1234_5678u32, &[0x10u8, 0x11][..])),
                        slice.message_id_and_payload()
                    );
                    assert_eq!(Some(&[0x10u8, 0x11][..]), slice.non_verbose_payload());
                } else {
                    assert_eq!(None, slice.message_id());
                    assert_eq!(None, slice.message_id_and_payload());
                    assert_eq!(None, slice.non_verbose_payload());
                }
            }

            // not enough data for the message id
            for missing_len in 1..=4 {
                let header = {
                    let mut header = t.0.clone();
                    header.is_big_endian = false;
                    header.length = header.header_len() + 4 - missing_len as u16;
                    header
                };

                // serialize
                let mut buffer = ArrayVec::<u8, { DltHeader::MAX_SERIALIZED_SIZE + 4 }>::new();
                buffer.try_extend_from_slice(&header.to_bytes()).unwrap();
                buffer
                    .try_extend_from_slice(&0x1234_5678u32.to_le_bytes())
                    .unwrap();
                
                // slice
                let slice = DltPacketSlice::from_slice(&buffer[..buffer.len() - missing_len]).unwrap();
                assert_eq!(None, slice.message_id());
                assert_eq!(None, slice.message_id_and_payload());
                assert_eq!(None, slice.non_verbose_payload());
            }
        }
    }
} // mod dlt_packet_slice
