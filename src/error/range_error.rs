/// Error that can occur when an out of range value is passed to a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RangeError {
    /// Error if the user defined value is outside the range of 7-15
    NetworkTypekUserDefinedOutsideOfRange(u8),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for RangeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl core::fmt::Display for RangeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use RangeError::*;

        match self {
            NetworkTypekUserDefinedOutsideOfRange(value) => {
                write!(f, "RangeError: Message type info field user defined value of {} outside of the allowed range of 7-15.", value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use proptest::prelude::*;

    #[test]
    fn clone_eq() {
        use RangeError::*;
        let v = NetworkTypekUserDefinedOutsideOfRange(123);
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        use RangeError::*;
        let v = NetworkTypekUserDefinedOutsideOfRange(123);
        assert_eq!(
            "NetworkTypekUserDefinedOutsideOfRange(123)",
            format!("{:?}", v)
        );
    }

    proptest! {
        #[test]
        fn display(value in any::<u8>()) {
            use RangeError::*;

            // NetworkTypekUserDefinedOutsideOfRange
            assert_eq!(
                &format!("RangeError: Message type info field user defined value of {} outside of the allowed range of 7-15.", value),
                &format!("{}", NetworkTypekUserDefinedOutsideOfRange(value))
            );
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn source() {
        use std::error::Error;
        use RangeError::*;

        assert!(NetworkTypekUserDefinedOutsideOfRange(123)
            .source()
            .is_none());
    }
} // mod tests
