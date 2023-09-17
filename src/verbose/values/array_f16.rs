use crate::verbose::{ArrayDimensions, VariableInfoUnit};

#[cfg(feature = "serde")]
use super::ArrayItDimension;
use super::RawF16;
use arrayvec::{ArrayVec, CapacityError};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayF16<'a> {
    pub is_big_endian: bool,
    pub dimensions: ArrayDimensions<'a>,
    pub variable_info: Option<VariableInfoUnit<'a>>,
    pub(crate) data: &'a [u8],
}

impl<'a> ArrayF16<'a> {
    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn iter(&'a self) -> ArrayF16Iterator<'a> {
        ArrayF16Iterator {
            is_big_endian: self.is_big_endian,
            rest: self.data,
        }
    }

    /// Adds the verbose value to the given dlt mesage buffer.
    pub fn add_to_msg<const CAP: usize>(
        &self,
        buf: &mut ArrayVec<u8, CAP>,
        is_big_endian: bool,
    ) -> Result<(), CapacityError> {
        if let Some(var_info) = &self.variable_info {
            let (name_len, unit_len, number_of_dimensions) = if is_big_endian {
                (
                    (var_info.name.len() as u16 + 1).to_be_bytes(),
                    (var_info.unit.len() as u16 + 1).to_be_bytes(),
                    (self.dimensions.dimensions.len() as u16 / 2).to_be_bytes(),
                )
            } else {
                (
                    (var_info.name.len() as u16 + 1).to_le_bytes(),
                    (var_info.unit.len() as u16 + 1).to_le_bytes(),
                    (self.dimensions.dimensions.len() as u16 / 2).to_le_bytes(),
                )
            };

            let type_info: [u8; 4] = [0b1000_0010, 0b0000_1001, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;

            buf.try_extend_from_slice(&number_of_dimensions)?;
            buf.try_extend_from_slice(self.dimensions.dimensions)?;
            buf.try_extend_from_slice(&[name_len[0], name_len[1], unit_len[0], unit_len[1]])?;
            buf.try_extend_from_slice(var_info.name.as_bytes())?;
            if buf.remaining_capacity() > var_info.unit.len() + 2 {
                // Safe as capacity is checked earlier
                unsafe { buf.push_unchecked(0) };
                let _ = buf.try_extend_from_slice(var_info.unit.as_bytes());
                unsafe { buf.push_unchecked(0) };
            } else {
                return Err(CapacityError::new(()));
            }
            buf.try_extend_from_slice(self.data)?;
        } else {
            let number_of_dimensions = if is_big_endian {
                (self.dimensions.dimensions.len() as u16 / 2).to_be_bytes()
            } else {
                (self.dimensions.dimensions.len() as u16 / 2).to_le_bytes()
            };
            let type_info: [u8; 4] = [0b1000_0010, 0b0000_0001, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&number_of_dimensions)?;
            buf.try_extend_from_slice(self.dimensions.dimensions)?;
            buf.try_extend_from_slice(self.data)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ArrayF16Iterator<'a> {
    pub(crate) is_big_endian: bool,
    pub(crate) rest: &'a [u8],
}

#[cfg(feature = "serde")]
impl<'a> Serialize for ArrayF16<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArrayF16", 3)?;
        state.serialize_field("variable_info", &self.variable_info)?;

        let iter = ArrayItDimension::<RawF16> {
            is_big_endian: self.is_big_endian,
            dimensions: self.dimensions.dimensions,
            data: self.data,
            phantom: Default::default(),
        };
        state.serialize_field("data", &iter)?;

        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'a> Serialize for ArrayF16Iterator<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.rest.len() / 2))?;
        for e in self.clone() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}

impl Iterator for ArrayF16Iterator<'_> {
    type Item = RawF16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.len() < 2 {
            None
        } else {
            let bytes = unsafe {
                // SAFETY: Safe as len checked to be at least 2.
                [*self.rest.get_unchecked(0), *self.rest.get_unchecked(1)]
            };
            let result = if self.is_big_endian {
                RawF16::from_be_bytes(bytes)
            } else {
                RawF16::from_le_bytes(bytes)
            };
            self.rest = unsafe {
                // SAFETY: Safe as len checked to be at least 2.
                core::slice::from_raw_parts(self.rest.as_ptr().add(2), self.rest.len() - 2)
            };
            Some(result)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.rest.len() / 2, Some(self.rest.len() / 2))
    }

    #[inline]
    fn count(self) -> usize {
        self.rest.len() / 2
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.rest.len() < 2 {
            None
        } else {
            let last_index = ((self.rest.len() / 2) - 1) * 2;
            let bytes = unsafe {
                // SAFETY: Safe as len checked to be at least 2.
                [
                    *self.rest.get_unchecked(last_index),
                    *self.rest.get_unchecked(last_index + 1),
                ]
            };
            Some(if self.is_big_endian {
                RawF16::from_be_bytes(bytes)
            } else {
                RawF16::from_le_bytes(bytes)
            })
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // Formula converted to ensure no overflow occurs:
        //    n*2 + 2 <= self.rest.len()
        //    n*2 <= self.rest.len() - 2
        //    n <= (self.rest.len() - 2) / 2
        if self.rest.len() >= 2 && n <= (self.rest.len() - 2) / 2 {
            let index = n * 2;
            let bytes = unsafe {
                [
                    // SAFETY: Safe as the length is checked beforehand to be at least n*2 + 2
                    *self.rest.get_unchecked(index),
                    *self.rest.get_unchecked(index + 1),
                ]
            };
            let result = if self.is_big_endian {
                RawF16::from_be_bytes(bytes)
            } else {
                RawF16::from_le_bytes(bytes)
            };
            self.rest = unsafe {
                // SAFETY: Safe as the length is checked beforehand to be at least n*2 + 2
                core::slice::from_raw_parts(
                    self.rest.as_ptr().add(index + 2),
                    self.rest.len() - index - 2,
                )
            };
            Some(result)
        } else {
            self.rest = unsafe {
                // SAFETY: Safe as the slice gets moved to its end with len 0.
                core::slice::from_raw_parts(self.rest.as_ptr().add(self.rest.len()), 0)
            };
            None
        }
    }
}

impl<'a> IntoIterator for &'a ArrayF16<'a> {
    type Item = RawF16;
    type IntoIter = ArrayF16Iterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::UnexpectedEndOfSliceError;
    use crate::error::VerboseDecodeError::UnexpectedEndOfSlice;
    use crate::verbose::VerboseValue;
    use crate::verbose::VerboseValue::ArrF16;
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use std::format;
    use std::mem::size_of;

    type TestType<'a> = ArrayF16<'a>;
    type InternalTypes = u16;

    proptest! {
        #[test]
        fn write_read(ref name in "\\pc{0,20}", ref unit in "\\pc{0,20}", dim_count in 0u16..5) {
            const TYPE_INFO_RAW: [u8; 4] = [0b1000_0010, 0b0000_0001, 0b0000_0000, 0b0000_0000];
            const VAR_INFO_FLAG: u8 = 0b0000_1000;

            const BUFFER_SIZE: usize = 400;

            // test big endian with name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = true;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 0..dim_count {
                        dimensions.extend_from_slice(&(i+1).to_be_bytes());
                    for x in 0..=i as InternalTypes {
                            content.extend_from_slice(&x.to_be_bytes());

                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;

                let len_name = (name.len() as u16 + 1).to_be_bytes();
                let len_unit = (unit.len() as u16 + 1).to_be_bytes();

                let mut content_buff = Vec::new();

                content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1] | VAR_INFO_FLAG, TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
                content_buff.extend_from_slice(&dim_count.to_be_bytes());
                content_buff.extend_from_slice(&dimensions);
                content_buff.extend_from_slice(&[len_name[0], len_name[1], len_unit[0], len_unit[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&content);

                prop_assert_eq!(&msg_buff[..], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((ArrF16(arr),&[] as &[u8])));

                }

            // test little endian with name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = false;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 0..dim_count {
                        dimensions.extend_from_slice(&(i+1).to_le_bytes());
                    for x in 0..=i as InternalTypes {
                            content.extend_from_slice(&x.to_le_bytes());

                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;

                let len_name = (name.len() as u16 + 1).to_le_bytes();
                let len_unit = (unit.len() as u16 + 1).to_le_bytes();

                let mut content_buff = Vec::new();

                content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1] | VAR_INFO_FLAG, TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
                content_buff.extend_from_slice(&dim_count.to_le_bytes());
                content_buff.extend_from_slice(&dimensions);
                content_buff.extend_from_slice(&[len_name[0], len_name[1], len_unit[0], len_unit[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&content);

                prop_assert_eq!(&msg_buff[..], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((ArrF16(arr),&[] as &[u8])));
            }

            // test big endian without name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = true;

                let variable_info = None;

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 0..dim_count {
                        dimensions.extend_from_slice(&(i+1).to_be_bytes());
                    for x in 0..=i as InternalTypes {
                            content.extend_from_slice(&x.to_be_bytes());

                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;
                let mut content_buff = Vec::new();

                content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1], TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
                content_buff.extend_from_slice(&dim_count.to_be_bytes());
                content_buff.extend_from_slice(&dimensions);
                content_buff.extend_from_slice(&content);

                prop_assert_eq!(&msg_buff[..], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((ArrF16(arr),&[] as &[u8])));
            }

            // test little endian without name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = false;

                let variable_info = None;

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 0..dim_count {
                        dimensions.extend_from_slice(&(i+1).to_le_bytes());
                    for x in 0..=i as InternalTypes {
                            content.extend_from_slice(&x.to_le_bytes());

                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;

                let mut content_buff = Vec::new();
                content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1], TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
                content_buff.extend_from_slice(&dim_count.to_le_bytes());
                content_buff.extend_from_slice(&dimensions);
                content_buff.extend_from_slice(&content);

                prop_assert_eq!(&msg_buff[..], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((ArrF16(arr),&[] as &[u8])));
            }


             // Capacity error big endian with name
             {
                let dim_count = dim_count + 1;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = true;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 1u16..=dim_count {
                    dimensions.extend_from_slice(&(i as u16).to_be_bytes());

                    for x in 0..(i-1) as InternalTypes {
                            content.extend_from_slice(&x.to_be_bytes());

                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;


                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                    layer: crate::error::Layer::VerboseValue,
                    minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize,
                    actual_size: msg_buff.len()
                })));
            }

             // Capacity error little endian with name
             {
                let dim_count = dim_count + 1;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = false;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 1u16..=dim_count {
                    dimensions.extend_from_slice(&(i as u16).to_le_bytes());

                    for x in 0..(i-1) as InternalTypes {
                            content.extend_from_slice(&x.to_le_bytes());

                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));
            }


             // Capacity error big endian without name
             {
                let dim_count = dim_count + 1;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = true;

                let variable_info = None;

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 1u16..=dim_count {
                    dimensions.extend_from_slice(&(i as u16).to_be_bytes());

                    for x in 0..(i-1) as InternalTypes {

                            content.extend_from_slice(&(x).to_be_bytes());

                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));

            }

             // Capacity error little endian without name
             {
                let dim_count = dim_count + 1;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = false;

                let variable_info = None;

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 1u16..=dim_count {
                    dimensions.extend_from_slice(&(i as u16).to_le_bytes());

                    for x in 0..(i-1) as InternalTypes {
                            content.extend_from_slice(&x.to_le_bytes());

                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));
            }


             // capacity error big endian with name 2
             {
                let name = "Abc";
                let unit = "Xyz";
                const DIM_COUNT: u16 = 5;
                const BUFFER_SIZE: usize = DIM_COUNT as usize * 2 + 14;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = true;

                let variable_info = Some(VariableInfoUnit { name , unit });
                let mut dimensions = Vec::with_capacity(DIM_COUNT as usize);
                let mut content = Vec::with_capacity(DIM_COUNT as usize);

                for i in 0..DIM_COUNT as InternalTypes {
                        dimensions.extend_from_slice(&(1 as InternalTypes).to_be_bytes());
                            content.extend_from_slice(&i.to_be_bytes());


                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                let err = arr.add_to_msg(&mut msg_buff, is_big_endian);

                prop_assert_eq!(err, Err(CapacityError::new(())));

                }


             // capacity error little endian with name
             {
                let name = "Abc";
                let unit = "Xyz";
                const DIM_COUNT: u16 = 5;
                const BUFFER_SIZE: usize = DIM_COUNT as usize * 2 + 15;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = true;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let mut dimensions = Vec::with_capacity(DIM_COUNT as usize);
                let mut content = Vec::with_capacity(DIM_COUNT as usize);

                for i in 0..DIM_COUNT as InternalTypes {
                        dimensions.extend_from_slice(&(1 as u16).to_le_bytes());
                            content.extend_from_slice(&i.to_le_bytes());


                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content };
                let err = arr.add_to_msg(&mut msg_buff, is_big_endian);

                prop_assert_eq!(err, Err(CapacityError::new(())));

                }
        }
    }

    proptest! {
        #[test]
        fn data(ref data in "\\pc{0,100}", ref dimensions in "\\pc{0,100}") {

            let arr_dim = ArrayDimensions {dimensions: dimensions.as_bytes(), is_big_endian: true };
            let arr = TestType {is_big_endian: true, dimensions:arr_dim,variable_info:None,data:data.as_bytes() };
            prop_assert_eq!(arr.data(), data.as_bytes());
        }
    }

    proptest! {
        #[test]
        fn next(
            value0 in any::<u16>(),
            value1 in any::<u16>()
        ) {

            // empty
            {
                let mut iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &[],
                };
                assert!(iter.next().is_none());
            }

            let value0_be = value0.to_be_bytes();
            let value1_be = value1.to_be_bytes();
            let value0_le = value0.to_le_bytes();
            let value1_le = value1.to_le_bytes();

            // big endian (aligned)
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1]];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                };
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }

            // little endian (aligned)
            {
                let bytes = [value0_le[0], value0_le[1], value1_le[0], value1_le[1]];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &bytes,
                };
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }

            // big endian (unaligned)
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1], 0];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                };
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }

            // little endian (unaligned)
            {
                let bytes = [value0_le[0], value0_le[1], value1_le[0], value1_le[1], 0];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &bytes,
                };
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }
        }
    }

    proptest! {
        #[test]
        fn size_hint(
            value0 in any::<u16>(),
            value1 in any::<u16>()
        ) {

            // empty
            {
                let iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &[],
                };
                assert_eq!((0, Some(0)), iter.size_hint());
            }

            let value0_be = value0.to_be_bytes();
            let value1_be = value1.to_be_bytes();

            // Aligned
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1]];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                };

                assert_eq!((2, Some(2)), iter.size_hint());
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!((1, Some(1)), iter.size_hint());
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!((0, Some(0)), iter.size_hint());
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }

            // Unaligned
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1], 0];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                };

                assert_eq!((2, Some(2)), iter.size_hint());
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!((1, Some(1)), iter.size_hint());
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!((0, Some(0)), iter.size_hint());
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }
        }
    }

    proptest! {
        #[test]
        fn count(
            value0 in any::<u16>(),
            value1 in any::<u16>()
        ) {

            // empty
            {
                let iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &[],
                };
                assert_eq!(0, iter.count());
            }

            let value0_be = value0.to_be_bytes();
            let value1_be = value1.to_be_bytes();

            // Aligned
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1]];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                };

                assert_eq!(2, iter.clone().count());
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(1, iter.clone().count());
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(0, iter.clone().count());
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }

            // Unaligned
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1], 0];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                };

                assert_eq!(2, iter.clone().count());
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(1, iter.clone().count());
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(0, iter.clone().count());
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }
        }
    }

    proptest! {
        #[test]
        fn last(
            value0 in any::<u16>(),
            value1 in any::<u16>()
        ) {

            // empty
            {
                let iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &[],
                };
                assert!(iter.last().is_none());
            }

            let value0_be = value0.to_be_bytes();
            let value1_be = value1.to_be_bytes();
            let value0_le = value0.to_le_bytes();
            let value1_le = value1.to_le_bytes();

            // big endian (aligned)
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1]];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                };

                assert_eq!(Some(value1), iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(Some(value1), iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(None, iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }

            // little endian (aligned)
            {
                let bytes = [value0_le[0], value0_le[1], value1_le[0], value1_le[1]];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &bytes,
                };

                assert_eq!(Some(value1), iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(Some(value1), iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(None, iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }

            // big endian (unaligned)
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1], 0];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                };

                assert_eq!(Some(value1), iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(Some(value1), iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(None, iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }

            // little endian (unaligned)
            {
                let bytes = [value0_le[0], value0_le[1], value1_le[0], value1_le[1], 0];
                let mut iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &bytes,
                };

                assert_eq!(Some(value1), iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    Some(value0),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(Some(value1), iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    Some(value1),
                    iter.next().map(|v| v.to_bits())
                );

                assert_eq!(None, iter.clone().last().map(|v| v.to_bits()));
                assert_eq!(
                    None,
                    iter.next().map(|v| v.to_bits())
                );
            }
        }
    }

    proptest! {
        #[test]
        fn nth(
            value0 in any::<u16>(),
            value1 in any::<u16>()
        ) {

            // empty
            {
                let mut iter = ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &[],
                };
                assert!(iter.nth(0).is_none());
                assert!(iter.nth(1).is_none());
            }

            let value0_be = value0.to_be_bytes();
            let value1_be = value1.to_be_bytes();
            let value0_le = value0.to_le_bytes();
            let value1_le = value1.to_le_bytes();

            let test_nth = |iter: ArrayF16Iterator| {
                {
                    let mut iter = iter.clone();
                    assert_eq!(
                        Some(value0),
                        iter.nth(0).map(|v| v.to_bits())
                    );
                    assert_eq!(
                        Some(value1),
                        iter.nth(0).map(|v| v.to_bits())
                    );
                    assert_eq!(
                        None,
                        iter.nth(0).map(|v| v.to_bits())
                    );
                }
                {
                    let mut iter = iter.clone();
                    assert_eq!(
                        Some(value1),
                        iter.nth(1).map(|v| v.to_bits())
                    );
                    assert_eq!(
                        None,
                        iter.nth(0).map(|v| v.to_bits())
                    );
                }
                {
                    let mut iter = iter.clone();
                    assert_eq!(
                        None,
                        iter.nth(2).map(|v| v.to_bits())
                    );
                }
            };

            // big endian (aligned)
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1]];
                test_nth(ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                });
            }

            // little endian (aligned)
            {
                let bytes = [value0_le[0], value0_le[1], value1_le[0], value1_le[1]];
                test_nth(ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &bytes,
                });
            }

            // big endian (unaligned)
            {
                let bytes = [value0_be[0], value0_be[1], value1_be[0], value1_be[1], 0];
                test_nth(ArrayF16Iterator{
                    is_big_endian: true,
                    rest: &bytes,
                });
            }

            // little endian (unaligned)
            {
                let bytes = [value0_le[0], value0_le[1], value1_le[0], value1_le[1], 0];
                test_nth(ArrayF16Iterator{
                    is_big_endian: false,
                    rest: &bytes,
                });
            }
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialization() {
        // test dim_count 0

        use alloc::string::ToString;
        {
            let dim_count: u16 = 0;
            let is_big_endian = true;

            let variable_info = None;

            let mut dimensions = Vec::with_capacity(dim_count as usize);
            let mut content = Vec::with_capacity(dim_count as usize);

            let mut elems: u8 = 1;

            for i in 0..dim_count {
                dimensions.extend_from_slice(&(i + 1).to_be_bytes());
                elems *= (i + 1) as u8;
            }

            for x in 0u8..elems as u8 {
                content.extend_from_slice(&(x as InternalTypes).to_be_bytes());
            }

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr = TestType {
                variable_info,
                dimensions: arr_dim,
                data: &content,
                is_big_endian,
            };

            let convert_content = "{\"variable_info\":null,\"data\":[]}".to_string();

            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
        }

        // test dim_count 1
        {
            let dim_count: u16 = 1;
            let is_big_endian = true;

            let variable_info = None;

            let mut dimensions = Vec::with_capacity(dim_count as usize);
            let mut content = Vec::with_capacity(dim_count as usize);

            for i in 0..dim_count {
                dimensions.extend_from_slice(&(i + 1).to_be_bytes());
            }

            content.extend_from_slice(&RawF16::ZERO.to_be_bytes());

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr = TestType {
                variable_info,
                dimensions: arr_dim,
                data: &content,
                is_big_endian,
            };

            let convert_content = "{\"variable_info\":null,\"data\":[0.0]}".to_string();

            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
        }

        // test dim_count 2
        {
            let dim_count: u16 = 2;
            let is_big_endian = true;

            let variable_info = None;

            let mut dimensions = Vec::with_capacity(dim_count as usize);
            let mut content = Vec::with_capacity(dim_count as usize);

            for i in 0..dim_count {
                dimensions.extend_from_slice(&(i + 1).to_be_bytes());
            }

            content.extend_from_slice(&RawF16::ZERO.to_be_bytes());
            content.extend_from_slice(&RawF16::ONE.to_be_bytes());

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr = TestType {
                variable_info,
                dimensions: arr_dim,
                data: &content,
                is_big_endian,
            };

            let convert_content = "{\"variable_info\":null,\"data\":[[0.0,1.0]]}".to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
        }

        // test dim_count 3
        {
            let dim_count: u16 = 3;
            let is_big_endian = true;
            let variable_info = None;

            let mut dimensions = Vec::with_capacity(dim_count as usize);
            let mut content = Vec::with_capacity(dim_count as usize);

            for i in 0..dim_count {
                dimensions.extend_from_slice(&(i + 1).to_be_bytes());
            }

            content.extend_from_slice(&RawF16::ZERO.to_be_bytes());
            content.extend_from_slice(&RawF16::ONE.to_be_bytes());
            content.extend_from_slice(&RawF16::ZERO.to_be_bytes());
            content.extend_from_slice(&RawF16::ONE.to_be_bytes());
            content.extend_from_slice(&RawF16::ZERO.to_be_bytes());
            content.extend_from_slice(&RawF16::ONE.to_be_bytes());

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr = TestType {
                variable_info,
                dimensions: arr_dim,
                data: &content,
                is_big_endian,
            };

            let convert_content =
                "{\"variable_info\":null,\"data\":[[[0.0,1.0,0.0],[1.0,0.0,1.0]]]}".to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
        }
    }
}
