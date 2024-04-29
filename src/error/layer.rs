/// Error in which an error occured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layer {
    /// Error occured while parsing or writing the DLT header.
    DltHeader,
    /// Error occured while parsing or writing a verbose type info.
    VerboseTypeInfo,
    /// Error occured while parsing or writing a verbose value.
    VerboseValue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn clone_eq() {
        use Layer::*;
        assert_eq!(VerboseTypeInfo, VerboseTypeInfo.clone());
    }

    #[test]
    fn debug() {
        use Layer::*;
        assert_eq!("VerboseTypeInfo", format!("{:?}", VerboseTypeInfo));
    }
}
