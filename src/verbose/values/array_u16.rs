use crate::verbose::{ArrayDimensions, Scaling, VariableInfoUnit};

#[cfg(feature = "serde")]
use super::ArrayItDimension;
use arrayvec::{ArrayVec, CapacityError};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayU16<'a> {
    pub is_big_endian: bool,
    pub dimensions: ArrayDimensions<'a>,
    pub variable_info: Option<VariableInfoUnit<'a>>,
    pub scaling: Option<Scaling<i32>>,
    pub(crate) data: &'a [u8],
}

impl<'a> ArrayU16<'a> {
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
    pub fn iter(&'a self) -> ArrayU16Iterator<'a> {
        ArrayU16Iterator {
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

            if let Some(scaler) = &self.scaling {
                let type_info = [0b0100_0010, 0b0001_1001, 0b0000_0000, 0b0000_0000];
                let quantization;
                let offset: [u8; 4];

                if is_big_endian {
                    quantization = scaler.quantization.to_be_bytes();
                    offset = scaler.offset.to_be_bytes();
                } else {
                    quantization = scaler.quantization.to_le_bytes();
                    offset = scaler.offset.to_le_bytes();
                }

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
                buf.try_extend_from_slice(&quantization)?;
                buf.try_extend_from_slice(&offset)?;
                buf.try_extend_from_slice(self.data)?;
                Ok(())
            } else {
                let type_info: [u8; 4] = [0b0100_0010, 0b0000_1001, 0b0000_0000, 0b0000_0000];
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
                Ok(())
            }
        } else {
            let number_of_dimensions = if is_big_endian {
                (self.dimensions.dimensions.len() as u16 / 2).to_be_bytes()
            } else {
                (self.dimensions.dimensions.len() as u16 / 2).to_le_bytes()
            };

            if let Some(scaler) = &self.scaling {
                let type_info: [u8; 4] = [0b0100_0010, 0b0001_0001, 0b0000_0000, 0b0000_0000];

                let quantization;
                let offset: [u8; 4];

                if is_big_endian {
                    quantization = scaler.quantization.to_be_bytes();
                    offset = scaler.offset.to_be_bytes();
                } else {
                    quantization = scaler.quantization.to_le_bytes();
                    offset = scaler.offset.to_le_bytes();
                }

                buf.try_extend_from_slice(&type_info)?;
                buf.try_extend_from_slice(&number_of_dimensions)?;
                buf.try_extend_from_slice(self.dimensions.dimensions)?;
                buf.try_extend_from_slice(&quantization)?;
                buf.try_extend_from_slice(&offset)?;
                buf.try_extend_from_slice(self.data)?;
            } else {
                let type_info: [u8; 4] = [0b0100_0010, 0b0000_0001, 0b0000_0000, 0b0000_0000];
                buf.try_extend_from_slice(&type_info)?;
                buf.try_extend_from_slice(&number_of_dimensions)?;
                buf.try_extend_from_slice(self.dimensions.dimensions)?;
                buf.try_extend_from_slice(self.data)?;
            }
            Ok(())
        }
    }
}

#[cfg(feature = "serde")]
impl<'a> Serialize for ArrayU16<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArrayU16", 4)?;
        state.serialize_field("variable_info", &self.variable_info)?;
        state.serialize_field("scaling", &self.scaling)?;

        let iter = ArrayItDimension::<u16> {
            is_big_endian: self.dimensions.is_big_endian,
            dimensions: self.dimensions.dimensions,
            data: self.data,
            phantom: Default::default(),
        };
        state.serialize_field("data", &iter)?;

        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct ArrayU16Iterator<'a> {
    pub(crate) is_big_endian: bool,
    pub(crate) rest: &'a [u8],
}

impl Iterator for ArrayU16Iterator<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.len() < 2 {
            None
        } else {
            let result = if self.is_big_endian {
                u16::from_be_bytes([self.rest[0], self.rest[1]])
            } else {
                u16::from_le_bytes([self.rest[0], self.rest[1]])
            };
            self.rest = &self.rest[2..];
            Some(result)
        }
    }
}

impl<'a> IntoIterator for &'a ArrayU16<'a> {
    type Item = u16;
    type IntoIter = ArrayU16Iterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "serde")]
impl<'a> Serialize for ArrayU16Iterator<'a> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::UnexpectedEndOfSliceError;
    use crate::error::VerboseDecodeError::UnexpectedEndOfSlice;
    use crate::verbose::VerboseValue;
    use crate::verbose::VerboseValue::ArrU16;
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use std::format;
    use std::mem::size_of;

    type TestType<'a> = ArrayU16<'a>;
    type InternalTypes = u16;

    proptest! {
        #[test]
        fn write_read(ref name in "\\pc{0,20}", ref unit in "\\pc{0,20}", quantization in any::<f32>(), offset in any::<i32>(), dim_count in 0u16..5) {
            const TYPE_INFO_RAW: [u8; 4] = [0b0100_0010, 0b0000_0001, 0b0000_0000, 0b0000_0000];
            const VAR_INFO_FLAG: u8 = 0b0000_1000;
            const FIXED_POINT_FLAG: u8 = 0b0001_0000;

            const BUFFER_SIZE: usize = 400;

            // test big endian with name & scaling
            {
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = Some(VariableInfoUnit {
                 name,
                 unit,
             });
             let scaling = Some(Scaling {
                 quantization,
                 offset,
             });

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 0..dim_count {
                 dimensions.extend_from_slice(&(i + 1).to_be_bytes());
                 for x in 0..=i as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes()); // Sample U16s
                 }
             }

             let arr_dim = ArrayDimensions {
                 is_big_endian,
                 dimensions: &dimensions,
             };
             let arr_u = TestType {        is_big_endian,
                        variable_info,
                        dimensions: arr_dim,
                        data: &content,
                        scaling,
                    };
             arr_u.add_to_msg(&mut msg_buff, is_big_endian)?;

             let len_name = (name.len() as u16 + 1).to_be_bytes();
             let len_unit = (unit.len() as u16 + 1).to_be_bytes();

             let mut content_buff = Vec::new();

             content_buff.extend_from_slice(&[
                 TYPE_INFO_RAW[0], TYPE_INFO_RAW[1] | VAR_INFO_FLAG | FIXED_POINT_FLAG, TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]
             ]);
             content_buff.extend_from_slice(&dim_count.to_be_bytes());
             content_buff.extend_from_slice(&dimensions);
             content_buff.extend_from_slice(&[
                 len_name[0],
                 len_name[1],
                 len_unit[0],
                 len_unit[1],
             ]);
             content_buff.extend_from_slice(name.as_bytes());
             content_buff.push(0);
             content_buff.extend_from_slice(unit.as_bytes());
             content_buff.push(0);
             content_buff.extend_from_slice(&quantization.to_be_bytes());
             content_buff.extend_from_slice(&offset.to_be_bytes());
             content_buff.extend_from_slice(&content);

             prop_assert_eq!(&msg_buff[..], &content_buff[..]);

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Ok((ArrU16(arr_u), &[] as &[u8])));
         }

            // test little endian with name & scaling
            {
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = Some(VariableInfoUnit {
                 name,
                 unit,
             });
             let scaling = Some(Scaling {
                 quantization,
                 offset,
             });

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 0..dim_count {
                 dimensions.extend_from_slice(&(i + 1).to_le_bytes());
                 for x in 0..=i as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes()); // Sample U16s
                 }
             }

             let arr_dim = ArrayDimensions {
                 is_big_endian,
                 dimensions: &dimensions,
             };
             let arr_u = TestType {        is_big_endian,
                        variable_info,
                        dimensions: arr_dim,
                        data: &content,
                        scaling,
                    };
             arr_u.add_to_msg(&mut msg_buff, is_big_endian)?;

             let len_name = (name.len() as u16 + 1).to_le_bytes();
             let len_unit = (unit.len() as u16 + 1).to_le_bytes();

             let mut content_buff = Vec::new();

             content_buff.extend_from_slice(&[
                 TYPE_INFO_RAW[0], TYPE_INFO_RAW[1] | VAR_INFO_FLAG | FIXED_POINT_FLAG, TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]
             ]);
             content_buff.extend_from_slice(&dim_count.to_le_bytes());
             content_buff.extend_from_slice(&dimensions);
             content_buff.extend_from_slice(&[
                 len_name[0],
                 len_name[1],
                 len_unit[0],
                 len_unit[1],
             ]);
             content_buff.extend_from_slice(name.as_bytes());
             content_buff.push(0);
             content_buff.extend_from_slice(unit.as_bytes());
             content_buff.push(0);
             content_buff.extend_from_slice(&quantization.to_le_bytes());
             content_buff.extend_from_slice(&offset.to_le_bytes());
             content_buff.extend_from_slice(&content);

             prop_assert_eq!(&msg_buff[..], &content_buff[..]);

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Ok((ArrU16(arr_u), &[] as &[u8])));
         }

         // test big endian with name without scaling
         {
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = Some(VariableInfoUnit { name , unit });
             let scaling = None;

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 0..dim_count {
                     dimensions.extend_from_slice(&(i+1).to_be_bytes());
                 for x in 0..=i as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes());       // Sample U16s
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
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
             prop_assert_eq!(parsed_back, Ok((ArrU16(arr),&[] as &[u8])));

             }

         // test little endian with name without scaling
         {
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = Some(VariableInfoUnit { name , unit });
             let scaling = None;

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 0..dim_count {
                     dimensions.extend_from_slice(&(i+1).to_le_bytes());
                 for x in 0..=i as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes());       // Sample U16s
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr_u = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr_u.add_to_msg(&mut msg_buff, is_big_endian)?;

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
             prop_assert_eq!(parsed_back, Ok((ArrU16(arr_u),&[] as &[u8])));

             }

         // test big endian without name & scaling
         {
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = None;
             let scaling = None;

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 0..dim_count {
                     dimensions.extend_from_slice(&(i+1).to_be_bytes());
                 for x in 0..=i as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes());       // Sample U16s
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;
             let mut content_buff = Vec::new();

             content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1], TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
             content_buff.extend_from_slice(&dim_count.to_be_bytes());
             content_buff.extend_from_slice(&dimensions);
             content_buff.extend_from_slice(&content);

             prop_assert_eq!(&msg_buff[..], &content_buff[..]);

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Ok((ArrU16(arr),&[] as &[u8])));

             }

         // test little endian without name & scaling
         {
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = None;
             let scaling = None;

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 0..dim_count {
                     dimensions.extend_from_slice(&(i+1).to_le_bytes());
                 for x in 0..=i as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes());       // Sample U16s
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;

             let mut content_buff = Vec::new();
             content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1], TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
             content_buff.extend_from_slice(&dim_count.to_le_bytes());
             content_buff.extend_from_slice(&dimensions);
             content_buff.extend_from_slice(&content);

             prop_assert_eq!(&msg_buff[..], &content_buff[..]);

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Ok((ArrU16(arr),&[] as &[u8])));

             }

          // test big endian with scaling and without name
          {
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = None;
             let scaling = Some(Scaling { quantization, offset });

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 0..dim_count {
                     dimensions.extend_from_slice(&(i+1).to_be_bytes());
                 for x in 0..=i as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes());       // Sample U16s
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;
             let mut content_buff = Vec::new();

             content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1] | FIXED_POINT_FLAG, TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
             content_buff.extend_from_slice(&dim_count.to_be_bytes());
             content_buff.extend_from_slice(&dimensions);
             content_buff.extend_from_slice(&quantization.to_be_bytes());
             content_buff.extend_from_slice(&offset.to_be_bytes());
             content_buff.extend_from_slice(&content);

             prop_assert_eq!(&msg_buff[..], &content_buff[..]);

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Ok((ArrU16(arr),&[] as &[u8])));

             }

         // test little endian with scaling and without name
         {
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = None;
             let scaling = Some(Scaling { quantization, offset });

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 0..dim_count {
                     dimensions.extend_from_slice(&(i+1).to_le_bytes());
                 for x in 0..=i as InternalTypes {
                     content.extend_from_slice(&x.to_le_bytes());       // Sample U16s
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;
             let mut content_buff = Vec::new();

             content_buff.extend_from_slice(&[TYPE_INFO_RAW[0], TYPE_INFO_RAW[1] | FIXED_POINT_FLAG, TYPE_INFO_RAW[2], TYPE_INFO_RAW[3]]);
             content_buff.extend_from_slice(&dim_count.to_le_bytes());
             content_buff.extend_from_slice(&dimensions);
             content_buff.extend_from_slice(&quantization.to_le_bytes());
             content_buff.extend_from_slice(&offset.to_le_bytes());
             content_buff.extend_from_slice(&content);

             prop_assert_eq!(&msg_buff[..], &content_buff[..]);

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Ok((ArrU16(arr),&[] as &[u8])));

             }

          // Capacity error big endian with name & scaling
          {
             let dim_count = dim_count + 1;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = Some(VariableInfoUnit { name , unit });
             let scaling = Some(Scaling { quantization, offset });

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             // for i in 1u16..=dim_count {
             //     dimensions.extend_from_slice(&(i as u16).to_be_bytes());

             //     for x in 0..(i-1) as InternalTypes {
             //         content.extend_from_slice(&x.to_be_bytes());
             //     }
             // }

             for i in 0..dim_count {
                 dimensions.extend_from_slice(&(i+1).to_be_bytes());
             for x in 0..i as InternalTypes {
                 content.extend_from_slice(&x.to_be_bytes());       // Sample U16s
             }
         }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;
             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));

         }

          // Capacity error little endian with name & scaling
          {
             let dim_count = dim_count + 1;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = Some(VariableInfoUnit { name , unit });
             let scaling = Some( Scaling { quantization, offset });

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 1u16..=dim_count {
                 dimensions.extend_from_slice(&(i as u16).to_le_bytes());

                 for x in 0..(i-1) as InternalTypes {
                     content.extend_from_slice(&x.to_le_bytes());
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));
         }

          // Capacity error big endian with name & no scaling
          {
             let dim_count = dim_count + 1;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = Some(VariableInfoUnit { name , unit });
             let scaling = None;

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 1u16..=dim_count {
                 dimensions.extend_from_slice(&(i as u16).to_be_bytes());

                 for x in 0..(i-1) as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes());
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;


             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));

         }

          // Capacity error little endian with name & no scaling
          {
             let dim_count = dim_count + 1;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = Some(VariableInfoUnit { name , unit });
             let scaling = None;

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 1u16..=dim_count {
                 dimensions.extend_from_slice(&(i as u16).to_le_bytes());

                 for x in 0..(i-1) as InternalTypes {
                     content.extend_from_slice(&x.to_le_bytes());
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));
         }

          // Capacity error big endian with scaling & no name
          {
             let dim_count = dim_count + 1;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = None;
             let scaling = Some(Scaling { quantization, offset });

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 1u16..=dim_count {
                 dimensions.extend_from_slice(&(i as u16).to_be_bytes());

                 for x in 0..(i-1) as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes());
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));

         }

          // Capacity error little endian with scaling & no name
          {
             let dim_count = dim_count + 1;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = None;
             let scaling = Some( Scaling { quantization, offset });

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 1u16..=dim_count {
                 dimensions.extend_from_slice(&(i as u16).to_le_bytes());

                 for x in 0..(i-1) as InternalTypes {
                     content.extend_from_slice(&x.to_le_bytes());
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));
         }

          // Capacity error big endian without scaling & name
          {
             let dim_count = dim_count + 1;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = None;
             let scaling = None;

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 1u16..=dim_count {
                 dimensions.extend_from_slice(&(i as u16).to_be_bytes());

                 for x in 0..(i-1) as InternalTypes {
                     content.extend_from_slice(&x.to_be_bytes());
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));

         }

          // Capacity error little endian without scaling & name
          {
             let dim_count = dim_count + 1;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = None;
             let scaling = None;

             let mut dimensions = Vec::with_capacity(dim_count as usize);
             let mut content = Vec::with_capacity(dim_count as usize);

             for i in 1u16..=dim_count {
                 dimensions.extend_from_slice(&(i as u16).to_le_bytes());

                 for x in 0..(i-1) as InternalTypes {
                     content.extend_from_slice(&x.to_le_bytes());
                 }
             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             arr.add_to_msg(&mut msg_buff, is_big_endian)?;

             // Now wrap back
             let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
             prop_assert_eq!(parsed_back, Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError { layer: crate::error::Layer::VerboseValue, minimum_size: msg_buff.len() + size_of::<InternalTypes>() * dim_count as usize, actual_size: msg_buff.len() })));
         }


          // capacity error big endian with name, without scaling
          {
             let name = "Abc";
             let unit = "Xyz";
             const DIM_COUNT: u16 = 5;
             const BUFFER_SIZE: usize = DIM_COUNT as usize * 2 + 14;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = true;

             let variable_info = Some(VariableInfoUnit { name , unit });
             let scaling = None;

             let mut dimensions = Vec::with_capacity(DIM_COUNT as usize);
             let mut content = Vec::with_capacity(DIM_COUNT as usize);

             for i in 0..DIM_COUNT as InternalTypes {
                     dimensions.extend_from_slice(&(1 as InternalTypes).to_be_bytes());
                     content.extend_from_slice(&i.to_be_bytes());       // Sample values

             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             let err = arr.add_to_msg(&mut msg_buff, is_big_endian);

             prop_assert_eq!(err, Err(CapacityError::new(())));

             }


          // capacity error little endian with name, without scaling
          {
             let name = "Abc";
             let unit = "Xyz";
             const DIM_COUNT: u16 = 5;
             const BUFFER_SIZE: usize = DIM_COUNT as usize * 2 + 15;
             let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
             let is_big_endian = false;

             let variable_info = Some(VariableInfoUnit { name , unit });
             let scaling = Some(Scaling { quantization, offset });

             let mut dimensions = Vec::with_capacity(DIM_COUNT as usize);
             let mut content = Vec::with_capacity(DIM_COUNT as usize);

             for i in 0..DIM_COUNT as InternalTypes {
                     dimensions.extend_from_slice(&(1 as u16).to_le_bytes());
                     content.extend_from_slice(&i.to_le_bytes());       // Sample values

             }

             let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };

             let arr = TestType { is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };
             let err = arr.add_to_msg(&mut msg_buff, is_big_endian);

             prop_assert_eq!(err, Err(CapacityError::new(())));

             }
        }
    }

    proptest! {
        #[test]
        fn data(ref data in "\\pc{0,100}", ref dimensions in "\\pc{0,100}", quantization in any::<f32>(), offset in any::<i32>()) {

            let arr_dim = ArrayDimensions {dimensions: dimensions.as_bytes(), is_big_endian: true };
            let scaling = Some(Scaling { quantization, offset });
            let arr = ArrayU16 {is_big_endian: true, dimensions:arr_dim,variable_info:None,data:data.as_bytes(), scaling };
            prop_assert_eq!(arr.data(), data.as_bytes());
        }
    }

    proptest! {
        #[test]
        fn iterator(dim_count in 0u16..5) {

            // test big endian without name & scaling
            {
                let is_big_endian = true;

                let variable_info = None;
                let scaling = None;

                let mut dimensions = Vec::with_capacity(dim_count as usize);
                let mut content = Vec::with_capacity(dim_count as usize);

                for i in 0..dim_count {
                        dimensions.extend_from_slice(&(i+1).to_be_bytes());
                    for x in 0..=i as InternalTypes {
                            content.extend_from_slice(&x.to_be_bytes());
                    }
                }

                let arr_dim = ArrayDimensions { is_big_endian, dimensions: &dimensions };
                let arr = TestType {is_big_endian, variable_info, dimensions:arr_dim,data: &content, scaling };

                let mut cnt = 0;
                for item in arr.iter() {
                    prop_assert_eq!(item, InternalTypes::from_be_bytes([content[cnt], content[cnt+1]]));
                    cnt += size_of::<InternalTypes>();
                }

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
                scaling: None,
                is_big_endian,
            };

            let convert_content =
                "{\"variable_info\":null,\"scaling\":null,\"data\":[]}".to_string();

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
                scaling: None,
                is_big_endian,
            };

            let convert_content =
                "{\"variable_info\":null,\"scaling\":null,\"data\":[0]}".to_string();

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
                scaling: None,
                is_big_endian,
            };

            let convert_content =
                "{\"variable_info\":null,\"scaling\":null,\"data\":[[0,1]]}".to_string();

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
                scaling: None,
                is_big_endian,
            };

            let convert_content =
                "{\"variable_info\":null,\"scaling\":null,\"data\":[[[0,1,2],[3,4,5]]]}"
                    .to_string();
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
                scaling: None,
                is_big_endian,
            };

            let convert_content = "{\"variable_info\":null,\"scaling\":null,\"data\":[[[[0,1,2,3],[4,5,6,7],[8,9,10,11]],[[12,13,14,15],[16,17,18,19],[20,21,22,23]]]]}".to_string();
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
                scaling: None,
                is_big_endian,
            };

            let convert_content = "{\"variable_info\":null,\"scaling\":null,\"data\":[[[[[0,1,2,3,4],[5,6,7,8,9],[10,11,12,13,14],[15,16,17,18,19]],[[20,21,22,23,24],[25,26,27,28,29],[30,31,32,33,34],[35,36,37,38,39]],[[40,41,42,43,44],[45,46,47,48,49],[50,51,52,53,54],[55,56,57,58,59]]],[[[60,61,62,63,64],[65,66,67,68,69],[70,71,72,73,74],[75,76,77,78,79]],[[80,81,82,83,84],[85,86,87,88,89],[90,91,92,93,94],[95,96,97,98,99]],[[100,101,102,103,104],[105,106,107,108,109],[110,111,112,113,114],[115,116,117,118,119]]]]]}".to_string();
            assert_eq!(convert_content, serde_json::to_string(&arr).unwrap());
        }
    }
}
