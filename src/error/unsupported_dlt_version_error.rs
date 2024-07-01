use crate::*;

/// Error that is triggered when an unsupported DLT version is
/// encountred when parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedDltVersionError {
    /// Unsupported version number that was encountered in the DLT header.
    pub unsupported_version: u8,
}

impl core::fmt::Display for UnsupportedDltVersionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Encountered unsupported DLT version '{}' in header. Only versions {:?} are supported.",
            self.unsupported_version,
            DltHeader::SUPPORTED_DECODABLE_VERSIONS
        )
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for UnsupportedDltVersionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod unsupported_dlt_version_error_test {
    use super::*;

    #[test]
    fn clone_eq() {
        let v = UnsupportedDltVersionError {
            unsupported_version: 123,
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        let v = UnsupportedDltVersionError {
            unsupported_version: 123,
        };
        assert_eq!(
            format!(
                "Encountered unsupported DLT version '{}' in header. Only versions {:?} are supported.",
                v.unsupported_version,
                crate::DltHeader::SUPPORTED_DECODABLE_VERSIONS
            ),
            format!("{}", v)
        );
    }

    #[test]
    fn display() {
        let v = UnsupportedDltVersionError {
            unsupported_version: 123,
        };
        assert_eq!(
            format!(
                "UnsupportedDltVersionError {{ unsupported_version: {} }}",
                v.unsupported_version
            ),
            format!("{:?}", v)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        assert!(UnsupportedDltVersionError {
            unsupported_version: 123,
        }
        .source()
        .is_none());
    }
}
