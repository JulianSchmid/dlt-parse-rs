/// A 16 bit floating point number stored in "raw".
/// 
/// This is needed as Rust does not support (and most systems)
/// don't support 16 bit floating point values. 
#[derive(Copy, Clone, Debug)]
pub struct RawF16(pub [u8;2]);

impl RawF16 {

    const SIGN_MASK: u16 = 0b1000_0000_0000_0000;
    const EXPO_MASK: u16 = 0b0111_1100_0000_0000;
    const FRAC_MASK: u16 = 0b0000_0011_1111_1111;

    /// Converts the f16 to a f32.
    #[inline]
    pub fn to_f32(self) -> f32 {
        let raw_u16 = u16::from_ne_bytes(self.0);
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
        let expo = ((raw_u16 & RawF16::EXPO_MASK) as u32) << (22 - 10);
        let frac = if RawF16::FRAC_MASK == raw_u16 & RawF16::FRAC_MASK {
            // max has to be handled specially (as it represents infinity or NaN)
            0b0111_1111_1000_0000__0000_0000_0000_0000
        } else {
            (raw_u16 & RawF16::FRAC_MASK) as u32
        };

        // recompose to u32
        f32::from_bits(sign | expo | frac)
    }

    /// Create a floating point value from its representation as a
    /// byte array in big endian.
    #[inline]
    pub const fn from_be_bytes(bytes: [u8;2]) -> RawF16 {
        RawF16(u16::from_be_bytes(bytes).to_ne_bytes())
    }

    /// Create a floating point value from its representation as a
    /// byte array in little endian.
    #[inline]
    pub const fn from_le_bytes(bytes: [u8;2]) -> RawF16 {
        RawF16(u16::from_le_bytes(bytes).to_ne_bytes())
    }

    /// Create a floating point value from its representation as a byte
    /// array in native endian.
    #[inline]
    pub const fn from_ne_bytes(bytes: [u8;2]) -> RawF16 {
        RawF16(bytes)
    }

    /// Return the memory representation of this floating point number
    /// as a byte array in big-endian (network) byte order.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8;2] {
        u16::from_ne_bytes(self.0).to_be_bytes()
    }

    /// Return the memory representation of this floating point number
    /// as a byte array in little-endian byte order
    #[inline]
    pub const fn to_le_bytes(self) -> [u8;2] {
        u16::from_ne_bytes(self.0).to_le_bytes()
    }

    /// Return the memory representation of this floating point number as
    /// a byte array in native byte order.
    #[inline]
    pub const fn to_ne_bytes(self) -> [u8;2] {
        self.0
    }

    /// Raw transmutation to u16.
    #[inline]
    pub const fn to_bits(self) -> u16 {
        u16::from_ne_bytes(self.0)
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
