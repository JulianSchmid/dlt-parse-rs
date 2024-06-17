use super::*;

/// Package containing a chunk of data of a file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DltFtDataPkg<'a> {
    /// File serial number (usually inode).
    pub file_serial_number: DltFtUInt,

    /// Transfered package number.
    pub package_nr: DltFtUInt,

    /// Transfered data.
    pub data: &'a [u8],
}

impl<'a> DltFtDataPkg<'a> {
    /// Verbose string at the start and end of the "DLT File Transfer Data" package.
    pub const PKG_FLAG: &'static str = "FLDA";
}
