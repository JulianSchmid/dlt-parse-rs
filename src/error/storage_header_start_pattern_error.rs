/// Error that occurs when another pattern then
/// [`crate::storage::StorageHeader::PATTERN_AT_START`] is encountered
/// at the start when parsing a StorageHeader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageHeaderStartPatternError {
    /// Encountered pattern at the start.
    pub actual_pattern: [u8; 4],
}

impl core::fmt::Display for StorageHeaderStartPatternError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Error when parsing DLT storage header. Expected pattern {:?} at start but got {:?}",
            crate::storage::StorageHeader::PATTERN_AT_START,
            self.actual_pattern
        )
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for StorageHeaderStartPatternError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn clone_eq() {
        let v = StorageHeaderStartPatternError {
            actual_pattern: [1, 2, 3, 4],
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        let v = StorageHeaderStartPatternError {
            actual_pattern: [1, 2, 3, 4],
        };
        assert_eq!(
            format!(
                "StorageHeaderStartPatternError {{ actual_pattern: {:?} }}",
                v.actual_pattern
            ),
            format!("{:?}", v)
        );
    }

    #[test]
    fn display() {
        let v = StorageHeaderStartPatternError {
            actual_pattern: [1, 2, 3, 4],
        };
        assert_eq!(
            format!(
                "Error when parsing DLT storage header. Expected pattern {:?} at start but got {:?}",
                crate::storage::StorageHeader::PATTERN_AT_START,
                v.actual_pattern
            ),
            format!("{}", v)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        assert!(StorageHeaderStartPatternError {
            actual_pattern: [1, 2, 3, 4]
        }
        .source()
        .is_none());
    }
}
