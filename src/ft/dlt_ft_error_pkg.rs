use super::*;

/// Error package sent when an error occured with an
/// existing file.
///
/// If a files does not exist
/// [`crate::ft::DltFileNotExistErrorPkg`] is sent instead.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DltFtErrorPkg<'a, 'b> {
    /// Error code.
    pub error_code: DltFtErrorCode,

    /// Standard linux error code.
    pub linux_error_code: DltFtInt,

    /// File serial number (usually inode).
    pub file_serial_number: DltFtUInt,

    /// Absolute path to the file.
    pub file_name: &'a str,

    /// Size of the file.
    pub file_size: DltFtUInt,

    /// File creaton date.
    pub creation_date: &'b str,

    /// Number of packages that will be used to transfer the file.
    pub number_of_packages: DltFtUInt,
}

impl<'a, 'b> DltFtErrorPkg<'a, 'b> {
    /// Verbose string at the start and end of the "DLT File Transfer Error" package.
    pub const PKG_FLAG: &'static str = "FLER";
}
