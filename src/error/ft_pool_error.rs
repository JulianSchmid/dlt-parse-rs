use super::*;
use crate::ft::DltFtUInt;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FtPoolError {
    /// Error re-assembling the stream.
    FtReassemble(FtReassembleError),

    /// Error if a data packet for an unknown stream is received.
    DataForUnknownStream{
        file_serial_number: DltFtUInt
    },

    /// Error if a end packet for an unknown stream is received.
    EndForUnknownStream{
        file_serial_number: DltFtUInt
    },
}

impl core::fmt::Display for FtPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FtPoolError::*;
        match self {
            FtReassemble(err) => err.fmt(f),
            DataForUnknownStream{ file_serial_number } => write!(f, "Received a DLT file transfer 'data' packet from an unknown stream with file_serial_number {file_serial_number:?}."),
            EndForUnknownStream{ file_serial_number } => write!(f, "Received a DLT file transfer 'end' packet from an unknown stream with file_serial_number {file_serial_number:?}."),
        }
    }
}

impl std::error::Error for FtPoolError {}

impl From<FtReassembleError> for FtPoolError {
    fn from(value: FtReassembleError) -> Self {
        FtPoolError::FtReassemble(value)
    }
}

#[cfg(test)]
mod tests {
    /*use super::FtPoolError::*;

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
