use crate::DltMessageInfo;

/// Error that can occur when trying to get a [`crate::DltTypedPayload`] from
/// a [`crate::DltPacketSlice`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedPayloadError {
    /// Error if the length of the message is smaller than a message id.
    LenSmallerThanMessageId {
        packet_len: usize,
        header_len: usize,
    },

    /// Error if the message info
    UnknownMessageInfo(DltMessageInfo),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for TypedPayloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl core::fmt::Display for TypedPayloadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use TypedPayloadError::*;
        match self {
            LenSmallerThanMessageId {
                packet_len,
                header_len,
            } => {
                write!(f, "DLT non verbose message too small for the message id (4 bytes required, only {packet_len} bytes present and {header_len}) bytes taken by the header.")
            }
            UnknownMessageInfo(message_info) => {
                write!(
                    f,
                    "DLT message info contains the value '{}' that is unknown",
                    message_info.0
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use proptest::prelude::*;

    #[test]
    fn clone_eq() {
        use TypedPayloadError::*;
        let v = LenSmallerThanMessageId {
            packet_len: 123,
            header_len: 234,
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        use TypedPayloadError::*;
        let v = LenSmallerThanMessageId {
            packet_len: 123,
            header_len: 234,
        };
        assert_eq!(
            "LenSmallerThanMessageId { packet_len: 123, header_len: 234 }",
            format!("{:?}", v)
        );
    }

    proptest! {
        #[test]
        fn display(value0_usize in any::<usize>(), value1_usize in any::<usize>(), value_u8 in any::<u8>()) {
            use TypedPayloadError::*;

            // LenSmallerThanMessageId
            assert_eq!(
                &format!("DLT non verbose message too small for the message id (4 bytes required, only {} bytes present and {}) bytes taken by the header.", value0_usize, value1_usize),
                &format!("{}", LenSmallerThanMessageId { packet_len: value0_usize, header_len: value1_usize })
            );

            // UnknownMessageInfo
            assert_eq!(
                &format!("DLT message info contains the value '{}' that is unknown", value_u8),
                &format!("{}", UnknownMessageInfo(DltMessageInfo(value_u8)))
            );
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn source() {
        use std::error::Error;
        use TypedPayloadError::*;

        assert!(LenSmallerThanMessageId {
            packet_len: 123,
            header_len: 234
        }
        .source()
        .is_none());
    }
} // mod tests
