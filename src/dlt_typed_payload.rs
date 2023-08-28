use crate::{verbose::VerboseIter, DltMessageInfo};

/// Payload of a DLT log message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DltTypedPayload<'a> {
    /// Verbose DLT message payload.
    Verbose(DltMessageInfo, VerboseIter<'a>),
    /// Non-verbose DLT message info, message id and payload.
    NonVerbose(Option<DltMessageInfo>, u32, &'a [u8]),
}
