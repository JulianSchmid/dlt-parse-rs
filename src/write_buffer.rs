
/// Buffer that can be used to store a message while it is written out
/// (non_std replacement for ).
///
/// This trait is used to define a buffer where a DLT message gets
/// serialized to.
/// 
/// # Notes on "Why not just std::io::Write"
/// 
/// 
pub trait WriteBuffer<ErrorT: Debug + Error + Display> {
    type Error;

    pub fn write(data: &[u8]) -> Result<(), Self::Error>;
}
