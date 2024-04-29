use crate::verbose::VerboseIter;

use self::error::TypedPayloadError;

use super::*;

const SET_LOG_LEVEL: &str = "set_log_level";
const SET_TRACE_STATUS: &str = "set_trace_status";
const GET_LOG_INFO: &str = "get_log_info";
const GET_DEFAULT_LOG_LEVEL: &str = "get_default_log_level";
const STORE_CONFIGURATION: &str = "store_configuration";
const RESET_TO_FACTORY_DEFAULT: &str = "reset_to_factory_default";
const SET_MESSAGE_FILTERING: &str = "set_message_filtering";
const SET_DEFAULT_LOG_LEVEL: &str = "set_default_log_level";
const SET_DEFAULT_TRACE_STATUS: &str = "set_default_trace_status";
const GET_SOFTWARE_VERSION: &str = "get_software_version";
const GET_DEFAULT_TRACE_STATUS: &str = "get_default_trace_status";
const GET_LOG_CHANNEL_NAMES: &str = "get_log_channel_names";
const GET_TRACE_STATUS: &str = "get_trace_status";
const SET_LOG_CHANNEL_ASSIGNMENT: &str = "set_log_channel_assignment";
const SET_LOG_CHANNEL_THRESHOLD: &str = "set_log_channel_threshold";
const GET_LOG_CHANNEL_THRESHOLD: &str = "get_log_channel_threshold";
const BUFFER_OVERFLOW_NOTIFICATION: &str = "buffer_overflow_notification";
const SYNC_TIME_STAMP: &str = "sync_time_stamp";
const CALL_SWC_INJECTIONS: &str = "call_swc_injections";

pub fn service_name(service_id: u32) -> Option<&'static str> {
    match service_id {
        0x01 => Some(SET_LOG_LEVEL),
        0x02 => Some(SET_TRACE_STATUS),
        0x03 => Some(GET_LOG_INFO),
        0x04 => Some(GET_DEFAULT_LOG_LEVEL),
        0x05 => Some(STORE_CONFIGURATION),
        0x06 => Some(RESET_TO_FACTORY_DEFAULT),
        0x0A => Some(SET_MESSAGE_FILTERING),
        0x11 => Some(SET_DEFAULT_LOG_LEVEL),
        0x12 => Some(SET_DEFAULT_TRACE_STATUS),
        0x13 => Some(GET_SOFTWARE_VERSION),
        0x15 => Some(GET_DEFAULT_TRACE_STATUS),
        0x17 => Some(GET_LOG_CHANNEL_NAMES),
        0x1F => Some(GET_TRACE_STATUS),
        0x20 => Some(SET_LOG_CHANNEL_ASSIGNMENT),
        0x21 => Some(SET_LOG_CHANNEL_THRESHOLD),
        0x22 => Some(GET_LOG_CHANNEL_THRESHOLD),
        0x23 => Some(BUFFER_OVERFLOW_NOTIFICATION),
        0x24 => Some(SYNC_TIME_STAMP),
        0xFFF..=0xFFFFFFFF => Some(CALL_SWC_INJECTIONS),
        _ => None,
    }
}

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
                    message_info: DltMessageInfo(*ext_slice.get_unchecked(0)),
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

    /// Returns a iterator over the verbose values (if the dlt message is a verbose message).
    pub fn verbose_value_iter(&self) -> Option<VerboseIter<'a>> {
        // verbose messages are required to have an extended header
        if self.has_extended_header() {
            // SAFETY:
            // Safe as if the extended header is present the
            // header_len is set in from_slice to be at least
            // 10 bytes and also checked against the slice length.
            let ext_slice =
                unsafe { from_raw_parts(self.slice.as_ptr().add(self.header_len - 10), 10) };

            // check if the verbose flag is set (aka check that this is a verbose dlt message)
            // SAFETY:
            // Safe as the ext_slice is at 10.
            if 0 != unsafe { ext_slice.get_unchecked(0) } & 0b1 {
                // SAFETY:
                // Safe as the ext_slice is at 10.
                let number_of_arguments = unsafe { *ext_slice.get_unchecked(1) };

                Some(VerboseIter::new(
                    self.is_big_endian(),
                    u16::from(number_of_arguments),
                    self.payload(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the verbose or non verbose payload of the given dlt message (if it has one).
    #[inline]
    pub fn typed_payload(&self) -> Result<DltTypedPayload<'a>, TypedPayloadError> {
        // verbose messages are required to have an extended header
        let message_info = if self.has_extended_header() {
            // SAFETY:
            // Safe as if the extended header is present the
            // header_len is set in from_slice to be at least
            // 10 bytes and also checked against the slice length.
            let ext_slice =
                unsafe { from_raw_parts(self.slice.as_ptr().add(self.header_len - 10), 10) };

            // check if the verbose flag is set (aka check that this is a verbose dlt message)
            // SAFETY:
            let message_info = DltMessageInfo(unsafe { *ext_slice.get_unchecked(0) });
            // Safe as the ext_slice is at 10.
            if message_info.is_verbose() {
                // SAFETY:
                // Safe as it is checked in from_slice that the slice
                // has at least a length of 4 bytes.
                let header_type = unsafe { *self.slice.get_unchecked(0) };
                let is_big_endian = 0 != header_type & BIG_ENDIAN_FLAG;
                // SAFETY:
                // Safe as the ext_slice is at 10.
                let number_of_arguments = unsafe { *ext_slice.get_unchecked(1) };

                if let Some(message_type) = message_info.into_message_type() {
                    match message_type {
                        DltMessageType::Control(msg_type) => {
                            return Ok(DltTypedPayload::ControlV(ControlVPayload {
                                msg_type,
                                iter: VerboseIter::new(
                                    is_big_endian,
                                    u16::from(number_of_arguments),
                                    self.payload(),
                                ),
                            }));
                        }
                        DltMessageType::Log(log_level) => {
                            return Ok(DltTypedPayload::LogV(LogVPayload {
                                log_level,
                                iter: VerboseIter::new(
                                    is_big_endian,
                                    u16::from(number_of_arguments),
                                    self.payload(),
                                ),
                            }));
                        }
                        DltMessageType::NetworkTrace(net_type) => {
                            return Ok(DltTypedPayload::NetworkV(NetworkVPayload {
                                net_type,
                                iter: VerboseIter::new(
                                    is_big_endian,
                                    u16::from(number_of_arguments),
                                    self.payload(),
                                ),
                            }));
                        }
                        DltMessageType::Trace(trace_type) => {
                            return Ok(DltTypedPayload::TraceV(TraceVPayload {
                                trace_type,
                                iter: VerboseIter::new(
                                    is_big_endian,
                                    u16::from(number_of_arguments),
                                    self.payload(),
                                ),
                            }));
                        }
                    }
                } else {
                    return Err(TypedPayloadError::UnknownMessageInfo(message_info));
                }
            } else {
                Some(message_info)
            }
        } else {
            None
        };

        if self.header_len + 4 <= self.slice.len() {
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
            if let Some(info) = message_info {
                match info.into_message_type() {
                    Some(DltMessageType::Control(msg_type)) => {
                        Ok(DltTypedPayload::ControlNv(ControlNvPayload {
                            msg_type,
                            service_id: message_id,
                            payload: non_verbose_payload,
                        }))
                    }
                    Some(DltMessageType::Log(log_level)) => {
                        Ok(DltTypedPayload::LogNv(LogNvPayload {
                            log_level,
                            msg_id: message_id,
                            payload: non_verbose_payload,
                        }))
                    }
                    Some(DltMessageType::NetworkTrace(net_type)) => {
                        Ok(DltTypedPayload::NetworkNv(NetworkNvPayload {
                            net_type,
                            msg_id: message_id,
                            payload: non_verbose_payload,
                        }))
                    }
                    Some(DltMessageType::Trace(trace_type)) => {
                        Ok(DltTypedPayload::TraceNv(TraceNvPayload {
                            trace_type,
                            msg_id: message_id,
                            payload: non_verbose_payload,
                        }))
                    }
                    None => {
                        // in case there is a message info but the type
                        // is not any known
                        Err(TypedPayloadError::UnknownMessageInfo(info))
                    }
                }
            } else {
                Ok(DltTypedPayload::UnknownNv(NvPayload {
                    msg_id: message_id,
                    payload: non_verbose_payload,
                }))
            }
        } else {
            // not enough data for a non verbose message id
            Err(TypedPayloadError::LenSmallerThanMessageId {
                packet_len: self.slice.len(),
                header_len: self.header_len,
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
                message_info: DltMessageInfo(unsafe { *slice.get_unchecked(0) }),
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
mod tests {

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

    proptest! {
        #[test]
        fn payload_methods(
            is_big_endian in any::<bool>(),
            log_level in log_level_any(),
            trace_type in trace_type_any(),
            net_type in network_type_any(),
            control_message_type in control_message_type_any()
        ) {
            struct Packet {
                header: DltHeader,
                packet: ArrayVec::<u8, { DltHeader::MAX_SERIALIZED_SIZE + 8 }>,
            }

            impl Packet {

                fn new(message_info: Option<DltMessageInfo>, is_big_endian: bool) -> Packet {
                    // build header
                    let mut header: DltHeader = Default::default();
                    header.is_big_endian = is_big_endian;
                    if let Some(message_info) = message_info {
                        header.extended_header = Some(DltExtendedHeader{
                            message_info,
                            number_of_arguments: 0,
                            application_id: [0;4],
                            context_id: [0;4]
                        });
                    }
                    header.length = header.header_len() + 4 + 2;

                    let packet = Self::serialize(&header, 0x1234_5678u32);
                    Packet{ header, packet }
                }

                fn serialize(header: &DltHeader, message_id: u32) -> ArrayVec::<u8, { DltHeader::MAX_SERIALIZED_SIZE + 8 }> {
                    let mut result = ArrayVec::<u8, { DltHeader::MAX_SERIALIZED_SIZE + 8 }>::new();
                    result.try_extend_from_slice(&header.to_bytes()).unwrap();
                    if header.is_big_endian {
                        result
                            .try_extend_from_slice(&message_id.to_be_bytes())
                            .unwrap();
                    } else {
                        result
                            .try_extend_from_slice(&message_id.to_le_bytes())
                            .unwrap();
                    }
                    // dummy padding for tests
                    result.try_extend_from_slice(&[0x10, 0x11]).unwrap();
                    result
                }

                fn to_slice(&self) -> DltPacketSlice {
                    DltPacketSlice::from_slice(&self.packet).unwrap()
                }

                fn message_id(&self) -> u32 {
                    0x1234_5678u32
                }

                fn non_verbose_payload(&self) -> [u8;2] {
                    [0x10, 0x11]
                }

                fn verb_iter(&self) -> VerboseIter {
                    VerboseIter::new(
                        self.header.is_big_endian,
                        self.header.extended_header.as_ref().map(|v| v.number_of_arguments).unwrap_or_default().into(),
                        &self.packet[self.header.header_len() as usize..]
                    )
                }

                fn check_nv_len_err(&self) {
                    for payload_len in 0..4 {

                        // build new header with the new payload len
                        let mut header = self.header.clone();
                        header.length = self.header.header_len() + payload_len;
                        let data = Self::serialize(&header, 0x1234_5678u32);

                        let slice = DltPacketSlice::from_slice(&data).unwrap();
                        assert_eq!(None, slice.message_id());
                        assert_eq!(None, slice.message_id_and_payload());
                        assert_eq!(None, slice.non_verbose_payload());
                        assert_eq!(
                            Err(TypedPayloadError::LenSmallerThanMessageId { packet_len: slice.slice().len(), header_len: slice.header_len }),
                            slice.typed_payload()
                        );
                    }
                }
            }

            // unknown non verbose (no message info)
            {
                let packet = Packet::new(None, is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), Some(packet.message_id()));
                assert_eq!(slice.non_verbose_payload(), Some(&packet.non_verbose_payload()[..]));
                assert_eq!(slice.message_id_and_payload(), Some((packet.message_id(), &packet.non_verbose_payload()[..])));
                assert_eq!(slice.verbose_value_iter(), None);
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::UnknownNv(NvPayload {
                        msg_id: packet.message_id(),
                        payload: &packet.non_verbose_payload()[..]
                    }))
                );

                // check len check
                packet.check_nv_len_err();
            }

            // log non verbose message
            {
                let info = DltMessageInfo(DltMessageType::Log(log_level).to_byte().unwrap());
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), Some(packet.message_id()));
                assert_eq!(slice.non_verbose_payload(), Some(&packet.non_verbose_payload()[..]));
                assert_eq!(slice.message_id_and_payload(), Some((packet.message_id(), &packet.non_verbose_payload()[..])));
                assert_eq!(slice.verbose_value_iter(), None);
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::LogNv(LogNvPayload {
                        msg_id: packet.message_id(),
                        log_level,
                        payload: &packet.non_verbose_payload()[..]
                    }))
                );

                // check len check
                packet.check_nv_len_err();
            }

            // log verbose message
            {
                let info = DltMessageInfo(
                    DltMessageType::Log(log_level).to_byte().unwrap() | EXT_MSIN_VERB_FLAG
                );
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), None);
                assert_eq!(slice.non_verbose_payload(), None);
                assert_eq!(slice.message_id_and_payload(), None);
                assert_eq!(slice.verbose_value_iter(), Some(packet.verb_iter()));
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::LogV(LogVPayload {
                        log_level,
                        iter: packet.verb_iter()
                    }))
                );
            }

            // trace non verbose message
            {
                let info = DltMessageInfo(DltMessageType::Trace(trace_type).to_byte().unwrap());
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), Some(packet.message_id()));
                assert_eq!(slice.non_verbose_payload(), Some(&packet.non_verbose_payload()[..]));
                assert_eq!(slice.message_id_and_payload(), Some((packet.message_id(), &packet.non_verbose_payload()[..])));
                assert_eq!(slice.verbose_value_iter(), None);
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::TraceNv(TraceNvPayload {
                        trace_type,
                        msg_id: packet.message_id(),
                        payload: &packet.non_verbose_payload()[..]
                    }))
                );

                // check len check
                packet.check_nv_len_err();
            }

            // trace verbose message
            {
                let info = DltMessageInfo(
                    DltMessageType::Trace(trace_type).to_byte().unwrap() | EXT_MSIN_VERB_FLAG
                );
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), None);
                assert_eq!(slice.non_verbose_payload(), None);
                assert_eq!(slice.message_id_and_payload(), None);
                assert_eq!(slice.verbose_value_iter(), Some(packet.verb_iter()));
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::TraceV(TraceVPayload {
                        trace_type,
                        iter: packet.verb_iter()
                    }))
                );
            }

            // network non verbose message
            {
                let info = DltMessageInfo(DltMessageType::NetworkTrace(net_type).to_byte().unwrap());
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), Some(packet.message_id()));
                assert_eq!(slice.non_verbose_payload(), Some(&packet.non_verbose_payload()[..]));
                assert_eq!(slice.message_id_and_payload(), Some((packet.message_id(), &packet.non_verbose_payload()[..])));
                assert_eq!(slice.verbose_value_iter(), None);
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::NetworkNv(NetworkNvPayload {
                        net_type,
                        msg_id: packet.message_id(),
                        payload: &packet.non_verbose_payload()[..]
                    }))
                );

                // check len check
                packet.check_nv_len_err();
            }

            // network verbose message
            {
                let info = DltMessageInfo(
                    DltMessageType::NetworkTrace(net_type).to_byte().unwrap() | EXT_MSIN_VERB_FLAG
                );
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), None);
                assert_eq!(slice.non_verbose_payload(), None);
                assert_eq!(slice.message_id_and_payload(), None);
                assert_eq!(slice.verbose_value_iter(), Some(packet.verb_iter()));
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::NetworkV(NetworkVPayload {
                        net_type,
                        iter: packet.verb_iter()
                    }))
                );
            }

            // control non verbose message
            {
                let info = DltMessageInfo(DltMessageType::Control(control_message_type).to_byte().unwrap());
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), Some(packet.message_id()));
                assert_eq!(slice.non_verbose_payload(), Some(&packet.non_verbose_payload()[..]));
                assert_eq!(slice.message_id_and_payload(), Some((packet.message_id(), &packet.non_verbose_payload()[..])));
                assert_eq!(slice.verbose_value_iter(), None);
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::ControlNv(ControlNvPayload {
                        msg_type: control_message_type,
                        service_id: packet.message_id(),
                        payload: &packet.non_verbose_payload()[..]
                    }))
                );

                // check len check
                packet.check_nv_len_err();
            }

            // control verbose message
            {
                let info = DltMessageInfo(
                    DltMessageType::Control(control_message_type).to_byte().unwrap() | EXT_MSIN_VERB_FLAG
                );
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), None);
                assert_eq!(slice.non_verbose_payload(), None);
                assert_eq!(slice.message_id_and_payload(), None);
                assert_eq!(slice.verbose_value_iter(), Some(packet.verb_iter()));
                assert_eq!(
                    slice.typed_payload(),
                    Ok(DltTypedPayload::ControlV(ControlVPayload {
                        msg_type: control_message_type,
                        iter: packet.verb_iter()
                    }))
                );
            }

            // unknown message info error
            {
                // 7 is not a valid log level
                let info = DltMessageInfo(EXT_MSIN_MSTP_TYPE_LOG | (0x7 << 4));
                let packet = Packet::new(Some(info), is_big_endian);
                let slice = packet.to_slice();

                // check accessors
                assert_eq!(slice.message_id(), Some(packet.message_id()));
                assert_eq!(slice.non_verbose_payload(), Some(&packet.non_verbose_payload()[..]));
                assert_eq!(slice.message_id_and_payload(), Some((packet.message_id(), &packet.non_verbose_payload()[..])));
                assert_eq!(slice.verbose_value_iter(), None);
                assert_eq!(
                    slice.typed_payload(),
                    Err(TypedPayloadError::UnknownMessageInfo(info))
                );
            }
        }
    }
} // mod dlt_packet_slice
