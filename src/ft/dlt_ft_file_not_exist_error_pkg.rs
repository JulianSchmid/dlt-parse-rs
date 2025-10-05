use super::*;

/// Error package sent if a file that should have been
/// transfered does not exists.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DltFtFileNotExistErrorPkg<'a> {
    /// Error code.
    pub error_code: DltFtErrorCode,

    /// Standard linux error code.
    pub linux_error_code: DltFtInt,

    /// Absolute path to the file.
    pub file_name: &'a str,
}

impl<'a> DltFtFileNotExistErrorPkg<'a> {
    /// Verbose string at the start and end of the "DLT File Transfer Error" package.
    pub const PKG_FLAG: &'static str = "FLER";

    /// Number of verbose arguments in a file transfer "file does not exist error
    /// package" written in the DLT extended header.
    pub const NUM_ARGS: u16 = 5;
}
