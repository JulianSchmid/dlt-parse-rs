use crate::verbose::{ArrayDimensions, VariableInfoUnit};

use arrayvec::{ArrayVec, CapacityError};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

#[cfg(feature = "serde")]
use super::ArrayItDimension;

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayF64<'a> {
    pub is_big_endian: bool,
    pub dimensions: ArrayDimensions<'a>,
    pub variable_info: Option<VariableInfoUnit<'a>>,
    pub(crate) data: &'a [u8],
}

impl<'a> ArrayF64<'a> {
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
    pub fn iter(&'a self) -> ArrayF64Iterator<'a> {
        ArrayF64Iterator {
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

            let type_info: [u8; 4] = [0b1000_0100, 0b0000_1001, 0b0000_0000, 0b0000_0000];
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
            let type_info: [u8; 4] = [0b1000_0100, 0b0000_0001, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&number_of_dimensions)?;
            buf.try_extend_from_slice(self.dimensions.dimensions)?;
            buf.try_extend_from_slice(self.data)?;
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
impl<'a> Serialize for ArrayF64<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArrayF64", 2)?;
        state.serialize_field("variable_info", &self.variable_info)?;

        let iter = ArrayItDimension::<f64> {
            is_big_endian: self.is_big_endian,
            dimensions: self.dimensions.dimensions,
            data: self.data,
            phantom: Default::default(),
        };
        state.serialize_field("data", &iter)?;

        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct ArrayF64Iterator<'a> {
    pub(crate) is_big_endian: bool,
    pub(crate) rest: &'a [u8],
}

impl Iterator for ArrayF64Iterator<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.len() < 8 {
            None
        } else {
            let result = if self.is_big_endian {
                f64::from_be_bytes([
                    self.rest[0],
                    self.rest[1],
                    self.rest[2],
                    self.rest[3],
                    self.rest[4],
                    self.rest[5],
                    self.rest[6],
                    self.rest[7],
                ])
            } else {
                f64::from_le_bytes([
                    self.rest[0],
                    self.rest[1],
                    self.rest[2],
                    self.rest[3],
                    self.rest[4],
                    self.rest[5],
                    self.rest[6],
                    self.rest[7],
                ])
            };
            self.rest = &self.rest[8..];
            Some(result)
        }
    }
}

impl<'a> IntoIterator for &'a ArrayF64<'a> {
    type Item = f64;
    type IntoIter = ArrayF64Iterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "serde")]
impl<'a> Serialize for ArrayF64Iterator<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.rest.len() / 8))?;
        for e in self.clone() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::UnexpectedEndOfSliceError;
    use crate::error::VerboseDecodeError::UnexpectedEndOfSlice;
    use crate::verbose::VerboseValue;
    use crate::verbose::VerboseValue::ArrF64;
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use std::format;
    use std::mem::size_of;

    type TestType<'a> = ArrayF64<'a>;
    type InternalTypes = f64;

    proptest! {
        #[test]
        fn write_read(ref name in "\\pc{0,20}", ref unit in "\\pc{0,20}", dim_count in 0u16..5) {
            const TYPE_INFO_RAW: [u8; 4] = [0b1000_0100, 0b0000_0001, 0b0000_0000, 0b0000_0000];
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
                    for x in 0..=i as i16 {
                        if x % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(x)).to_be_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1* x)).to_be_bytes());
                        }
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
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
                prop_assert_eq!(parsed_back, Ok((ArrF64(arr),&[] as &[u8])));

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
                    for x in 0..=i as i16 {
                        if x % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(x)).to_le_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1 * x)).to_le_bytes());
                        }
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
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
                prop_assert_eq!(parsed_back, Ok((ArrF64(arr),&[] as &[u8])));

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
                    for x in 0..=i as i16 {
                        if x % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(x)).to_be_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1 * x)).to_be_bytes());
                        }
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;
                let mut content_buff = Vec::new();

                content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1], TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
                content_buff.extend_from_slice(&dim_count.to_be_bytes());
                content_buff.extend_from_slice(&dimensions);
                content_buff.extend_from_slice(&content);

                prop_assert_eq!(&msg_buff[..], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((ArrF64(arr),&[] as &[u8])));

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
                    for x in 0..=i as i16 {
                        if x % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(x)).to_le_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1 * x)).to_le_bytes());
                        }
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;

                let mut content_buff = Vec::new();
                content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1], TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
                content_buff.extend_from_slice(&dim_count.to_le_bytes());
                content_buff.extend_from_slice(&dimensions);
                content_buff.extend_from_slice(&content);

                prop_assert_eq!(&msg_buff[..], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((ArrF64(arr),&[] as &[u8])));

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

                    for x in 0..(i-1) as i16 {
                        if x % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(x)).to_be_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1* x)).to_be_bytes());
                        }
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
                arr.add_to_msg(&mut msg_buff, is_big_endian)?;


                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));

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

                    for x in 0..(i-1) as i16 {
                        if x % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(x)).to_le_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1 * x)).to_le_bytes());
                        }
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
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

                    for x in 0..(i-1) as i16 {
                        if x % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(x)).to_be_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1 * x)).to_be_bytes());
                        }
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
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

                    for x in 0..(i-1) as i16 {
                        if x % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(x)).to_le_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1 * x)).to_le_bytes());
                        }
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
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

                for i in 0..DIM_COUNT as i16 {
                        dimensions.extend_from_slice(&(1 as InternalTypes).to_be_bytes());
                        if i % 2 == 1 {
                            content.extend_from_slice(&i.to_be_bytes());
                        }
                        else {
                            content.extend_from_slice(&(-1* i).to_be_bytes());
                        }

                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
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

                for i in 0..DIM_COUNT as i16 {
                        dimensions.extend_from_slice(&(1 as u16).to_le_bytes());
                        if i % 2 == 1 {
                            content.extend_from_slice(&(InternalTypes::from(i)).to_le_bytes());
                        }
                        else {
                            content.extend_from_slice(&(InternalTypes::from(-1 * i)).to_le_bytes());
                        }

                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

                let arr = TestType {is_big_endian, variable_info,dimensions:arr_dim,data: &content };
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
                if x % 2 == 1 {
                    content.extend_from_slice(&(-0.5 * x as InternalTypes).to_be_bytes());
                } else {
                    content.extend_from_slice(&(x as InternalTypes).to_be_bytes());
                }
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

            let mut elems: u8 = 1;

            for i in 0..dim_count {
                dimensions.extend_from_slice(&(i + 1).to_be_bytes());
                elems *= (i + 1) as u8;
            }

            for x in 0u8..elems as u8 {
                if x % 2 == 1 {
                    content.extend_from_slice(&(-0.5 * x as InternalTypes).to_be_bytes());
                } else {
                    content.extend_from_slice(&(x as InternalTypes).to_be_bytes());
                }
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

            let mut elems: u8 = 1;

            for i in 0..dim_count {
                dimensions.extend_from_slice(&(i + 1).to_be_bytes());
                elems *= (i + 1) as u8;
            }

            for x in 0u8..elems as u8 {
                if x % 2 == 1 {
                    content.extend_from_slice(&(-0.5 * x as InternalTypes).to_be_bytes());
                } else {
                    content.extend_from_slice(&(x as InternalTypes).to_be_bytes());
                }
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

            let convert_content = "{\"variable_info\":null,\"data\":[[0.0,-0.5]]}".to_string();

            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
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
                if x % 2 == 1 {
                    content.extend_from_slice(&(-0.5 * x as InternalTypes).to_be_bytes());
                } else {
                    content.extend_from_slice(&(x as InternalTypes).to_be_bytes());
                }
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

            let convert_content =
                "{\"variable_info\":null,\"data\":[[[0.0,-0.5,2.0],[-1.5,4.0,-2.5]]]}".to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
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
                if x % 2 == 1 {
                    content.extend_from_slice(&(-0.5 * x as InternalTypes).to_be_bytes());
                } else {
                    content.extend_from_slice(&(x as InternalTypes).to_be_bytes());
                }
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

            let convert_content = "{\"variable_info\":null,\"data\":[[[[0.0,-0.5,2.0,-1.5],[4.0,-2.5,6.0,-3.5],[8.0,-4.5,10.0,-5.5]],[[12.0,-6.5,14.0,-7.5],[16.0,-8.5,18.0,-9.5],[20.0,-10.5,22.0,-11.5]]]]}".to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
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
                if x % 2 == 1 {
                    content.extend_from_slice(&(-0.5 * x as InternalTypes).to_be_bytes());
                } else {
                    content.extend_from_slice(&(x as InternalTypes).to_be_bytes());
                }
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

            let convert_content = "{\"variable_info\":null,\"data\":[[[[[0.0,-0.5,2.0,-1.5,4.0],[-2.5,6.0,-3.5,8.0,-4.5],[10.0,-5.5,12.0,-6.5,14.0],[-7.5,16.0,-8.5,18.0,-9.5]],[[20.0,-10.5,22.0,-11.5,24.0],[-12.5,26.0,-13.5,28.0,-14.5],[30.0,-15.5,32.0,-16.5,34.0],[-17.5,36.0,-18.5,38.0,-19.5]],[[40.0,-20.5,42.0,-21.5,44.0],[-22.5,46.0,-23.5,48.0,-24.5],[50.0,-25.5,52.0,-26.5,54.0],[-27.5,56.0,-28.5,58.0,-29.5]]],[[[60.0,-30.5,62.0,-31.5,64.0],[-32.5,66.0,-33.5,68.0,-34.5],[70.0,-35.5,72.0,-36.5,74.0],[-37.5,76.0,-38.5,78.0,-39.5]],[[80.0,-40.5,82.0,-41.5,84.0],[-42.5,86.0,-43.5,88.0,-44.5],[90.0,-45.5,92.0,-46.5,94.0],[-47.5,96.0,-48.5,98.0,-49.5]],[[100.0,-50.5,102.0,-51.5,104.0],[-52.5,106.0,-53.5,108.0,-54.5],[110.0,-55.5,112.0,-56.5,114.0],[-57.5,116.0,-58.5,118.0,-59.5]]]]]}".to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
        }
    }
}
