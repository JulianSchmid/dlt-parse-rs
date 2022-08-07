use core::str::Utf8Error;

/// Error in which an error occured.
#[derive(Debug, PartialEq, Eq)]
pub enum Layer {
    /// Error occured while parsing or writing the DLT header.
    DltHeader,
    /// Error occured while parsing or writing a verbose type info.
    VerboseTypeInfo,
    /// Error occured while parsing or writing a verbose value.
    VerboseValue,
}

/// Error if the length field in a DLT headeris smaller then the header the calculated
/// header size based on the flags (+ minimum payload size of 4 bytes/octetets)
#[derive(Debug, PartialEq, Eq)]
pub struct DltMessageLengthTooSmallError {
    pub required_length: usize,
    pub actual_length: usize,
}

impl core::fmt::Display for DltMessageLengthTooSmallError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "DLT Header Error: The message length of {} present in the dlt header is smaller then minimum required size of {} bytes.",
            self.actual_length,
            self.required_length
        )
    }
}

/// Errors that can occur when slicing a DLT packet.
#[derive(Debug, PartialEq, Eq)]
pub enum PacketSliceError {
    /// Error if the dlt length is smaller then the header the calculated
    /// header size based on the flags (+ minimum payload size of 4 bytes/octetets)
    MessageLengthTooSmall(DltMessageLengthTooSmallError),

    /// Error if a slice did not contain enough data to decode a value.
    UnexpectedEndOfSlice(UnexpectedEndOfSliceError)
}

/// Error if a slice did not contain enough data to decode a value.
#[derive(Debug, PartialEq, Eq)]
pub struct UnexpectedEndOfSliceError {
    /// Layer in which the length error was detected.
    pub layer: Layer,

    /// Minimum expected slice length.
    pub minimum_size: usize,

    /// Actual slice length (which was too small).
    pub actual_size: usize,
}

/// Error that can occur if the data in an DltHeader can not be encoded.
#[derive(Debug, PartialEq, Eq)]
pub enum DltHeaderEncodeError {
    /// Error that occurs when the given version number in the header was
    /// larger [`DltHeader::MAX_VERSION`].
    VersionTooLarge(u8),
}

impl core::fmt::Display for DltHeaderEncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

#[derive(Debug, PartialEq, Eq)]
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

/// Error that occurs when another pattern then
/// [`crate::storage::StorageHeader::PATTERN_AT_START`] is encountered
/// at the start when parsing a StorageHeader.
#[derive(Debug, PartialEq, Eq)]
pub struct StorageHeaderStartPatternError {
    /// Encountered pattern at the start.
    pub actual_pattern: [u8;4],
}

impl core::fmt::Display for StorageHeaderStartPatternError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
