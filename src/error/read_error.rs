#[cfg(feature = "std")]
use super::*;

///Errors that can occure on reading a dlt header.
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[derive(Debug)]
pub enum ReadError {
    /// Error if the slice is smaller then dlt length field or minimal size.
    UnexpectedEndOfSlice(UnexpectedEndOfSliceError),

    /// An unsupporetd version number has been encountered
    /// while decoding the header.
    UnsupportedDltVersion(UnsupportedDltVersionError),

    /// Error if the dlt length is smaller then the header the calculated header size based on the flags (+ minimum payload size of 4 bytes/octetets)
    DltMessageLengthTooSmall(DltMessageLengthTooSmallError),

    /// Error if a storage header does not start with the correct pattern.
    StorageHeaderStartPattern(StorageHeaderStartPatternError),

    /// Standard io error.
    IoError(std::io::Error),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ReadError::*;
        match self {
            UnexpectedEndOfSlice(ref err) => Some(err),
            UnsupportedDltVersion(ref err) => Some(err),
            DltMessageLengthTooSmall(ref err) => Some(err),
            StorageHeaderStartPattern(ref err) => Some(err),
            IoError(ref err) => Some(err),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl core::fmt::Display for ReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use ReadError::*;

        match self {
            UnexpectedEndOfSlice(err) => {
                write!(f, "ReadError: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.", err.actual_size, err.minimum_size)
            }
            UnsupportedDltVersion(err) => err.fmt(f),
            DltMessageLengthTooSmall(err) => err.fmt(f),
            StorageHeaderStartPattern(err) => err.fmt(f),
            IoError(err) => err.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<StorageHeaderStartPatternError> for ReadError {
    fn from(err: StorageHeaderStartPatternError) -> ReadError {
        ReadError::StorageHeaderStartPattern(err)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<PacketSliceError> for ReadError {
    fn from(err: PacketSliceError) -> ReadError {
        use PacketSliceError as I;
        match err {
            I::UnsupportedDltVersion(err) => ReadError::UnsupportedDltVersion(err),
            I::MessageLengthTooSmall(err) => ReadError::DltMessageLengthTooSmall(err),
            I::UnexpectedEndOfSlice(err) => ReadError::UnexpectedEndOfSlice(err),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> ReadError {
        ReadError::IoError(err)
    }
}

/// Tests for `ReadError` methods
#[cfg(all(feature = "std", test))]
mod tests {
    use super::*;
    use alloc::format;
    use proptest::prelude::*;

    #[test]
    fn debug() {
        use ReadError::*;

        {
            let c = UnexpectedEndOfSliceError {
                minimum_size: 1,
                actual_size: 2,
                layer: Layer::DltHeader,
            };
            assert_eq!(
                format!("UnexpectedEndOfSlice({:?})", c),
                format!("{:?}", UnexpectedEndOfSlice(c))
            );
        }
        {
            let c = UnsupportedDltVersionError {
                unsupported_version: 123,
            };
            assert_eq!(
                format!("UnsupportedDltVersion({:?})", c),
                format!("{:?}", UnsupportedDltVersion(c))
            );
        }
        {
            let c = DltMessageLengthTooSmallError {
                required_length: 3,
                actual_length: 4,
            };
            assert_eq!(
                format!("DltMessageLengthTooSmall({:?})", c),
                format!("{:?}", DltMessageLengthTooSmall(c))
            );
        }
        {
            let c = std::io::Error::new(std::io::ErrorKind::Other, "oh no!");
            assert_eq!(format!("IoError({:?})", c), format!("{:?}", IoError(c)));
        }
    }

    proptest! {
        #[test]
        fn display(
            usize0 in any::<usize>(),
            usize1 in any::<usize>(),
        ) {

            use ReadError::*;

            // UnexpectedEndOfSlice
            assert_eq!(
                &format!("ReadError: Unexpected end of slice. The given slice only contained {} bytes, which is less then minimum required {} bytes.", usize1, usize0),
                &format!(
                    "{}",
                    UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError {
                            layer: Layer::DltHeader,
                            minimum_size: usize0,
                            actual_size: usize1,
                        }
                    )
                )
            );

            // UnsupportedDltVersionError
            {
                let c = UnsupportedDltVersionError{ unsupported_version: 123 };
                assert_eq!(
                    &format!("{}", c),
                    &format!("{}", UnsupportedDltVersion(c))
                );
            }

            // DltMessageLengthTooSmall
            {
                let c = DltMessageLengthTooSmallError{
                    required_length: usize0,
                    actual_length: usize1
                };
                assert_eq!(
                    &format!("{}", c),
                    &format!("{}", DltMessageLengthTooSmall(c))
                );
            }

            // StorageHeaderStartPattern
            {
                let c = StorageHeaderStartPatternError{
                    actual_pattern: [1,2,3,4]
                };
                assert_eq!(
                    &format!("{}", c),
                    &format!("{}", StorageHeaderStartPattern(c))
                );
            }

            //IoError
            {
                let custom_error = std::io::Error::new(std::io::ErrorKind::Other, "some error");
                assert_eq!(
                    &format!("{}", custom_error),
                    &format!("{}", IoError(custom_error))
                );
            }
        }
    }

    #[test]
    fn source() {
        use std::error::Error;
        use ReadError::*;

        assert!(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
            layer: Layer::DltHeader,
            minimum_size: 1,
            actual_size: 2
        })
        .source()
        .is_some());
        assert!(UnsupportedDltVersion(UnsupportedDltVersionError {
            unsupported_version: 123
        })
        .source()
        .is_some());
        assert!(DltMessageLengthTooSmall(DltMessageLengthTooSmallError {
            required_length: 3,
            actual_length: 4
        })
        .source()
        .is_some());
        assert!(StorageHeaderStartPattern(StorageHeaderStartPatternError {
            actual_pattern: [1, 2, 3, 4]
        })
        .source()
        .is_some());
        assert!(
            IoError(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
                .source()
                .is_some()
        );
    }

    #[test]
    fn from_io_error() {
        let r: ReadError = std::io::Error::new(std::io::ErrorKind::Other, "oh no!").into();
        assert_matches!(r, ReadError::IoError(_));
    }

    #[test]
    fn from_storage_header_error() {
        let r: ReadError = StorageHeaderStartPatternError {
            actual_pattern: [1, 2, 3, 4],
        }
        .into();
        assert_matches!(r, ReadError::StorageHeaderStartPattern(_));
    }

    #[test]
    fn from_packet_slice_error() {
        use PacketSliceError as I;

        // UnsupportedDltVersion
        {
            let r: ReadError = I::UnsupportedDltVersion(UnsupportedDltVersionError {
                unsupported_version: 123,
            })
            .into();
            assert_matches!(r, ReadError::UnsupportedDltVersion(_));
        }

        // MessageLengthTooSmall
        {
            let r: ReadError = I::MessageLengthTooSmall(DltMessageLengthTooSmallError {
                required_length: 3,
                actual_length: 4,
            })
            .into();
            assert_matches!(r, ReadError::DltMessageLengthTooSmall(_));
        }

        // UnexpectedEndOfSlice
        {
            let r: ReadError = I::UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::DltHeader,
                minimum_size: 1,
                actual_size: 2,
            })
            .into();
            assert_matches!(r, ReadError::UnexpectedEndOfSlice(_));
        }
    }
} // mod tests
