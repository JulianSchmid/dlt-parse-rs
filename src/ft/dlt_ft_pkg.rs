use super::*;
use crate::verbose::*;

/// DLT file transfer package.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DltFtPkg<'a, 'b> {
    /// Packet sent at the start of a file transfer.
    Header(DltFtHeaderPkg<'a, 'b>),
    /// Package containing a chunk of data of a file.
    Data(DltFtDataPkg<'a>),
    /// Package sent after a file transfer is complete.
    End(DltFtEndPkg),
    /// Info packet for a file if only metadat is sent.
    Info(DltFtInfoPkg<'a, 'b>),
    /// Error package sent when an error occured with an
    /// existing file.
    Error(DltFtErrorPkg<'a, 'b>),
    /// Error package sent if a file that should have been
    /// transfered does not exists.
    FileNotExistsError(DltFtFileNotExistErrorPkg<'a>),
}

impl<'a, 'b> DltFtPkg<'a, 'b> {
    /// Checks if the verbose iterator contains a DLT file transfer package
    /// and returns the package if so.
    pub fn try_from(mut iter: VerboseIter<'a>) -> Option<DltFtPkg<'a, 'a>> {
        match iter.number_of_arguments() {
            3 => {
                Self::check_for_str(DltFtEndPkg::PKG_FLAG, &mut iter)?;
                let file_serial_number = DltFtUInt::try_take_from_iter(&mut iter)?;
                Self::check_for_str(DltFtEndPkg::PKG_FLAG, &mut iter)?;

                Some(DltFtPkg::End(DltFtEndPkg { file_serial_number }))
            }
            5 => {
                let first = Self::try_take_str_from_iter(&mut iter)?;
                match first {
                    DltFtDataPkg::PKG_FLAG => {
                        let file_serial_number = DltFtUInt::try_take_from_iter(&mut iter)?;
                        let package_nr = DltFtUInt::try_take_from_iter(&mut iter)?;
                        let data = Self::try_take_raw_from_iter(&mut iter)?;
                        Self::check_for_str(DltFtDataPkg::PKG_FLAG, &mut iter)?;

                        Some(DltFtPkg::Data(DltFtDataPkg {
                            file_serial_number,
                            package_nr,
                            data,
                        }))
                    }
                    DltFtFileNotExistErrorPkg::PKG_FLAG => {
                        let error_code = DltFtInt::try_take_from_iter(&mut iter)?;
                        let linux_error_code = DltFtInt::try_take_from_iter(&mut iter)?;
                        let file_name = Self::try_take_str_from_iter(&mut iter)?;
                        Self::check_for_str(DltFtFileNotExistErrorPkg::PKG_FLAG, &mut iter)?;

                        Some(DltFtPkg::FileNotExistsError(DltFtFileNotExistErrorPkg {
                            error_code: DltFtErrorCode(error_code),
                            linux_error_code,
                            file_name,
                        }))
                    }
                    _ => None,
                }
            }
            7 => {
                Self::check_for_str(DltFtInfoPkg::PKG_FLAG, &mut iter)?;
                let file_serial_number = DltFtUInt::try_take_from_iter(&mut iter)?;
                let file_name = Self::try_take_str_from_iter(&mut iter)?;
                let file_size = DltFtUInt::try_take_from_iter(&mut iter)?;
                let creation_date = Self::try_take_str_from_iter(&mut iter)?;
                let number_of_packages = DltFtUInt::try_take_from_iter(&mut iter)?;
                Self::check_for_str(DltFtInfoPkg::PKG_FLAG, &mut iter)?;

                Some(DltFtPkg::Info(DltFtInfoPkg {
                    file_serial_number,
                    file_name,
                    file_size,
                    creation_date,
                    number_of_packages,
                }))
            }
            8 => {
                Self::check_for_str(DltFtHeaderPkg::PKG_FLAG, &mut iter)?;
                let file_serial_number = DltFtUInt::try_take_from_iter(&mut iter)?;
                let file_name = Self::try_take_str_from_iter(&mut iter)?;
                let file_size = DltFtUInt::try_take_from_iter(&mut iter)?;
                let creation_date = Self::try_take_str_from_iter(&mut iter)?;
                let number_of_packages = DltFtUInt::try_take_from_iter(&mut iter)?;
                let buffer_size = DltFtUInt::try_take_from_iter(&mut iter)?;
                Self::check_for_str(DltFtHeaderPkg::PKG_FLAG, &mut iter)?;

                Some(DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number,
                    file_name,
                    file_size,
                    creation_date,
                    number_of_packages,
                    buffer_size,
                }))
            }
            9 => {
                Self::check_for_str(DltFtErrorPkg::PKG_FLAG, &mut iter)?;
                let error_code = DltFtInt::try_take_from_iter(&mut iter)?;
                let file_serial_number = DltFtUInt::try_take_from_iter(&mut iter)?;
                let linux_error_code = DltFtInt::try_take_from_iter(&mut iter)?;
                let file_name = Self::try_take_str_from_iter(&mut iter)?;
                let file_size = DltFtUInt::try_take_from_iter(&mut iter)?;
                let creation_date = Self::try_take_str_from_iter(&mut iter)?;
                let number_of_packages = DltFtUInt::try_take_from_iter(&mut iter)?;
                Self::check_for_str(DltFtErrorPkg::PKG_FLAG, &mut iter)?;

                Some(DltFtPkg::Error(DltFtErrorPkg {
                    error_code: DltFtErrorCode(error_code),
                    linux_error_code,
                    file_serial_number,
                    file_name,
                    file_size,
                    creation_date,
                    number_of_packages,
                }))
            }
            _ => None,
        }
    }

    fn try_take_str_from_iter<'c>(iter: &mut VerboseIter<'c>) -> Option<&'c str> {
        let Some(Ok(VerboseValue::Str(result))) = iter.next() else {
            return None;
        };
        if result.name.is_some() {
            return None;
        }
        Some(result.value)
    }

    fn try_take_raw_from_iter<'c>(iter: &mut VerboseIter<'c>) -> Option<&'c [u8]> {
        let Some(Ok(VerboseValue::Raw(result))) = iter.next() else {
            return None;
        };
        if result.name.is_some() {
            return None;
        }
        Some(result.data)
    }

    fn check_for_str(expected: &str, iter: &mut VerboseIter<'_>) -> Option<()> {
        let Some(Ok(VerboseValue::Str(result))) = iter.next() else {
            return None;
        };
        if result.name.is_some() {
            return None;
        }
        if result.value != expected {
            return None;
        }
        Some(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use arrayvec::ArrayVec;

    #[test]
    fn try_from() {
        // TODO
    }

    #[test]
    fn try_take_str_from_iter() {
        // ok case
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            StringValue{
                name: None,
                value: "a",
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::try_take_str_from_iter(&mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                Some("a")
            );
        }

        // name error case
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            StringValue{
                name: Some("name"),
                value: "a",
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::try_take_str_from_iter(&mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                None
            );
        }

        // non string value
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            U64Value{
                variable_info: None,
                scaling: None,
                value: 1,
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::try_take_str_from_iter(&mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                None
            );
        }
    }

    #[test]
    fn try_take_raw_from_iter() {
        // ok case
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            RawValue{
                name: None,
                data: &[1,2,3],
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::try_take_raw_from_iter(&mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                Some(&[1u8,2,3][..])
            );
        }

        // name error case
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            RawValue{
                name: Some("name"),
                data: &[1,2,3],
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::try_take_raw_from_iter(&mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                None
            );
        }

        // non raw value
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            U64Value{
                variable_info: None,
                scaling: None,
                value: 1,
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::try_take_raw_from_iter(&mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                None
            );
        }
    }

    #[test]
    fn check_for_str() {
        // ok case
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            StringValue{
                name: None,
                value: "a",
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::check_for_str("a", &mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                Some(())
            );
        }

        // non matching string
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            StringValue{
                name: None,
                value: "a",
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::check_for_str("b", &mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                None
            );
        }

        // name error case
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            StringValue{
                name: Some("name"),
                value: "a",
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::check_for_str("a", &mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                None
            );
        }

        // non string value
        {
            let mut bytes = ArrayVec::<u8, 1000>::new();
            U64Value{
                variable_info: None,
                scaling: None,
                value: 1,
            }.add_to_msg(&mut bytes, false).unwrap();
            assert_eq!(
                DltFtPkg::check_for_str("a", &mut VerboseIter::new(
                    false, 1, &bytes[..]
                )),
                None
            );
        }
    }
}
