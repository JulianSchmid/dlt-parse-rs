use super::VerboseValue;
use crate::error::VerboseDecodeError;

/// Iterator over verbose values.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VerboseIter<'a> {
    is_big_endian: bool,
    number_of_arguments: u8,
    rest: &'a [u8],
}

impl<'a> VerboseIter<'a> {
    /// Creates new iterator to iterate over the verbose values of a dlt messages.
    #[inline]
    pub fn new(is_big_endian: bool, number_of_arguments: u8, payload: &'a [u8]) -> VerboseIter<'a> {
        VerboseIter {
            is_big_endian,
            number_of_arguments,
            rest: payload,
        }
    }

    /// Returns if the values encoded in the big endian format.
    #[inline]
    pub fn is_big_endian(&self) -> bool {
        self.is_big_endian
    }

    /// Number of arguments left in the iterator.
    #[inline]
    pub fn number_of_arguments(&self) -> u8 {
        self.number_of_arguments
    }

    /// Raw data.
    #[inline]
    pub fn raw(&self) -> &'a [u8] {
        self.rest
    }
}

impl<'a> core::iter::Iterator for VerboseIter<'a> {
    type Item = Result<VerboseValue<'a>, VerboseDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.number_of_arguments == 0 {
            None
        } else {
            match VerboseValue::from_slice(self.rest, self.is_big_endian) {
                Ok((value, rest)) => {
                    self.rest = rest;
                    self.number_of_arguments -= 1;
                    Some(Ok(value))
                }
                Err(err) => {
                    // move to end in case of error so we end the iteration
                    self.rest = &self.rest[self.rest.len()..];
                    self.number_of_arguments = 0;
                    Some(Err(err))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::VerboseIter;
    use crate::verbose::{U16Value, U32Value, VerboseValue};
    use arrayvec::ArrayVec;

    #[test]
    fn new() {
        let data = [1, 2, 3, 4];
        let actual = VerboseIter::new(true, 123, &data);
        assert!(actual.is_big_endian);
        assert_eq!(actual.number_of_arguments, 123);
        assert_eq!(actual.rest, &data);
    }

    #[test]
    fn next() {
        // empty
        {
            let data = [1, 2, 3, 4];
            let mut iter = VerboseIter::new(false, 0, &data);
            assert_eq!(None, iter.next());
            assert_eq!(None, iter.next());
        }
        // single value ok (big endian)
        {
            let mut data = ArrayVec::<u8, 1000>::new();
            let value = U16Value {
                variable_info: None,
                scaling: None,
                value: 1234,
            };
            value.add_to_msg(&mut data, true).unwrap();

            let mut iter = VerboseIter::new(true, 1, &data);
            assert_eq!(Some(Ok(VerboseValue::U16(value))), iter.next());
            assert_eq!(None, iter.next());
            assert_eq!(None, iter.next());
        }
        // two values ok (little endian)
        {
            let mut data = ArrayVec::<u8, 1000>::new();
            let first_value = U16Value {
                variable_info: None,
                scaling: None,
                value: 1234,
            };
            first_value.add_to_msg(&mut data, false).unwrap();
            let second_value = U32Value {
                variable_info: None,
                scaling: None,
                value: 2345,
            };
            second_value.add_to_msg(&mut data, false).unwrap();

            let mut iter = VerboseIter::new(false, 2, &data);
            assert_eq!(Some(Ok(VerboseValue::U16(first_value))), iter.next());
            assert_eq!(Some(Ok(VerboseValue::U32(second_value))), iter.next());
            assert_eq!(None, iter.next());
            assert_eq!(None, iter.next());
        }
        // more values present then number of arguments
        {
            let mut data = ArrayVec::<u8, 1000>::new();
            let first_value = U16Value {
                variable_info: None,
                scaling: None,
                value: 1234,
            };
            first_value.add_to_msg(&mut data, false).unwrap();
            let second_value = U32Value {
                variable_info: None,
                scaling: None,
                value: 2345,
            };
            second_value.add_to_msg(&mut data, false).unwrap();

            let mut iter = VerboseIter::new(false, 1, &data);
            assert_eq!(Some(Ok(VerboseValue::U16(first_value))), iter.next());
            assert_eq!(None, iter.next());
            assert_eq!(None, iter.next());
        }
        // number of arguments bigger then present data
        {
            let mut data = ArrayVec::<u8, 1000>::new();
            let first_value = U16Value {
                variable_info: None,
                scaling: None,
                value: 1234,
            };
            first_value.add_to_msg(&mut data, false).unwrap();
            let second_value = U32Value {
                variable_info: None,
                scaling: None,
                value: 2345,
            };
            second_value.add_to_msg(&mut data, false).unwrap();

            let mut iter = VerboseIter::new(false, 3, &data);
            assert_eq!(Some(Ok(VerboseValue::U16(first_value))), iter.next());
            assert_eq!(Some(Ok(VerboseValue::U32(second_value))), iter.next());
            assert!(iter.next().unwrap().is_err());
            assert_eq!(None, iter.next());
        }
    }
}
