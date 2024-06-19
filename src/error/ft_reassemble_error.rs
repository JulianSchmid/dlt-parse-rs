#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FtReassembleError {

    /// Error if the number of bytes of data in a "DLT file transfer" packet
    /// is not matching the original in the header specified buffer size.
    DataLenNotMatchingBufferSize {
        header_buffer_len: u64,
        data_pkt_len: u64,
        data_pkt_nr: u64,
        number_of_packages: u64,
    },

    /// Error if the number of packages & buffer len do not match the file_size.
    InconsitantHeaderLenValues {
        file_size: u64,
        number_of_packages: u64,
        buffer_len: u64
    },

    /// Error if a data package with an unexpected package nr is received.
    UnexpectedPackageNrInDataPkg {
        expected_nr_of_packages: u64,
        package_nr: u64,
    },

    /// Error if not enough memory could be allocated to store the file in memory.
    AllocationFailure { len: usize },
}

impl core::fmt::Display for FtReassembleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FtReassembleError::*;
        match self {
            DataLenNotMatchingBufferSize{ header_buffer_len, data_pkt_len, data_pkt_nr, number_of_packages } => write!(f, "Payload length {data_pkt_len} of DLT file transfer data packet (nr {data_pkt_nr} of {number_of_packages}) is not matching the buffer len {header_buffer_len} set by the header packet."),
            InconsitantHeaderLenValues{ file_size, number_of_packages, buffer_len } => write!(f, "DLT file transfer header packet 'file size' {file_size} is inconsistant with the 'buffer size' {buffer_len} and 'number of packages' {number_of_packages}"),
            UnexpectedPackageNrInDataPkg { expected_nr_of_packages, package_nr } => write!(f, "Received a DLT file transfer data packet with the unexpected package number {package_nr} (expected number of packages based on header package is {expected_nr_of_packages})."),
            AllocationFailure { len } => write!(f, "Failed to allocate {len} bytes of memory to reconstruct the SOMEIP TP packets."),
        }
    }
}

impl std::error::Error for FtReassembleError {}

#[cfg(test)]
mod tests {
    /*use super::FtReassembleError::*;

    
    #[test]
    fn debug() {
        let err = AllocationFailure { len: 0 };
        let _ = format!("{err:?}");
    }

    #[test]
    fn clone_eq_hash_ord() {
        use core::cmp::Ordering;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let err = AllocationFailure { len: 0 };
        assert_eq!(err, err.clone());
        let hash_a = {
            let mut hasher = DefaultHasher::new();
            err.hash(&mut hasher);
            hasher.finish()
        };
        let hash_b = {
            let mut hasher = DefaultHasher::new();
            err.clone().hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(hash_a, hash_b);
        assert_eq!(Ordering::Equal, err.cmp(&err));
        assert_eq!(Some(Ordering::Equal), err.partial_cmp(&err));
    }

    #[test]
    fn fmt() {
        let tests = [
            (UnalignedTpPayloadLen { offset: 1, payload_len: 2 }, "Payload length 2 of SOMEIP TP segment (offset 1) is not a multiple of 16. This is only allowed for TP packets where the 'more segements' flag is not set."),
            (SegmentTooBig { offset: 1, payload_len: 2, max: 3, }, "Overall length of TP segment (offset 1, payload len: 2) bigger then the maximum allowed size of 3."),
            (ConflictingEnd { previous_end: 1, conflicting_end: 2, }, "Received a TP package (offset + len: 2) which conflicts a package that previously set the end to 1."),
            (AllocationFailure { len: 0 }, "Faield to allocate 0 bytes of memory to reconstruct the SOMEIP TP packets."),
        ];
        for test in tests {
            assert_eq!(format!("{}", test.0), test.1);
        }
    }

    #[test]
    fn source() {
        use std::error::Error;
        assert!(AllocationFailure { len: 0 }.source().is_none());
    }
     */
}