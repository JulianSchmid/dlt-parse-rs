use super::{VerboseIter, VerboseValue};
use crate::error::VerboseDecodeError;

/// Iterator over verbose values (payload was verified at start and contains no errors).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PrecheckedVerboseIter<'a> {
    iter: VerboseIter<'a>,
}

impl<'a> PrecheckedVerboseIter<'a> {
    /// Creates an iterator to iterate over verbose values and check
    /// there are no errors present.
    pub fn try_new(
        is_big_endian: bool,
        number_of_arguments: u8,
        payload: &'a [u8],
    ) -> Result<PrecheckedVerboseIter<'a>, VerboseDecodeError> {
        let iter = VerboseIter::new(is_big_endian, number_of_arguments, payload);
        // do a test run through the data to ensure all is good
        for v in iter.clone() {
            v?;
        }
        Ok(PrecheckedVerboseIter { iter })
    }
}

impl<'a> TryFrom<VerboseIter<'a>> for PrecheckedVerboseIter<'a> {
    type Error = VerboseDecodeError;

    fn try_from(value: VerboseIter<'a>) -> Result<Self, Self::Error> {
        for v in value.clone() {
            v?;
        }
        Ok(PrecheckedVerboseIter { iter: value })
    }
}

impl<'a> core::iter::Iterator for PrecheckedVerboseIter<'a> {
    type Item = VerboseValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v.unwrap())
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::ser::Serialize for PrecheckedVerboseIter<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.iter.number_of_arguments().into()))?;
        for element in self.clone() {
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod test {
    use super::VerboseIter;
    use crate::verbose::{PrecheckedVerboseIter, U16Value, U32Value, VerboseValue};
    use arrayvec::ArrayVec;

    #[test]
    fn new_and_next() {
        // zero args
        {
            let data = [1, 2, 3, 4];
            let actual = PrecheckedVerboseIter::try_new(true, 0, &data).unwrap();
            assert_eq!(actual.iter, VerboseIter::new(true, 0, &data));
        }
        // single value
        {
            let mut data = ArrayVec::<u8, 1000>::new();
            let value = U16Value {
                variable_info: None,
                scaling: None,
                value: 1234,
            };
            value.add_to_msg(&mut data, true).unwrap();

            let mut iter = PrecheckedVerboseIter::try_new(true, 1, &data).unwrap();
            assert_eq!(Some(VerboseValue::U16(value)), iter.next());
            assert_eq!(None, iter.next());
            assert_eq!(None, iter.next());
        }
        // error: number of arguments bigger then present data
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

            assert!(PrecheckedVerboseIter::try_new(false, 3, &data).is_err());
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        use VerboseValue::{U16, U32};

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

        let iter = PrecheckedVerboseIter::try_new(false, 2, &data).unwrap();

        assert_eq!(
            serde_json::to_string(&iter).unwrap(),
            serde_json::to_string(&[U16(first_value), U32(second_value)]).unwrap()
        );
    }
}
