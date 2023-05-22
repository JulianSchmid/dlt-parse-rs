#[cfg(feature = "serde")]
use super::{
    ArrayBoolIterator, ArrayF128Iterator, ArrayF16Iterator, ArrayF32Iterator, ArrayF64Iterator,
    ArrayI128Iterator, ArrayI16Iterator, ArrayI32Iterator, ArrayI64Iterator, ArrayI8Iterator,
    ArrayU128Iterator, ArrayU16Iterator, ArrayU32Iterator, ArrayU64Iterator, ArrayU8Iterator,
};
use super::{F128, F16};
#[cfg(feature = "serde")]
use crate::verbose::std::mem::size_of;
#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeSeq, Serializer};

#[cfg(feature = "serde")]
#[derive(Clone, Debug)]
pub(crate) struct ArrayItDimension<'a, T: ArrayIteratable + Sized> {
    pub(crate) is_big_endian: bool,
    pub(crate) dimensions: &'a [u8],
    pub(crate) data: &'a [u8],
    pub(crate) phantom: core::marker::PhantomData<T>,
}

#[cfg(feature = "serde")]
impl<'a, T: ArrayIteratable + Sized> Serialize for ArrayItDimension<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.dimensions.len() > 2 {
            // calculate memory step size
            let mut stepsize: usize = 1;
            for i in (2..self.dimensions.len()).step_by(2) {
                let bytes = [self.dimensions[i], self.dimensions[i + 1]];
                stepsize *= usize::from(if self.is_big_endian {
                    u16::from_be_bytes(bytes)
                } else {
                    u16::from_le_bytes(bytes)
                });
            }

            // determine own dim size & the subdimensions
            let sub_dimensions = &self.dimensions[2..];
            let dim_count: usize = {
                let bytes = [self.dimensions[0], self.dimensions[1]];
                if self.is_big_endian {
                    u16::from_be_bytes(bytes)
                } else {
                    u16::from_le_bytes(bytes)
                }
            }
            .into();
            // iterate over blocks
            let mut seq = serializer.serialize_seq(Some(dim_count.into()))?;
            for i in 0..dim_count {
                // serialize subdimensions
                let block_start = i * stepsize * size_of::<T>();
                let block_end = (i + 1) * stepsize * size_of::<T>();

                let subit = ArrayItDimension::<'a, T> {
                    is_big_endian: self.is_big_endian,
                    dimensions: sub_dimensions,
                    data: &self.data[block_start..block_end],
                    phantom: Default::default(),
                };
                seq.serialize_element(&subit)?;
            }
            seq.end()
        } else if self.dimensions.len() == 2 {
            T::serialize_elements(self.is_big_endian, self.data, serializer)
        } else {
            let seq = serializer.serialize_seq(Some(0))?;
            seq.end()
        }
    }
}

#[cfg(feature = "serde")]
pub(crate) trait ArrayIteratable {
    const ELEMENT_SIZE: usize;

    fn serialize_elements<S: Serializer>(
        is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}

#[cfg(feature = "serde")]
impl ArrayIteratable for bool {
    const ELEMENT_SIZE: usize = 1;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayBoolIterator { rest: data }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for u8 {
    const ELEMENT_SIZE: usize = 1;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayU8Iterator { rest: data }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for u16 {
    const ELEMENT_SIZE: usize = 2;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayU16Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for u32 {
    const ELEMENT_SIZE: usize = 4;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayU32Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for u64 {
    const ELEMENT_SIZE: usize = 8;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayU64Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for u128 {
    const ELEMENT_SIZE: usize = 16;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayU128Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for i8 {
    const ELEMENT_SIZE: usize = 1;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayI8Iterator { rest: data }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for i16 {
    const ELEMENT_SIZE: usize = 2;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayI16Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for i32 {
    const ELEMENT_SIZE: usize = 4;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayI32Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for i64 {
    const ELEMENT_SIZE: usize = 8;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayI64Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for i128 {
    const ELEMENT_SIZE: usize = 16;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayI128Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for F16 {
    const ELEMENT_SIZE: usize = 2;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayF16Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for f32 {
    const ELEMENT_SIZE: usize = 4;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayF32Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for f64 {
    const ELEMENT_SIZE: usize = 8;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayF64Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl ArrayIteratable for F128 {
    const ELEMENT_SIZE: usize = 16;

    fn serialize_elements<S: Serializer>(
        _is_big_endian: bool,
        data: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ArrayF128Iterator {
            rest: data,
            is_big_endian: _is_big_endian,
        }
        .serialize(serializer)
    }
}
