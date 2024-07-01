use arrayvec::{ArrayVec, CapacityError};

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

    /// Number of verbose arguments in a file transfer "end package" written
    /// in the DLT extended header.
    pub const NUM_ARGS: u16 = 3;

    
}
