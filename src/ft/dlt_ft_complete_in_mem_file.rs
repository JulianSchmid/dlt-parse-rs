use super::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DltFtCompleteInMemFile<'a> {
    /// File serial number (usually inode).
    pub file_serial_number: DltFtUInt,

    /// Absolute path to the file.
    pub file_name: &'a str,

    /// File creaton date.
    pub creation_date: &'a str,

    /// Slice containing the complete file data.
    pub data: &'a [u8],
}
