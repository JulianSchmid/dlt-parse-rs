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
    Utf8(std::str::Utf8Error),

    /// Error in case value decoding is not yet supported.
    ///
    /// TODO: Remove this value
    Unsupported,
}

impl From<std::str::Utf8Error> for VerboseDecodeError {
    fn from(err: std::str::Utf8Error) -> VerboseDecodeError {
        VerboseDecodeError::Utf8(err)
    }
}

/// Error that occurs when another pattern then
/// [`StorageHeader::PATTERN_AT_START`] is encountered at the start
/// when parsing a StorageHeader.
#[derive(Debug, PartialEq, Eq)]
pub struct StorageHeaderStartPatternError {
    /// Encountered pattern at the start.
    pub actual_pattern: [u8;4],
}

impl std::fmt::Display for StorageHeaderStartPatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "Error when parsing DLT storage header. Expected pattern {:?} at start but got {:?}",
            super::StorageHeader::PATTERN_AT_START,
            self.actual_pattern
        )
    }
}

impl std::error::Error for StorageHeaderStartPatternError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
