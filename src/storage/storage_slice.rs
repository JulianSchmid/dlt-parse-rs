use super::StorageHeader;
use crate::DltPacketSlice;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageSlice<'a> {
    pub storage_header: StorageHeader,
    pub packet: DltPacketSlice<'a>,
}

/// A [`StorageSlice`] together with its byte position in the source.
///
/// Returned by [`crate::storage::DltStorageReader::next_packet_seek`] to track
/// the byte offset of each DLT message's storage header within the source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageSliceWithPosition<'a> {
    /// Byte offset of the start of the storage header in the source.
    pub position: u64,
    /// The storage slice at this position.
    pub slice: StorageSlice<'a>,
}
