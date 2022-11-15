use crate::DltPacketSlice;
use super::StorageHeader;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageSlice<'a> {
    pub storage_header: StorageHeader,
    pub packet: DltPacketSlice<'a>,
}
