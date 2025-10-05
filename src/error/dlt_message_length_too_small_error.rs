/// Error if the length field in a DLT headeris smaller then the header the calculated
/// header size based on the flags (+ minimum payload size of 4 bytes/octetets)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DltMessageLengthTooSmallError {
    pub required_length: usize,
    pub actual_length: usize,
}

impl core::fmt::Display for DltMessageLengthTooSmallError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "DLT Header Error: The message length of {} present in the dlt header is smaller then minimum required size of {} bytes.",
            self.actual_length,
            self.required_length
        )
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for DltMessageLengthTooSmallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod dlt_message_length_too_small_error_test {
    use super::*;
    use alloc::format;

    #[test]
    fn clone_eq() {
        let v = DltMessageLengthTooSmallError {
            required_length: 1,
            actual_length: 2,
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        let v = DltMessageLengthTooSmallError {
            required_length: 1,
            actual_length: 2,
        };
        assert_eq!(
            format!(
                "DltMessageLengthTooSmallError {{ required_length: {}, actual_length: {} }}",
                v.required_length, v.actual_length,
            ),
            format!("{:?}", v)
        );
    }

    #[test]
    fn display() {
        let v = DltMessageLengthTooSmallError {
            required_length: 1,
            actual_length: 2,
        };
        assert_eq!(
            format!(
                "DLT Header Error: The message length of {} present in the dlt header is smaller then minimum required size of {} bytes.",
                v.actual_length,
                v.required_length,
            ),
            format!("{}", v)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        assert!(DltMessageLengthTooSmallError {
            required_length: 1,
            actual_length: 2,
        }
        .source()
        .is_none());
    }
}
