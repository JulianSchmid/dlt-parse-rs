/// Error in which an error occured.
#[derive(Debug, PartialEq, Eq)]
pub enum Layer {
    /// Error occured while parsing or writing the DLT header.
    DltHeader,
    /// Error occured while parsing or writing the verbose type info.
    VerboseTypeInfo,
}

/// Error if a slice did not contain enough data to decode a value.
#[derive(Debug, PartialEq, Eq)]
pub struct UnexpectedEndOfSliceError {
    pub layer: Layer,
    pub minimum_size: usize,
    pub actual_size: usize,
}
