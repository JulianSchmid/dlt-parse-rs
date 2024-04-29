use super::*;

/// Error if a slice did not contain enough data to decode a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnexpectedEndOfSliceError {
    /// Layer in which the length error was detected.
    pub layer: Layer,

    /// Minimum expected slice length.
    pub minimum_size: usize,

    /// Actual slice length (which was too small).
    pub actual_size: usize,
}

impl core::fmt::Display for UnexpectedEndOfSliceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:?}: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.",
            self.layer,
            self.actual_size,
            self.minimum_size
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnexpectedEndOfSliceError {
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
        let v = UnexpectedEndOfSliceError {
            layer: Layer::DltHeader,
            minimum_size: 2,
            actual_size: 3,
        };
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        let v = UnexpectedEndOfSliceError {
            layer: Layer::DltHeader,
            minimum_size: 2,
            actual_size: 3,
        };
        assert_eq!(
            format!(
                "UnexpectedEndOfSliceError {{ layer: {:?}, minimum_size: {}, actual_size: {} }}",
                v.layer, v.minimum_size, v.actual_size
            ),
            format!("{:?}", v)
        );
    }

    #[test]
    fn display() {
        let v = UnexpectedEndOfSliceError {
            layer: Layer::DltHeader,
            minimum_size: 2,
            actual_size: 3,
        };
        assert_eq!(
            format!(
                "{:?}: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.",
                v.layer,
                v.actual_size,
                v.minimum_size,
            ),
            format!("{}", v)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn source() {
        use std::error::Error;
        assert!(UnexpectedEndOfSliceError {
            layer: Layer::DltHeader,
            minimum_size: 2,
            actual_size: 3,
        }
        .source()
        .is_none());
    }
}
