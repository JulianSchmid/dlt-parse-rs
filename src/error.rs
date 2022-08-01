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

    /// Error when decoding an string (can also be variable names or units).
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
