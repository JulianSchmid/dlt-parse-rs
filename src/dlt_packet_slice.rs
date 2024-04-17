use crate::verbose::VerboseIter;

use super::*;

#[cfg(feature = "std")]
const SET_LOG_LEVEL: &[u8] = "set_log_level".as_bytes();
#[cfg(feature = "std")]
const SET_TRACE_STATUS: &[u8] = "set_trace_status".as_bytes();
#[cfg(feature = "std")]
const GET_LOG_INFO: &[u8] = "get_log_info".as_bytes();
#[cfg(feature = "std")]
const GET_DEFAULT_LOG_LEVEL: &[u8] = "get_default_log_level".as_bytes();
#[cfg(feature = "std")]
const STORE_CONFIGURATION: &[u8] = "store_configuration".as_bytes();
#[cfg(feature = "std")]
const RESET_TO_FACTORY_DEFAULT: &[u8] = "reset_to_factory_default".as_bytes();
#[cfg(feature = "std")]
const SET_MESSAGE_FILTERING: &[u8] = "set_message_filtering".as_bytes();
#[cfg(feature = "std")]
const SET_DEFAULT_LOG_LEVEL: &[u8] = "set_default_log_level".as_bytes();
#[cfg(feature = "std")]
const SET_DEFAULT_TRACE_STATUS: &[u8] = "set_default_trace_status".as_bytes();
#[cfg(feature = "std")]
const GET_SOFTWARE_VERSION: &[u8] = "get_software_version".as_bytes();
#[cfg(feature = "std")]
const GET_DEFAULT_TRACE_STATUS: &[u8] = "get_default_trace_status".as_bytes();
#[cfg(feature = "std")]
const GET_LOG_CHANNEL_NAMES: &[u8] = "get_log_channel_names".as_bytes();
#[cfg(feature = "std")]
const GET_TRACE_STATUS: &[u8] = "get_trace_status".as_bytes();
#[cfg(feature = "std")]
const SET_LOG_CHANNEL_ASSIGNMENT: &[u8] = "set_log_channel_assignment".as_bytes();
#[cfg(feature = "std")]
const SET_LOG_CHANNEL_THRESHOLD: &[u8] = "set_log_channel_threshold".as_bytes();
#[cfg(feature = "std")]
const GET_LOG_CHANNEL_THRESHOLD: &[u8] = "get_log_channel_threshold".as_bytes();
#[cfg(feature = "std")]
const BUFFER_OVERFLOW_NOTIFICATION: &[u8] = "buffer_overflow_notification".as_bytes();
#[cfg(feature = "std")]
const SYNC_TIME_STAMP: &[u8] = "sync_time_stamp".as_bytes();
#[cfg(feature = "std")]
const CALL_SWC_INJECTIONS: &[u8] = "call_swc_injections".as_bytes();
#[cfg(feature = "std")]
const DEPRECATED_COMMAND_NAME: &[u8] = "deprecated_command_name".as_bytes();

const OK: &[u8] = "ok".as_bytes();
const NOT_SUPPORTED: &[u8] = "not_supported".as_bytes();
const ERROR: &[u8] = "error".as_bytes();

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ControlMessage<'a> {
    message_id: u32,
    status: Option<&'a [u8]>,
    non_verbose_payload: &'a [u8],
}

#[cfg(feature = "std")]
impl std::io::Write for ControlMessage<'_> {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        let mut control_message = std::vec::Vec::with_capacity(16);
        control_message.push(91);
        control_message.extend_from_slice(self.service_name());
        if let Some(status) = self.status {
            control_message.push(32);
            control_message.extend_from_slice(status);
        }
        control_message.extend_from_slice(&[93, 32]);
        control_message.extend_from_slice(self.non_verbose_payload);
        control_message.push(10);
        io::stdout().write(&control_message)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

#[cfg(feature = "std")]
impl ControlMessage<'_> {
    fn service_name(&self) -> &'static [u8] {
        match self.message_id {
            0x01 => SET_LOG_LEVEL,
            0x02 => SET_TRACE_STATUS,
            0x03 => GET_LOG_INFO,
            0x04 => GET_DEFAULT_LOG_LEVEL,
            0x05 => STORE_CONFIGURATION,
            0x06 => RESET_TO_FACTORY_DEFAULT,
            0x0A => SET_MESSAGE_FILTERING,
            0x11 => SET_DEFAULT_LOG_LEVEL,
            0x12 => SET_DEFAULT_TRACE_STATUS,
            0x13 => GET_SOFTWARE_VERSION,
            0x15 => GET_DEFAULT_TRACE_STATUS,
            0x17 => GET_LOG_CHANNEL_NAMES,
            0x1F => GET_TRACE_STATUS,
            0x20 => SET_LOG_CHANNEL_ASSIGNMENT,
            0x21 => SET_LOG_CHANNEL_THRESHOLD,
            0x22 => GET_LOG_CHANNEL_THRESHOLD,
            0x23 => BUFFER_OVERFLOW_NOTIFICATION,
            0x24 => SYNC_TIME_STAMP,
            0xFFF..=0xFFFFFFFF => CALL_SWC_INJECTIONS,
            _ => DEPRECATED_COMMAND_NAME,
        }
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
    pub fn typed_payload(&self) -> Option<DltTypedPayload<'a>> {
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
                        DltMessageType::Control(_) => {
                            return Some(DltTypedPayload::ControlV(ControlVPayload {
                                info: message_info,
                                iter: VerboseIter::new(
                                    is_big_endian,
                                    u16::from(number_of_arguments),
                                    self.payload(),
                                ),
                            }));
                        }
                        DltMessageType::Log(_) => {
                            return Some(DltTypedPayload::LogV(LogVPayload {
                                info: message_info,
                                iter: VerboseIter::new(
                                    is_big_endian,
                                    u16::from(number_of_arguments),
                                    self.payload(),
                                ),
                            }));
                        }
                        DltMessageType::NetworkTrace(_) => {
                            return Some(DltTypedPayload::NetworkV(NetworkVPayload {
                                info: message_info,
                                iter: VerboseIter::new(
                                    is_big_endian,
                                    u16::from(number_of_arguments),
                                    self.payload(),
                                ),
                            }));
                        }
                        DltMessageType::Trace(_) => {
                            return Some(DltTypedPayload::TraceV(TraceVPayload {
                                info: message_info,
                                iter: VerboseIter::new(
                                    is_big_endian,
                                    u16::from(number_of_arguments),
                                    self.payload(),
                                ),
                            }));
                        }
                    }
                }
                return None;
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
                    Some(DltMessageType::Control(DltControlMessageType::Response)) => {
                        return determine_dlt_typed_playload_for_non_verbose_response(
                            non_verbose_payload,
                            message_id,
                            message_info,
                        );
                    }
                    Some(DltMessageType::Control(DltControlMessageType::Request)) => {
                        return Some(DltTypedPayload::ControlNv(ControlNvPayload {
                            info: message_info,
                            msg_id: message_id,
                            payload: non_verbose_payload,
                            control_message: Some(ControlMessage {
                                non_verbose_payload,
                                message_id,
                                status: None,
                            }),
                        }));
                    }
                    Some(DltMessageType::Log(_)) => {
                        return Some(DltTypedPayload::LogNv(LogNvPayload {
                            info: message_info,
                            msg_id: message_id,
                            payload: non_verbose_payload,
                        }));
                    }
                    Some(DltMessageType::NetworkTrace(_)) => {
                        return Some(DltTypedPayload::NetworkNv(NetworkNvPayload {
                            info: message_info,
                            msg_id: message_id,
                            payload: non_verbose_payload,
                        }));
                    }
                    Some(DltMessageType::Trace(_)) => {
                        return Some(DltTypedPayload::TraceNv(TraceNvPayload {
                            info: message_info,
                            msg_id: message_id,
                            payload: non_verbose_payload,
                        }));
                    }
                    None => {
                        return Some(DltTypedPayload::GenericNv(GenericNvPayload {
                            info: message_info,
                            msg_id: message_id,
                            payload: non_verbose_payload,
                        }));
                    }
                }
            }
        }
        None
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

fn determine_dlt_typed_playload_for_non_verbose_response(
    non_verbose_payload: &[u8],
    message_id: u32,
    message_info: Option<DltMessageInfo>,
) -> Option<DltTypedPayload<'_>> {
    if non_verbose_payload.len() > 5 {
        let control_message = match non_verbose_payload[0] {
            0 => Some(ControlMessage {
                non_verbose_payload: &non_verbose_payload[5..],
                message_id,
                status: Some(OK),
            }),
            1 => Some(ControlMessage {
                non_verbose_payload: &non_verbose_payload[5..],
                message_id,
                status: Some(NOT_SUPPORTED),
            }),
            2 => Some(ControlMessage {
                non_verbose_payload: &non_verbose_payload[5..],
                message_id,
                status: Some(ERROR),
            }),
            _ => None,
        };

        return Some(DltTypedPayload::ControlNv(ControlNvPayload {
            info: message_info,
            msg_id: message_id,
            payload: non_verbose_payload,
            control_message,
        }));
    }
    None
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

    #[test]
    fn payload_methods() {
        //pairs of (header, expected_non_verbose)
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
                buffer.try_extend_from_slice(&[0x10, 0x11]).unwrap();

                // slice
                let slice = DltPacketSlice::from_slice(&buffer).unwrap();
                if t.1 {
                    let expected_message_id = 0x1234_5678u32;
                    let expected_payload = &[0x10u8, 0x11][..];
                    let expected_message_info =
                        t.0.extended_header.as_ref().map(|v| v.message_info);

                    assert_eq!(Some(expected_message_id), slice.message_id());
                    assert_eq!(
                        Some((expected_message_id, expected_payload)),
                        slice.message_id_and_payload()
                    );
                    assert_eq!(Some(expected_payload), slice.non_verbose_payload());
                    assert_eq!(None, slice.verbose_value_iter());

                    if slice.header_len + 4 <= slice.slice.len() && slice.has_extended_header() {
                        assert_eq!(
                            Some(DltTypedPayload::GenericNv(GenericNvPayload {
                                info: expected_message_info,
                                msg_id: expected_message_id,
                                payload: expected_payload,
                            })),
                            slice.typed_payload()
                        );
                    } else {
                        assert_eq!(None, slice.typed_payload());
                    }
                } else {
                    let ext = t.0.extended_header.clone().unwrap();
                    let p_start = 0x1234_5678u32.to_be_bytes();
                    let payload = [p_start[0], p_start[1], p_start[2], p_start[3], 0x10, 0x11];
                    let expected_iter =
                        VerboseIter::new(true, u16::from(ext.number_of_arguments), &payload);

                    assert_eq!(None, slice.message_id());
                    assert_eq!(None, slice.message_id_and_payload());
                    assert_eq!(None, slice.non_verbose_payload());

                    assert_eq!(Some(expected_iter.clone()), slice.verbose_value_iter());
                    assert_eq!(None, slice.typed_payload());
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
                buffer.try_extend_from_slice(&[0x10, 0x11]).unwrap();

                // slice
                let slice = DltPacketSlice::from_slice(&buffer).unwrap();
                if t.1 {
                    let expected_message_id = 0x1234_5678u32;
                    let expected_payload = &[0x10u8, 0x11][..];
                    let expected_message_info =
                        t.0.extended_header.as_ref().map(|v| v.message_info);

                    assert_eq!(Some(expected_message_id), slice.message_id());
                    assert_eq!(
                        Some((expected_message_id, expected_payload)),
                        slice.message_id_and_payload()
                    );
                    assert_eq!(Some(expected_payload), slice.non_verbose_payload());
                    assert_eq!(None, slice.verbose_value_iter());

                    if slice.header_len + 4 <= slice.slice.len() && slice.has_extended_header() {
                        assert_eq!(
                            Some(DltTypedPayload::GenericNv(GenericNvPayload {
                                info: expected_message_info,
                                msg_id: expected_message_id,
                                payload: expected_payload,
                            })),
                            slice.typed_payload()
                        );
                    } else {
                        assert_eq!(None, slice.typed_payload());
                    }
                } else {
                    let ext = t.0.extended_header.clone().unwrap();
                    let p_start = 0x1234_5678u32.to_le_bytes();
                    let payload = [p_start[0], p_start[1], p_start[2], p_start[3], 0x10, 0x11];
                    let expected_iter =
                        VerboseIter::new(false, u16::from(ext.number_of_arguments), &payload);

                    assert_eq!(None, slice.message_id());
                    assert_eq!(None, slice.message_id_and_payload());
                    assert_eq!(None, slice.non_verbose_payload());

                    assert_eq!(Some(expected_iter.clone()), slice.verbose_value_iter());
                    assert_eq!(None, slice.typed_payload());
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
                let slice =
                    DltPacketSlice::from_slice(&buffer[..buffer.len() - missing_len]).unwrap();
                assert_eq!(None, slice.message_id());
                assert_eq!(None, slice.message_id_and_payload());
                assert_eq!(None, slice.non_verbose_payload());
                if t.1 {
                    assert_eq!(None, slice.typed_payload());
                }
            }
        }
    }
} // mod dlt_packet_slice
