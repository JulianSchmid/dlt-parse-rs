/// A 128 bit floating point number stored in "raw".
///
/// This is needed as Rust does not support (and most systems)
/// don't support 128 bit floating point values.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RawF128(u128);

impl RawF128 {
    /// Create a floating point value from its representation as a
    /// byte array in big endian.
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 16]) -> RawF128 {
        RawF128(u128::from_be_bytes(bytes))
    }

    /// Create a floating point value from its representation as a
    /// byte array in little endian.
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 16]) -> RawF128 {
        RawF128(u128::from_le_bytes(bytes))
    }

    /// Create a floating point value from its representation as a byte
    /// array in native endian.
    #[inline]
    pub const fn from_ne_bytes(bytes: [u8; 16]) -> RawF128 {
        RawF128(u128::from_ne_bytes(bytes))
    }

    /// Return the memory representation of this floating point number
    /// as a byte array in big-endian (network) byte order.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    /// Return the memory representation of this floating point number
    /// as a byte array in little-endian byte order
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 16] {
        self.0.to_le_bytes()
    }

    /// Return the memory representation of this floating point number as
    /// a byte array in native byte order.
    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; 16] {
        self.0.to_ne_bytes()
    }

    /// Raw transmutation from `u128`.
    #[inline]
    pub const fn from_bits(bits: u128) -> RawF128 {
        RawF128(bits)
    }

    /// Raw transmutation to `u1128`.
    #[inline]
    pub const fn to_bits(self) -> u128 {
        self.0
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RawF128 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u128(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn debug_clone_eq(value in any::<u128>()) {
            use alloc::format;

            assert_eq!(
                format!("{:?}", RawF128(value)),
                format!("{:?}", RawF128(value))
            );
            assert_eq!(RawF128(value), RawF128(value).clone());
        }
    }

    proptest! {
        #[test]
        fn from_be_bytes(value in any::<u128>()) {
            assert_eq!(
                value,
                RawF128::from_be_bytes(value.to_be_bytes()).0
            );
        }
    }

    proptest! {
        #[test]
        fn from_le_bytes(value in any::<u128>()) {
            assert_eq!(
                value,
                RawF128::from_le_bytes(value.to_le_bytes()).0
            );
        }
    }

    proptest! {
        #[test]
        fn from_ne_bytes(value in any::<u128>()) {
            assert_eq!(
                value,
                RawF128::from_ne_bytes(value.to_ne_bytes()).0
            );
        }
    }

    proptest! {
        #[test]
        fn to_be_bytes(value in any::<u128>()) {
            assert_eq!(
                value.to_be_bytes(),
                RawF128(value).to_be_bytes()
            );
        }
    }

    proptest! {
        #[test]
        fn to_le_bytes(value in any::<u128>()) {
            assert_eq!(
                value.to_le_bytes(),
                RawF128(value).to_le_bytes()
            );
        }
    }

    proptest! {
        #[test]
        fn to_ne_bytes(value in any::<u128>()) {
            assert_eq!(
                value.to_ne_bytes(),
                RawF128(value).to_ne_bytes()
            );
        }
    }

    proptest! {
        #[test]
        fn from_bits(value in any::<u128>()) {
            assert_eq!(
                value,
                RawF128::from_bits(value).0
            );
        }
    }

    proptest! {
        #[test]
        fn to_bits(value in any::<u128>()) {
            assert_eq!(
                value,
                RawF128(value).to_bits()
            );
        }
    }

    #[cfg(feature = "serde")]
    proptest! {
        #[test]
        fn serialize(value in any::<u128>()) {
            let v = RawF128(value);
            assert_eq!(
                serde_json::to_string(&v.to_bits()).unwrap(),
                serde_json::to_string(&v).unwrap()
            );
        }
    }
}
