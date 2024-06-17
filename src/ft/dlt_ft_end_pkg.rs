use super::*;

/// Package sent after a file transfer is complete.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DltFtEndPkg {
    /// File serial number (usually inode).
    pub file_serial_number: DltFtUInt,
}

impl DltFtEndPkg {
    /// Verbose string at the start and end of the "DLT File Transfer End" package.
    pub const PKG_FLAG: &'static str = "FLFI";
}
