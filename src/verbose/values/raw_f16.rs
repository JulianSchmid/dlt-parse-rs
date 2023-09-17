/// A 16 bit floating point number stored in "raw".
///
/// This is needed as Rust does not support (and most systems)
/// don't support 16 bit floating point values.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RawF16(u16);

impl RawF16 {
    const SIGN_MASK: u16 = 0b1000_0000_0000_0000;
    const EXPO_MASK: u16 = 0b0111_1100_0000_0000;
    const FRAC_MASK: u16 = 0b0000_0011_1111_1111;

    pub const ZERO: RawF16 = RawF16::from_bits(0);
    pub const ONE: RawF16 = RawF16::from_bits(0b0011_1100_0000_0000);
    pub const NAN: RawF16 = RawF16::from_bits(0b0111_1100_0000_0001);
    pub const INFINITY: RawF16 = RawF16::from_bits(0b0111_1100_0000_0000);
    pub const NEGATIVE_INFINITY: RawF16 = RawF16::from_bits(0b1111_1100_0000_0000);

    /// Converts the f16 to a f32.
    #[inline]
    pub fn to_f32(self) -> f32 {
        let raw_u16 = self.0;
        // extract elements & re-shift to f32
        //
        // f16
        //   * 10 bits fraction
        //   * 5 bits exponent
        //   * 1 bit sign bit
        // f32
        //   * 23 bits fraction
        //   * 8 bits exponent
        //   * 1 bit sign bit
        let sign = ((raw_u16 & RawF16::SIGN_MASK) as u32) << (31 - 15);
        let masked_expo = raw_u16 & RawF16::EXPO_MASK;
        let expo = if RawF16::EXPO_MASK == masked_expo {
            // max has to be handled specially (as it represents infinity or NaN)
            0b0111_1111_1000_0000__0000_0000_0000_0000
        } else if masked_expo == 0 {
            0
        } else {
            // to get the to the exponent substract 0b01111
            let decoded_expo = i32::from((raw_u16 & RawF16::EXPO_MASK) >> 10) - 0b01111;
            // and to get to the 32 bit encoding 0x7f needs to be added
            ((decoded_expo + 0x7F) as u32) << 23
        };
        // shift by 13 as the bits start from the highest to the lowest
        let frac = ((raw_u16 & RawF16::FRAC_MASK) as u32) << 13;

        // recompose to u32
        f32::from_bits(sign | expo | frac)
    }

    /// Create a floating point value from its representation as a
    /// byte array in big endian.
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 2]) -> RawF16 {
        RawF16(u16::from_be_bytes(bytes))
    }

    /// Create a floating point value from its representation as a
    /// byte array in little endian.
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 2]) -> RawF16 {
        RawF16(u16::from_le_bytes(bytes))
    }

    /// Create a floating point value from its representation as a byte
    /// array in native endian.
    #[inline]
    pub const fn from_ne_bytes(bytes: [u8; 2]) -> RawF16 {
        RawF16(u16::from_ne_bytes(bytes))
    }

    /// Return the memory representation of this floating point number
    /// as a byte array in big-endian (network) byte order.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }

    /// Return the memory representation of this floating point number
    /// as a byte array in little-endian byte order
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 2] {
        self.0.to_le_bytes()
    }

    /// Return the memory representation of this floating point number as
    /// a byte array in native byte order.
    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; 2] {
        self.0.to_ne_bytes()
    }

    /// Raw transmutation from `u16`.
    #[inline]
    pub const fn from_bits(bits: u16) -> RawF16 {
        RawF16(bits)
    }

    /// Raw transmutation to `u16`.
    #[inline]
    pub const fn to_bits(self) -> u16 {
        self.0
    }
}

impl From<RawF16> for f32 {
    #[inline]
    fn from(value: RawF16) -> Self {
        value.to_f32()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RawF16 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(self.to_f32())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn debug_clone_eq(value in any::<u16>()) {
            use alloc::format;

            assert_eq!(
                format!("{:?}", RawF16(value)),
                format!("{:?}", RawF16(value))
            );
            assert_eq!(RawF16(value), RawF16(value).clone());
        }
    }

    #[test]
    fn constant() {
        assert_eq!(0.0, RawF16::ZERO.to_f32());
        assert_eq!(1.0, RawF16::ONE.to_f32());
        assert!(RawF16::NAN.to_f32().is_nan());
        assert!(RawF16::INFINITY.to_f32().is_infinite());
        assert!(RawF16::NEGATIVE_INFINITY.to_f32().is_infinite());
    }

    #[test]
    fn to_f32() {
        // zero
        assert_eq!(0.0, RawF16(0).to_f32());

        // one
        assert_eq!(1.0, RawF16::from_bits(0b0_01111_0000000000).to_f32());

        // infinite
        assert!(RawF16::from_bits(0b0111_1100_0000_0000)
            .to_f32()
            .is_infinite());

        // nan
        assert!(RawF16::from_bits(0b0111_1100_0000_0001).to_f32().is_nan());

        // largest normal number
        assert_eq!(65504.0, RawF16::from_bits(0b0111_1011_1111_1111).to_f32());
        assert_eq!(-65504.0, RawF16::from_bits(0b1111_1011_1111_1111).to_f32());
    }

    proptest! {
        #[test]
        fn from_be_bytes(value in any::<u16>()) {
            assert_eq!(
                value,
                RawF16::from_be_bytes(value.to_be_bytes()).0
            );
        }
    }

    proptest! {
        #[test]
        fn from_le_bytes(value in any::<u16>()) {
            assert_eq!(
                value,
                RawF16::from_le_bytes(value.to_le_bytes()).0
            );
        }
    }

    proptest! {
        #[test]
        fn from_ne_bytes(value in any::<u16>()) {
            assert_eq!(
                value,
                RawF16::from_ne_bytes(value.to_ne_bytes()).0
            );
        }
    }

    proptest! {
        #[test]
        fn to_be_bytes(value in any::<u16>()) {
            assert_eq!(
                value.to_be_bytes(),
                RawF16(value).to_be_bytes()
            );
        }
    }

    proptest! {
        #[test]
        fn to_le_bytes(value in any::<u16>()) {
            assert_eq!(
                value.to_le_bytes(),
                RawF16(value).to_le_bytes()
            );
        }
    }

    proptest! {
        #[test]
        fn to_ne_bytes(value in any::<u16>()) {
            assert_eq!(
                value.to_ne_bytes(),
                RawF16(value).to_ne_bytes()
            );
        }
    }

    proptest! {
        #[test]
        fn from_bits(value in any::<u16>()) {
            assert_eq!(
                value,
                RawF16::from_bits(value).0
            );
        }
    }

    proptest! {
        #[test]
        fn to_bits(value in any::<u16>()) {
            assert_eq!(
                value,
                RawF16(value).to_bits()
            );
        }
    }

    proptest! {
        #[test]
        fn from_f16_to_f32(value in any::<u16>()) {
            let v = RawF16(value);
            let actual: f32 = v.into();
            if actual.is_nan() {
                assert!(v.to_f32().is_nan());
            } else {
                assert_eq!(actual, v.to_f32());
            }
        }
    }

    #[cfg(feature = "serde")]
    proptest! {
        #[test]
        fn serialize(value in any::<u16>()) {
            let v = RawF16(value);
            assert_eq!(
                serde_json::to_string(&v.to_f32()).unwrap(),
                serde_json::to_string(&v).unwrap()
            );
        }
    }
}
