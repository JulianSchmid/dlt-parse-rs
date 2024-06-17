use super::*;

/// Packet sent at the start of a file transfer.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DltFtHeaderPkg<'a, 'b> {
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
    
    /// Needed buffer size to reconsturct the file.
    pub buffer_size: DltFtUInt,
}

impl<'a, 'b> DltFtHeaderPkg<'a, 'b> {
    /// Verbose string at the start and end of the "DLT File Transfer Header" package.
    pub const PKG_FLAG: &'static str = "FLST";
}
