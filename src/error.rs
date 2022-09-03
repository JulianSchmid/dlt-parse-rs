use core::str::Utf8Error;
use core::fmt;
use super::*;

#[cfg(feature = "std")]
use std::io;

#[cfg(test)]
use alloc::format;

/// Error in which an error occured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layer {
    /// Error occured while parsing or writing the DLT header.
    DltHeader,
    /// Error occured while parsing or writing a verbose type info.
    VerboseTypeInfo,
    /// Error occured while parsing or writing a verbose value.
    VerboseValue,
}

#[cfg(test)]
mod layer_test {
    use super::*;

    #[test]
    fn clone_eq() {
        use Layer::*;
        assert_eq!(VerboseTypeInfo, VerboseTypeInfo.clone());
    }

    #[test]
    fn debug() {
        use Layer::*;
        assert_eq!(
            "VerboseTypeInfo",
            format!("{:?}", VerboseTypeInfo)
        );
    }
}

/// Error if the length field in a DLT headeris smaller then the header the calculated
/// header size based on the flags (+ minimum payload size of 4 bytes/octetets)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DltMessageLengthTooSmallError {
    pub required_length: usize,
    pub actual_length: usize,
}

impl fmt::Display for DltMessageLengthTooSmallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DLT Header Error: The message length of {} present in the dlt header is smaller then minimum required size of {} bytes.",
            self.actual_length,
            self.required_length
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DltMessageLengthTooSmallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod dlt_message_length_too_small_error_test {
    use super::*;

    #[test]
    fn clone_eq() {
        let v = DltMessageLengthTooSmallError {
            required_length: 1,
            actual_length: 2,
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        let v = DltMessageLengthTooSmallError {
            required_length: 1,
            actual_length: 2,
        };
        assert_eq!(
            format!(
                "DltMessageLengthTooSmallError {{ required_length: {}, actual_length: {} }}",
                v.required_length,
                v.actual_length,
            ),
            format!("{:?}", v)
        );
    }

    #[test]
    fn display() {
        let v = DltMessageLengthTooSmallError {
            required_length: 1,
            actual_length: 2,
        };
        assert_eq!(
            format!(
                "DLT Header Error: The message length of {} present in the dlt header is smaller then minimum required size of {} bytes.",
                v.actual_length,
                v.required_length,
            ),
            format!("{}", v)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        assert!(
            DltMessageLengthTooSmallError {
                required_length: 1,
                actual_length: 2,
            }.source().is_none()
        );
    }
}

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
    UnexpectedEndOfSlice(UnexpectedEndOfSliceError)
}

impl fmt::Display for PacketSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PacketSliceError::*;
        match self {
            UnsupportedDltVersion(v) => v.fmt(f),
            MessageLengthTooSmall(v) => v.fmt(f),
            UnexpectedEndOfSlice(v) => v.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
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
mod packet_slice_error_test {
    use super::*;

    #[test]
    fn clone_eq() {
        use PacketSliceError::*;
        let v = UnsupportedDltVersion(UnsupportedDltVersionError{
            unsupported_version: 123,
        });
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        use PacketSliceError::*;
        let inner = UnsupportedDltVersionError{
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
            let inner = UnsupportedDltVersionError{
                unsupported_version: 123,
            };
            assert_eq!(
                format!("{}", inner),
                format!("{}", UnsupportedDltVersion(inner.clone())),
            );
        }
        {
            let inner = DltMessageLengthTooSmallError{
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
        use PacketSliceError::*;
        use std::error::Error;
        assert!(
            UnsupportedDltVersion(UnsupportedDltVersionError{
                unsupported_version: 123,
            }).source().is_some()
        );
        assert!(MessageLengthTooSmall(DltMessageLengthTooSmallError{
                actual_length: 1,
                required_length: 2,
            }).source().is_some()
        );
        assert!(
            UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                actual_size: 1,
                layer: Layer::DltHeader,
                minimum_size: 3,
            }).source().is_some()
        );
    }
}

/// Error if a slice did not contain enough data to decode a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnexpectedEndOfSliceError {
    /// Layer in which the length error was detected.
    pub layer: Layer,

    /// Minimum expected slice length.
    pub minimum_size: usize,

    /// Actual slice length (which was too small).
    pub actual_size: usize,
}

impl fmt::Display for UnexpectedEndOfSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.",
            self.layer,
            self.actual_size,
            self.minimum_size
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnexpectedEndOfSliceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod unexpected_end_of_slice_error_test {
    use super::*;

    #[test]
    fn clone_eq() {
        let v = UnexpectedEndOfSliceError{
            layer: Layer::DltHeader,
            minimum_size: 2,
            actual_size: 3,
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        let v = UnexpectedEndOfSliceError{
            layer: Layer::DltHeader,
            minimum_size: 2,
            actual_size: 3,
        };
        assert_eq!(
            format!(
                "UnexpectedEndOfSliceError {{ layer: {:?}, minimum_size: {}, actual_size: {} }}",
                v.layer,
                v.minimum_size,
                v.actual_size
            ),
            format!("{:?}", v)
        );
    }

    #[test]
    fn display() {
        let v = UnexpectedEndOfSliceError{
            layer: Layer::DltHeader,
            minimum_size: 2,
            actual_size: 3,
        };
        assert_eq!(
            format!(
                "{:?}: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.",
                v.layer,
                v.actual_size,
                v.minimum_size,
            ),
            format!("{}", v)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        assert!(UnexpectedEndOfSliceError{
            layer: Layer::DltHeader,
            minimum_size: 2,
            actual_size: 3,
        }.source().is_none());
    }
}

/// Error that is triggered when an unsupported DLT version is
/// encountred when parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedDltVersionError {
    /// Unsupported version number that was encountered in the DLT header.
    pub unsupported_version: u8,
}

impl fmt::Display for UnsupportedDltVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Encountered unsupported DLT version '{}' in header. Only versions {:?} are supported.",
            self.unsupported_version,
            DltHeader::SUPPORTED_DECODABLE_VERSIONS
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnsupportedDltVersionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod unsupported_dlt_version_error_test {
    use super::*;

    #[test]
    fn clone_eq() {
        let v = UnsupportedDltVersionError{
            unsupported_version: 123,
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        let v = UnsupportedDltVersionError{
            unsupported_version: 123,
        };
        assert_eq!(
            format!(
                "Encountered unsupported DLT version '{}' in header. Only versions {:?} are supported.",
                v.unsupported_version,
                crate::DltHeader::SUPPORTED_DECODABLE_VERSIONS
            ),
            format!("{}", v)
        );
    }

    #[test]
    fn display() {
        let v = UnsupportedDltVersionError{
            unsupported_version: 123,
        };
        assert_eq!(
            format!(
                "UnsupportedDltVersionError {{ unsupported_version: {} }}",
                v.unsupported_version
            ),
            format!("{:?}", v)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        assert!(
            UnsupportedDltVersionError{
                unsupported_version: 123,
            }.source().is_none()
        );
    }
}

/// Error that can occur if the data in an DltHeader can not be encoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DltHeaderEncodeError {
    /// Error that occurs when the given version number in the header was
    /// larger [`DltHeader::MAX_VERSION`].
    VersionTooLarge(u8),
}

impl fmt::Display for DltHeaderEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DltHeaderEncodeError::*;
        match self {
            VersionTooLarge(version) => write!(
                f,
                "DLT Header Encode Error: Version value '{}' is not encodable (maximum allowed value is {}).",
                version,
                crate::MAX_VERSION,
            )
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DltHeaderEncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod dlt_header_encode_error_test {
    use super::*;

    #[test]
    fn clone_eq() {
        use DltHeaderEncodeError::*;
        let v = VersionTooLarge(123);
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        use DltHeaderEncodeError::*;
        assert_eq!(
            format!("VersionTooLarge({})", 123),
            format!("{:?}", VersionTooLarge(123))
        );
    }

    #[test]
    fn display() {
        use DltHeaderEncodeError::*;
        assert_eq!(
            format!(
                "DLT Header Encode Error: Version value '{}' is not encodable (maximum allowed value is {}).",
                123,
                crate::MAX_VERSION
            ),
            format!("{}", VersionTooLarge(123))
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use DltHeaderEncodeError::*;
        use std::error::Error;
        assert!(VersionTooLarge(123).source().is_none());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerboseDecodeError {
    /// Error that occurs if a type info is inconsistent.
    ///
    /// An example is when the type is described as both a
    /// bool and float or otherwise contains flags that contradict
    /// each other.
    ///
    /// The encoded type info is given as an argument.
    InvalidTypeInfo([u8;4]),

    /// Error in case an invalid bool value is encountered (not 0 or 1).
    InvalidBoolValue(u8),

    /// Error if not enough data was present in the slice to decode
    /// a verbose value.
    UnexpectedEndOfSlice(UnexpectedEndOfSliceError),

    /// Error when decoding an string (can also occur for variable names or unit names).
    Utf8(Utf8Error),

    /// Error in case value decoding is not yet supported.
    ///
    /// TODO: Remove this value
    Unsupported,
}

impl From<Utf8Error> for VerboseDecodeError {
    fn from(err: Utf8Error) -> VerboseDecodeError {
        VerboseDecodeError::Utf8(err)
    }
}

impl fmt::Display for VerboseDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use VerboseDecodeError::*;
        match self {
            InvalidTypeInfo(value) => write!(
                f, "DLT Verbose Message Field: Encountered an invalid typeinfo {:?} (contradicting or unknown)", value
            ),
            InvalidBoolValue(value) => write!(
                f, "DLT Verbose Message Field: Encountered invalid bool value '{}' (only 0 or 1 are valid)", value
            ),
            UnexpectedEndOfSlice(err) => err.fmt(f),
            Utf8(err) => err.fmt(f),
            Unsupported => write!(
                f, "DLT Verbose Message Field: Unsupported field type"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VerboseDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use VerboseDecodeError::*;
        match self {
            InvalidTypeInfo(_) => None,
            InvalidBoolValue(_) => None,
            UnexpectedEndOfSlice(err) => Some(err),
            Utf8(err) => Some(err),
            Unsupported => None,
        }
    }
}

#[cfg(test)]
mod verbose_decode_error_tests {
    use super::*;

    #[test]
    fn clone_eq() {
        use VerboseDecodeError::*;
        let v = InvalidBoolValue(2);
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        use VerboseDecodeError::*;
        let v = InvalidBoolValue(2);
        assert_eq!(
            format!("InvalidBoolValue({})", 2),
            format!("{:?}", v)
        );
    }

    #[test]
    fn display() {
        use VerboseDecodeError::*;

        assert_eq!(
            format!("DLT Verbose Message Field: Encountered an invalid typeinfo {:?} (contradicting or unknown)", [1,2,3,4]),
            format!("{}", InvalidTypeInfo([1,2,3,4]))
        );

        assert_eq!(
            format!("DLT Verbose Message Field: Encountered invalid bool value '{}' (only 0 or 1 are valid)", 2),
            format!("{}", InvalidBoolValue(2))
        );

        {
            let v = UnexpectedEndOfSliceError{
                layer: Layer::DltHeader,
                actual_size: 1,
                minimum_size: 2,
            };
            assert_eq!(
                format!("{}", v),
                format!("{}", UnexpectedEndOfSlice(v))
            );
        }
        {
            let v = std::str::from_utf8(&[0, 159, 146, 150]).unwrap_err();
            assert_eq!(
                format!("{}", v),
                format!("{}", Utf8(v))
            );
        }
        assert_eq!(
            format!("DLT Verbose Message Field: Unsupported field type"),
            format!("{}", Unsupported)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use VerboseDecodeError::*;
        use std::error::Error;
        assert!(InvalidTypeInfo([1,2,3,4]).source().is_none());
        assert!(InvalidBoolValue(2).source().is_none());
        assert!(
            UnexpectedEndOfSlice(UnexpectedEndOfSliceError{
                layer: Layer::DltHeader,
                actual_size: 1,
                minimum_size: 2,
            }).source().is_some()
        );
        assert!(
            Utf8(std::str::from_utf8(&[0, 159, 146, 150]).unwrap_err())
            .source()
            .is_some()
        );
        assert!(Unsupported.source().is_none());
    }

    #[test]
    fn from_utf8_error() {
        let e: VerboseDecodeError = std::str::from_utf8(&[0, 159, 146, 150]).unwrap_err().into();
        assert_matches!(e, VerboseDecodeError::Utf8(_));
    }
}

/// Error that occurs when another pattern then
/// [`crate::storage::StorageHeader::PATTERN_AT_START`] is encountered
/// at the start when parsing a StorageHeader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageHeaderStartPatternError {
    /// Encountered pattern at the start.
    pub actual_pattern: [u8;4],
}

impl fmt::Display for StorageHeaderStartPatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "Error when parsing DLT storage header. Expected pattern {:?} at start but got {:?}",
            super::storage::StorageHeader::PATTERN_AT_START,
            self.actual_pattern
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StorageHeaderStartPatternError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod storage_header_start_pattern_error_tests {
    use super::*;

    #[test]
    fn clone_eq() {
        let v = StorageHeaderStartPatternError{
            actual_pattern: [1,2,3,4]
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        let v = StorageHeaderStartPatternError{
            actual_pattern: [1,2,3,4]
        };
        assert_eq!(
            format!(
                "StorageHeaderStartPatternError {{ actual_pattern: {:?} }}",
                v.actual_pattern
            ),
            format!("{:?}", v)
        );
    }

    #[test]
    fn display() {
        let v = StorageHeaderStartPatternError{
            actual_pattern: [1,2,3,4]
        };
        assert_eq!(
            format!(
                "Error when parsing DLT storage header. Expected pattern {:?} at start but got {:?}",
                crate::storage::StorageHeader::PATTERN_AT_START,
                v.actual_pattern
            ),
            format!("{}", v)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        assert!(
            StorageHeaderStartPatternError{
                actual_pattern: [1,2,3,4]
            }.source().is_none()
        );
    }
}

///Errors that can occure on reading a dlt header.
#[cfg(feature = "std")]
#[derive(Debug)]
pub enum ReadError {
    /// Error if the slice is smaller then dlt length field or minimal size.
    UnexpectedEndOfSlice(UnexpectedEndOfSliceError),

    /// Error if the dlt length is smaller then the header the calculated header size based on the flags (+ minimum payload size of 4 bytes/octetets)
    DltMessageLengthTooSmall(DltMessageLengthTooSmallError),

    StorageHeaderStartPattern(StorageHeaderStartPatternError),
    
    /// Standard io error.
    IoError(io::Error),
}

#[cfg(feature = "std")]
impl From<StorageHeaderStartPatternError> for ReadError {
    fn from(err: StorageHeaderStartPatternError) -> ReadError {
        ReadError::StorageHeaderStartPattern(err)
    }
}

#[cfg(feature = "std")]
impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::IoError(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ReadError::*;
        match self {
            UnexpectedEndOfSlice(ref err) => Some(err),
            DltMessageLengthTooSmall(ref err) => Some(err),
            StorageHeaderStartPattern(ref err) => Some(err),
            IoError(ref err) => Some(err),
        }
    }
}

#[cfg(feature = "std")]
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ReadError::*;

        match self {
            UnexpectedEndOfSlice(err) => {
                write!(f, "ReadError: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.", err.actual_size, err.minimum_size)
            }
            DltMessageLengthTooSmall(err) => err.fmt(f),
            StorageHeaderStartPattern(err) => err.fmt(f),
            IoError(err) => err.fmt(f),
        }
    }
}

/// Tests for `ReadError` methods
#[cfg(all(feature = "std", test))]
mod read_error {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn debug() {
        use ReadError::*;

        {
            let c = UnexpectedEndOfSliceError {
                minimum_size: 1,
                actual_size: 2,
                layer: Layer::DltHeader,
            };
            assert_eq!(
                format!("UnexpectedEndOfSlice({:?})", c),
                format!("{:?}", UnexpectedEndOfSlice(c))
            );
        }
        {
            let c = DltMessageLengthTooSmallError{
                required_length: 3,
                actual_length: 4,
            };
            assert_eq!(
                format!("DltMessageLengthTooSmall({:?})", c),
                format!("{:?}", DltMessageLengthTooSmall(c))
            );
        }
        {
            let c = std::io::Error::new(std::io::ErrorKind::Other, "oh no!");
            assert_eq!(
                format!("IoError({:?})", c),
                format!("{:?}", IoError(c))
            );
        }
    }

    proptest! {
        #[test]
        fn display(
            usize0 in any::<usize>(),
            usize1 in any::<usize>(),
        ) {

            use ReadError::*;

            //UnexpectedEndOfSlice
            assert_eq!(
                &format!("ReadError: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.", usize1, usize0),
                &format!(
                    "{}",
                    UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError {
                            layer: Layer::DltHeader,
                            minimum_size: usize0,
                            actual_size: usize1,
                        }
                    )
                )
            );

            // DltMessageLengthTooSmall
            {
                let c = DltMessageLengthTooSmallError{
                    required_length: usize0,
                    actual_length: usize1
                };
                assert_eq!(
                    &format!("{}", c),
                    &format!("{}", DltMessageLengthTooSmall(c))
                );
            }

            // StorageHeaderStartPattern
            {
                let c = StorageHeaderStartPatternError{
                    actual_pattern: [1,2,3,4]
                };
                assert_eq!(
                    &format!("{}", c),
                    &format!("{}", StorageHeaderStartPattern(c))
                );
            }

            //IoError
            {
                let custom_error = std::io::Error::new(std::io::ErrorKind::Other, "some error");
                assert_eq!(
                    &format!("{}", custom_error),
                    &format!("{}", IoError(custom_error))
                );
            }
        }
    }

    #[test]
    fn source() {
        use ReadError::*;
        use std::error::Error;

        assert!(
            UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError {
                    layer: Layer::DltHeader,
                    minimum_size: 1,
                    actual_size: 2
                }
            )
            .source()
            .is_some());
        assert!(
            DltMessageLengthTooSmall(
                DltMessageLengthTooSmallError {
                    required_length: 3,
                    actual_length: 4
                }
            )
            .source()
            .is_some()
        );
        assert!(
            StorageHeaderStartPattern(
                StorageHeaderStartPatternError{
                    actual_pattern: [1,2,3,4]
                }
            )
            .source()
            .is_some()
        );
        assert!(
            IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
            .source()
            .is_some()
        );
    }

    #[test]
    fn from_io_error() {
        let r: ReadError = std::io::Error::new(std::io::ErrorKind::Other, "oh no!").into();
        assert_matches!(
            r,
            ReadError::IoError(_)
        );
    }

    #[test]
    fn from_storage_header_error() {
        let r: ReadError = StorageHeaderStartPatternError {
            actual_pattern: [1,2,3,4],
        }.into();
        assert_matches!(
            r,
            ReadError::StorageHeaderStartPattern(_)
        );
    }
} // mod read_error

///Errors that can occur when serializing a dlt header.
#[cfg(feature = "std")]
#[derive(Debug)]
pub enum WriteError {
    VersionTooLarge(u8),
    IoError(io::Error),
}

#[cfg(feature = "std")]
impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> WriteError {
        WriteError::IoError(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WriteError::IoError(ref err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(feature = "std")]
impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WriteError::*;

        match self {
            VersionTooLarge(version) => {
                write!(
                    f,
                    "WriteError: DLT version {} is larger then the maximum supported value of {}",
                    version, crate::MAX_VERSION
                )
            }
            IoError(err) => err.fmt(f),
        }
    }
}

/// Tests for `WriteError` methods
#[cfg(all(feature = "std", test))]
mod write_error {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn debug() {
        use WriteError::*;
        assert_eq!(
            "VersionTooLarge(123)",
            format!("{:?}", VersionTooLarge(123))
        );
    }

    proptest! {
        #[test]
        fn display(version in any::<u8>()) {
            use WriteError::*;

            // VersionTooLarge
            assert_eq!(
                &format!("WriteError: DLT version {} is larger then the maximum supported value of {}", version, crate::MAX_VERSION),
                &format!("{}", VersionTooLarge(version))
            );

            //IoError
            {
                let custom_error = std::io::Error::new(std::io::ErrorKind::Other, "some error");
                assert_eq!(
                    &format!("{}", custom_error),
                    &format!("{}", IoError(custom_error))
                );
            }
        }
    }

    #[test]
    fn source() {
        use std::error::Error;
        use WriteError::*;

        assert!(VersionTooLarge(123).source().is_none());
        assert!(
            IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
                .source()
                .is_some()
        );
    }
} // mod write_error

/// Error that can occur when an out of range value is passed to a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RangeError {
    /// Error if the user defined value is outside the range of 7-15
    NetworkTypekUserDefinedOutsideOfRange(u8),
}

#[cfg(feature = "std")]
impl std::error::Error for RangeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RangeError::*;

        match self {
            NetworkTypekUserDefinedOutsideOfRange(value) => {
                write!(f, "RangeError: Message type info field user defined value of {} outside of the allowed range of 7-15.", value)
            }
        }
    }
}

#[cfg(test)]
mod range_error_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn clone_eq() {
        use RangeError::*;
        let v = NetworkTypekUserDefinedOutsideOfRange(123);
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        use RangeError::*;
        let v = NetworkTypekUserDefinedOutsideOfRange(123);
        assert_eq!(
            "NetworkTypekUserDefinedOutsideOfRange(123)",
            format!("{:?}", v)
        );
    }

    proptest! {
        #[test]
        fn display(value in any::<u8>()) {
            use RangeError::*;

            // NetworkTypekUserDefinedOutsideOfRange
            assert_eq!(
                &format!("RangeError: Message type info field user defined value of {} outside of the allowed range of 7-15.", value),
                &format!("{}", NetworkTypekUserDefinedOutsideOfRange(value))
            );
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn source() {
        use std::error::Error;
        use RangeError::*;

        assert!(NetworkTypekUserDefinedOutsideOfRange(123)
            .source()
            .is_none());
    }
} // mod range_error
