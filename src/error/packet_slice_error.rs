pub use super::*;

/// Errors that can occur when slicing a DLT packet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PacketSliceError {
    /// An unsupporetd version number has been encountered
    /// while decoding the header.
    UnsupportedDltVersion(UnsupportedDltVersionError),

    /// Error if the dlt length is smaller then the header the calculated
    /// header size based on the flags (+ minimum payload size of 4 bytes/octetets)
    MessageLengthTooSmall(DltMessageLengthTooSmallError),

    /// Error if a slice did not contain enough data to decode a value.
    UnexpectedEndOfSlice(UnexpectedEndOfSliceError),
}

impl core::fmt::Display for PacketSliceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use PacketSliceError::*;
        match self {
            UnsupportedDltVersion(v) => v.fmt(f),
            MessageLengthTooSmall(v) => v.fmt(f),
            UnexpectedEndOfSlice(v) => v.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for PacketSliceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use PacketSliceError::*;
        match self {
            UnsupportedDltVersion(v) => Some(v),
            MessageLengthTooSmall(v) => Some(v),
            UnexpectedEndOfSlice(v) => Some(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn clone_eq() {
        use PacketSliceError::*;
        let v = UnsupportedDltVersion(UnsupportedDltVersionError {
            unsupported_version: 123,
        });
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        use PacketSliceError::*;
        let inner = UnsupportedDltVersionError {
            unsupported_version: 123,
        };
        assert_eq!(
            format!("UnsupportedDltVersion({:?})", inner),
            format!("{:?}", UnsupportedDltVersion(inner.clone())),
        );
    }

    #[test]
    fn display() {
        use PacketSliceError::*;
        {
            let inner = UnsupportedDltVersionError {
                unsupported_version: 123,
            };
            assert_eq!(
                format!("{}", inner),
                format!("{}", UnsupportedDltVersion(inner.clone())),
            );
        }
        {
            let inner = DltMessageLengthTooSmallError {
                actual_length: 1,
                required_length: 2,
            };
            assert_eq!(
                format!("{}", inner),
                format!("{}", MessageLengthTooSmall(inner.clone())),
            );
        }
        {
            let inner = UnexpectedEndOfSliceError {
                actual_size: 1,
                layer: Layer::DltHeader,
                minimum_size: 3,
            };
            assert_eq!(
                format!("{}", inner),
                format!("{}", UnexpectedEndOfSlice(inner.clone())),
            );
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        use PacketSliceError::*;
        assert!(UnsupportedDltVersion(UnsupportedDltVersionError {
            unsupported_version: 123,
        })
        .source()
        .is_some());
        assert!(MessageLengthTooSmall(DltMessageLengthTooSmallError {
            actual_length: 1,
            required_length: 2,
        })
        .source()
        .is_some());
        assert!(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
            actual_size: 1,
            layer: Layer::DltHeader,
            minimum_size: 3,
        })
        .source()
        .is_some());
    }
}
