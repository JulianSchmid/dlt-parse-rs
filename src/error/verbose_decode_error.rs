use super::*;
use core::str::Utf8Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerboseDecodeError {
    /// Error that occurs if a type info is inconsistent.
    ///
    /// An example is when the type is described as both a
    /// bool and float or otherwise contains flags that contradict
    /// each other.
    ///
    /// The encoded type info is given as an argument.
    InvalidTypeInfo([u8; 4]),

    /// Error in case an invalid bool value is encountered (not 0 or 1).
    InvalidBoolValue(u8),

    /// Error if not enough data was present in the slice to decode
    /// a verbose value.
    UnexpectedEndOfSlice(UnexpectedEndOfSliceError),

    /// Error if a variable name string is not zero terminated.
    VariableNameStringMissingNullTermination,

    /// Error if a variable unit string is not zero terminated.
    VariableUnitStringMissingNullTermination,

    /// Error if the total len calculated from the array dimensions overflows.
    ArrayDimensionsOverflow,

    StructDataLengthOverflow,

    /// Error when decoding an string (can also occur for variable names or unit names).
    Utf8(Utf8Error),
}

impl core::fmt::Display for VerboseDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use VerboseDecodeError::*;
        match self {
            InvalidTypeInfo(value) => write!(
                f, "DLT Verbose Message Field: Encountered an invalid typeinfo {:?} (contradicting or unknown)", value
            ),
            InvalidBoolValue(value) => write!(
                f, "DLT Verbose Message Field: Encountered invalid bool value '{}' (only 0 or 1 are valid)", value
            ),
            UnexpectedEndOfSlice(err) => err.fmt(f),
            VariableNameStringMissingNullTermination => write!(
                f, "DLT Verbose Message Field: Encountered a variable name string missing the terminating zero value"
            ),
            VariableUnitStringMissingNullTermination => write!(
                f, "DLT Verbose Message Field: Encountered a variable unit string missing the terminating zero value"
            ),
            Utf8(err) => err.fmt(f),
            ArrayDimensionsOverflow => write!(f, "DLT Verbose Message Field: Array dimension sizes too big. Calculating the overall array size would cause an integer overflow."),
            StructDataLengthOverflow => write!(f, "DLT Verbose Message Field: Struct data length too big. Would cause an integer overflow."),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VerboseDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use VerboseDecodeError::*;
        match self {
            InvalidTypeInfo(_) => None,
            InvalidBoolValue(_) => None,
            UnexpectedEndOfSlice(err) => Some(err),
            VariableNameStringMissingNullTermination => None,
            VariableUnitStringMissingNullTermination => None,
            Utf8(err) => Some(err),
            ArrayDimensionsOverflow => None,
            StructDataLengthOverflow => None,
        }
    }
}

impl From<Utf8Error> for VerboseDecodeError {
    fn from(err: Utf8Error) -> VerboseDecodeError {
        VerboseDecodeError::Utf8(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn clone_eq() {
        use VerboseDecodeError::*;
        let v = InvalidBoolValue(2);
        assert_eq!(v, v.clone());
    }

    #[test]
    fn debug() {
        use VerboseDecodeError::*;
        let v = InvalidBoolValue(2);
        assert_eq!(format!("InvalidBoolValue({})", 2), format!("{:?}", v));
    }

    #[test]
    fn display() {
        use VerboseDecodeError::*;

        assert_eq!(
            format!("DLT Verbose Message Field: Encountered an invalid typeinfo {:?} (contradicting or unknown)", [1,2,3,4]),
            format!("{}", InvalidTypeInfo([1,2,3,4]))
        );

        assert_eq!(
            format!("DLT Verbose Message Field: Encountered invalid bool value '{}' (only 0 or 1 are valid)", 2),
            format!("{}", InvalidBoolValue(2))
        );

        {
            let v = UnexpectedEndOfSliceError {
                layer: Layer::DltHeader,
                actual_size: 1,
                minimum_size: 2,
            };
            assert_eq!(format!("{}", v), format!("{}", UnexpectedEndOfSlice(v)));
        }

        assert_eq!(
            format!("DLT Verbose Message Field: Encountered a variable name string missing the terminating zero value"),
            format!("{}", VariableNameStringMissingNullTermination)
        );

        assert_eq!(
            format!("DLT Verbose Message Field: Encountered a variable unit string missing the terminating zero value"),
            format!("{}", VariableUnitStringMissingNullTermination)
        );

        #[allow(invalid_from_utf8)]
        {
            let v = std::str::from_utf8(&[0, 159, 146, 150]).unwrap_err();
            assert_eq!(format!("{}", v), format!("{}", Utf8(v)));
        }
    }

    #[cfg(feature = "std")]
    #[test]
    #[allow(invalid_from_utf8)]
    fn source() {
        use std::error::Error;
        use VerboseDecodeError::*;
        assert!(InvalidTypeInfo([1, 2, 3, 4]).source().is_none());
        assert!(InvalidBoolValue(2).source().is_none());
        assert!(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
            layer: Layer::DltHeader,
            actual_size: 1,
            minimum_size: 2,
        })
        .source()
        .is_some());
        assert!(VariableNameStringMissingNullTermination.source().is_none());
        assert!(VariableUnitStringMissingNullTermination.source().is_none());
        assert!(Utf8(std::str::from_utf8(&[0, 159, 146, 150]).unwrap_err())
            .source()
            .is_some());
    }

    #[test]
    #[allow(invalid_from_utf8)]
    fn from_utf8_error() {
        let e: VerboseDecodeError = std::str::from_utf8(&[0, 159, 146, 150]).unwrap_err().into();
        assert_matches!(e, VerboseDecodeError::Utf8(_));
    }
}
