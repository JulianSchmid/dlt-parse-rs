use crate::verbose::{ArrayDimensions, VariableInfoUnit};

use arrayvec::{ArrayVec, CapacityError};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

#[cfg(feature = "serde")]
use super::ArrayItDimension;

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayBool<'a> {
    pub dimensions: ArrayDimensions<'a>,
    pub variable_info: Option<VariableInfoUnit<'a>>,
    pub(crate) data: &'a [u8],
}

#[derive(Clone, Debug)]
pub struct ArrayBoolIterator<'a> {
    pub(crate) rest: &'a [u8],
}

#[cfg(feature = "serde")]
impl<'a> Serialize for ArrayBool<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArrayBool", 3)?;
        state.serialize_field("variable_info", &self.variable_info)?;

        let iter = ArrayItDimension::<bool> {
            is_big_endian: self.dimensions.is_big_endian,
            dimensions: self.dimensions.dimensions,
            data: self.data,
            phantom: Default::default(),
        };
        state.serialize_field("data", &iter)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'a> Serialize for ArrayBoolIterator<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.rest.len()))?;
        for e in self.clone() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}

impl<'a> ArrayBool<'a> {
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
    pub fn iter(&'a self) -> ArrayBoolIterator<'a> {
        ArrayBoolIterator { rest: self.data }
    }
    /// Adds the verbose value to the given dlt mesage buffer.
    pub fn add_to_msg<const CAP: usize>(
        &self,
        buf: &mut ArrayVec<u8, CAP>,
        is_big_endian: bool,
    ) -> Result<(), CapacityError> {
        if let Some(var_info) = &self.variable_info {
            let type_info: [u8; 4] = [0b0001_0001, 0b0000_1001, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;

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

            buf.try_extend_from_slice(&number_of_dimensions)?;
            buf.try_extend_from_slice(self.dimensions.dimensions)?;
            buf.try_extend_from_slice(&[name_len[0], name_len[1], unit_len[0], unit_len[1]])?;
            buf.try_extend_from_slice(var_info.name.as_bytes())?;
            if buf.remaining_capacity() > var_info.unit.len() + 2 {
                // Safe as capacity is checked earlier
                unsafe { buf.push_unchecked(u8::from(0)) };
                let _ = buf.try_extend_from_slice(var_info.unit.as_bytes());
                unsafe { buf.push_unchecked(u8::from(0)) };
                buf.try_extend_from_slice(self.data)?;
                Ok(())
            } else {
                return Err(CapacityError::new(()));
            }
        } else {
            let number_of_dimensions = match is_big_endian {
                true => (self.dimensions.dimensions.len() as u16 / 2).to_be_bytes(),
                false => (self.dimensions.dimensions.len() as u16 / 2).to_le_bytes(),
            };
            let type_info: [u8; 4] = [0b0001_0001, 0b0000_0001, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&number_of_dimensions)?;
            buf.try_extend_from_slice(self.dimensions.dimensions)?;
            buf.try_extend_from_slice(self.data)?;

            Ok(())
        }
    }
}

impl Iterator for ArrayBoolIterator<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.len() == 0 {
            None
        } else {
            let result = self.rest[0] != 0;
            self.rest = &self.rest[1..];
            Some(result)
        }
    }
}

impl<'a> IntoIterator for &'a ArrayBool<'a> {
    type Item = bool;
    type IntoIter = ArrayBoolIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(feature = "serde_json")]
    use crate::alloc::string::ToString;
    use crate::error::UnexpectedEndOfSliceError;
    use crate::error::VerboseDecodeError::UnexpectedEndOfSlice;
    use crate::verbose::VerboseValue;
    use crate::verbose::VerboseValue::ArrBool;
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use std::format;

    proptest! {
        #[test]
        fn write_read(ref name in "\\pc{0,20}", ref unit in "\\pc{0,20}", dim_count in 0u16..5) {
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
                for x in 0..=i {
                    content.push(u8::from(x % 2 == 0));       // Sample booleans
                }
            }

            let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

            let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
            arr_bool.add_to_msg(&mut msg_buff, is_big_endian)?;

            let len_name_be = (name.len() as u16 + 1).to_be_bytes();
            let len_unit_be = (unit.len() as u16 + 1).to_be_bytes();

            let mut content_buff = Vec::new();

            content_buff.extend_from_slice(&[0b0001_0001, 0b0000_1001, 0b0000_0000, 0b0000_0000]);
            content_buff.extend_from_slice(&dim_count.to_be_bytes());
            content_buff.extend_from_slice(&dimensions);
            content_buff.extend_from_slice(&[len_name_be[0], len_name_be[1], len_unit_be[0], len_unit_be[1]]);
            content_buff.extend_from_slice(name.as_bytes());
            content_buff.push(0);
            content_buff.extend_from_slice(unit.as_bytes());
            content_buff.push(0);
            content_buff.extend_from_slice(&content);

            prop_assert_eq!(&msg_buff[..], &content_buff[..]);

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
            prop_assert_eq!(parsed_back, Ok((ArrBool(arr_bool),&[] as &[u8])));

            }

            // test little endian with name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = false;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 0..dim_count {
                    dimensions.extend_from_slice(&(i + 1).to_le_bytes());
                    for x in 0..=i {
                        content.push(u8::from(x % 2 == 0));       // Sample booleans
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
                arr_bool.add_to_msg(&mut msg_buff, is_big_endian)?;

                let len_name_le = (name.len() as u16 + 1).to_le_bytes();
                let len_unit_le = (unit.len() as u16 + 1).to_le_bytes();

                let mut content_buff = Vec::new();

                content_buff.extend_from_slice(&[0b0001_0001, 0b0000_1001, 0b0000_0000, 0b0000_0000]);
                content_buff.extend_from_slice(&dim_count.to_le_bytes());
                content_buff.extend_from_slice(&dimensions);
                content_buff.extend_from_slice(&[len_name_le[0], len_name_le[1], len_unit_le[0], len_unit_le[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&content);

                prop_assert_eq!(&msg_buff[..], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((ArrBool(arr_bool),&[] as &[u8])));

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
                    for x in 0..=i {
                        content.push(u8::from(x % 2 == 0));       // Sample booleans
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
                arr_bool.add_to_msg(&mut msg_buff, is_big_endian)?;

                let mut content_buff = Vec::new();

                content_buff.extend_from_slice(&[0b0001_0001, 0b0000_0001, 0b0000_0000, 0b0000_0000]);
                content_buff.extend_from_slice(&dim_count.to_be_bytes());
                content_buff.extend_from_slice(&dimensions);
                content_buff.extend_from_slice(&content);

                prop_assert_eq!(&msg_buff[..], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((ArrBool(arr_bool),&[] as &[u8])));

                }

        // test little endian without name
        {
            let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
            let is_big_endian = false;

            let variable_info = None;

            let mut dimensions = Vec::with_capacity(dim_count as usize);
            let mut content = Vec::with_capacity(dim_count as usize);

            for i in 0..dim_count {
                    dimensions.extend_from_slice(&(i + 1).to_le_bytes());
                for x in 0..=i {
                    content.push(u8::from(x % 2 == 0));       // Sample booleans
                }
            }

            let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

            let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
            arr_bool.add_to_msg(&mut msg_buff, is_big_endian)?;

            let mut content_buff = Vec::new();

            content_buff.extend_from_slice(&[0b0001_0001, 0b0000_0001, 0b0000_0000, 0b0000_0000]);
            content_buff.extend_from_slice(&dim_count.to_le_bytes());
            content_buff.extend_from_slice(&dimensions);
            content_buff.extend_from_slice(&content);

            prop_assert_eq!(&msg_buff[..], &content_buff[..]);

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
            prop_assert_eq!(parsed_back, Ok((ArrBool(arr_bool),&[] as &[u8])));

            }

            // Capacity error big endian with name
            {
                let dim_count = dim_count + 1;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = true;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);


                for i in 0..dim_count {
                    dimensions.extend_from_slice(&(i+1).to_be_bytes());
                    for x in 0..i {
                        content.push(u8::from(x % 2 == 0));       // Sample booleans
                    }
                }


                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
                arr_bool.add_to_msg(&mut msg_buff, is_big_endian)?;

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len()+dim_count as usize, actual_size: msg_buff.len() })));

            }

            // Capacity error little endian with name
            {
                let dim_count = dim_count + 1;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = false;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);


                for i in 0..dim_count {

                    dimensions.extend_from_slice(&(i + 1).to_le_bytes());
                    for x in 0..i {
                        content.push(u8::from(x % 2 == 0));       // Sample booleans
                    }
                }


                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
                arr_bool.add_to_msg(&mut msg_buff, is_big_endian)?;

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len()+dim_count as usize, actual_size: msg_buff.len() })));

            }

            // Capacity error big endian without name
            {
                let dim_count = dim_count + 1;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = true;

                let variable_info = None;

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);


                for i in 0..dim_count {
                        dimensions.extend_from_slice(&(i+1).to_be_bytes());

                    for x in 0..i {
                        content.push(u8::from(x % 2 == 0));       // Sample booleans
                    }
                }


                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
                arr_bool.add_to_msg(&mut msg_buff, is_big_endian)?;

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len()+dim_count as usize, actual_size: msg_buff.len() })));

            }

            // Capacity error little endian without name
            {
                let dim_count = dim_count + 1;
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let is_big_endian = false;

                let variable_info = None;

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);


                for i in 0..dim_count {

                        dimensions.extend_from_slice(&(i + 1).to_le_bytes());

                    for x in 0..i {
                        content.push(u8::from(x % 2 == 0));       // Sample booleans
                    }
                }


                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
                arr_bool.add_to_msg(&mut msg_buff, is_big_endian)?;

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len()+dim_count as usize, actual_size: msg_buff.len() })));

            }


             // capacity error big endian with name
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

                for i in 0..DIM_COUNT {
                        dimensions.extend_from_slice(&(1 as u16).to_be_bytes());
                        content.push(u8::from(i % 2 == 0));       // Sample booleans

                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr_bool = ArrayBool {variable_info, dimensions: arr_dim, data: &content };
                let err = arr_bool.add_to_msg(&mut msg_buff, is_big_endian);

                prop_assert_eq!(err, Err(CapacityError::new(())));

                }



        }
    }

    proptest! {
        #[test]
        fn data(ref data in "\\pc{0,100}", ref dimensions in "\\pc{0,100}") {

            let arr_dim = ArrayDimensions {dimensions: dimensions.as_bytes(), is_big_endian: true };
            let arr_bool = ArrayBool { dimensions: arr_dim, variable_info: None, data: data.as_bytes()};
            prop_assert_eq!(arr_bool.data(), data.as_bytes());
        }
    }

    proptest! {
        #[test]
        fn iterator(dim_count in 0u16..5) {

            // test big endian without name
            {
                let is_big_endian = true;

                let variable_info = None;

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 0..dim_count {
                        dimensions.extend_from_slice(&(i+1).to_be_bytes());
                    for x in 0u8..=i as u8 {
                        content.push(x % 2);
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr_u8 = ArrayBool {variable_info, dimensions:arr_dim,data: &content };

                let mut cnt = 0;
                for item in arr_u8.iter() {
                    prop_assert_eq!(item, content[cnt] != 0);
                    cnt += 1;
                }

                }
        }
    }
    #[cfg(feature = "serde")]
    #[cfg(feature = "serde_json")]
    #[test]
    fn serialization() {
        // test dim_count 0
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
                content.push(x % 2);
            }

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr_u8 = ArrayBool {
                variable_info,
                dimensions: arr_dim,
                data: &content,
            };

            let mut cnt = 0;
            for item in arr_u8.iter() {
                assert_eq!(item, content[cnt] != 0);
                cnt += 1;
            }

            let convert_content = "{\"variable_info\":null,\"data\":[]}".to_string();

            assert_eq!(convert_content, serde_json::to_string(&arr_u8).unwrap());
        }

        // test dim_count 1
        {
            let dim_count: u16 = 1;
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
                content.push(x % 2);
            }

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr_u8 = ArrayBool {
                variable_info,
                dimensions: arr_dim,
                data: &content,
            };

            let mut cnt = 0;
            for item in arr_u8.iter() {
                assert_eq!(item, content[cnt] != 0);
                cnt += 1;
            }

            let convert_content = "{\"variable_info\":null,\"data\":[false]}".to_string();

            assert_eq!(convert_content, serde_json::to_string(&arr_u8).unwrap());
        }

        // test dim_count 2
        {
            let dim_count: u16 = 2;
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
                content.push(x % 2);
            }

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr_u8 = ArrayBool {
                variable_info,
                dimensions: arr_dim,
                data: &content,
            };

            let mut cnt = 0;
            for item in arr_u8.iter() {
                assert_eq!(item, content[cnt] != 0);
                cnt += 1;
            }

            let convert_content = "{\"variable_info\":null,\"data\":[[false,true]]}".to_string();

            assert_eq!(convert_content, serde_json::to_string(&arr_u8).unwrap());
        }

        // test dim_count 3
        {
            let dim_count: u16 = 3;
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
                content.push(x % 2);
            }

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr_u8 = ArrayBool {
                variable_info,
                dimensions: arr_dim,
                data: &content,
            };

            let mut cnt = 0;
            for item in arr_u8.iter() {
                assert_eq!(item, content[cnt] != 0);
                cnt += 1;
            }

            let convert_content =
                "{\"variable_info\":null,\"data\":[[[false,true,false],[true,false,true]]]}"
                    .to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr_u8).unwrap());
        }

        // test dim_count 4
        {
            let dim_count: u16 = 4;
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
                content.push(x % 2);
            }

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr_u8 = ArrayBool {
                variable_info,
                dimensions: arr_dim,
                data: &content,
            };

            let mut cnt = 0;
            for item in arr_u8.iter() {
                assert_eq!(item, content[cnt] != 0);
                cnt += 1;
            }

            let convert_content = "{\"variable_info\":null,\"data\":[[[[false,true,false,true],[false,true,false,true],[false,true,false,true]],[[false,true,false,true],[false,true,false,true],[false,true,false,true]]]]}".to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr_u8).unwrap());
        }

        // test dim_count 5
        {
            let dim_count: u16 = 5;
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
                content.push(x % 2);
            }

            let arr_dim = ArrayDimensions {
                is_big_endian,
                dimensions: &dimensions,
            };
            let arr_u8 = ArrayBool {
                variable_info,
                dimensions: arr_dim,
                data: &content,
            };

            let mut cnt = 0;
            for item in arr_u8.iter() {
                assert_eq!(item, content[cnt] != 0);
                cnt += 1;
            }

            let convert_content = "{\"variable_info\":null,\"data\":[[[[[false,true,false,true,false],[true,false,true,false,true],[false,true,false,true,false],[true,false,true,false,true]],[[false,true,false,true,false],[true,false,true,false,true],[false,true,false,true,false],[true,false,true,false,true]],[[false,true,false,true,false],[true,false,true,false,true],[false,true,false,true,false],[true,false,true,false,true]]],[[[false,true,false,true,false],[true,false,true,false,true],[false,true,false,true,false],[true,false,true,false,true]],[[false,true,false,true,false],[true,false,true,false,true],[false,true,false,true,false],[true,false,true,false,true]],[[false,true,false,true,false],[true,false,true,false,true],[false,true,false,true,false],[true,false,true,false,true]]]]]}".to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr_u8).unwrap());
        }
    }
}
