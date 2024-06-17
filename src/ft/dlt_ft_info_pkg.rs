use super::*;

/// Info packet for a file if only metadat is sent.
///
/// This packet is sent if only informations about a file
/// are sent without the file contents.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DltFtInfoPkg<'a, 'b> {
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

impl<'a, 'b> DltFtInfoPkg<'a, 'b> {
    /// Verbose string at the start and end of the "DLT File Transfer Info" package.
    pub const PKG_FLAG: &'static str = "FLIF";
}
