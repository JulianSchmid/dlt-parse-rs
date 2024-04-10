use crate::{verbose::VerboseIter, ControlMessage, DltMessageInfo};

/// Payload of a DLT log message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DltTypedPayload<'a> {
    /// Generic non verbose message without message type infos.
    GenericNv(GenericNvPayload<'a>),
    /// Verbose log message.
    LogV(LogVPayload<'a>),
    /// Non verbose log message.
    LogNv(LogNvPayload<'a>),
    /// Verbose trace message.
    TraceV(TraceVPayload<'a>),
    /// Non verbose trace message.
    TraceNv(TraceNvPayload<'a>),
    /// Verbose network message.
    NetworkV(NetworkVPayload<'a>),
    /// Non verbose network message.
    NetworkNv(NetworkNvPayload<'a>),
    /// Verbose control message.
    ControlV(ControlVPayload<'a>),
    /// Non verbose control message.
    ControlNv(ControlNvPayload<'a>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LogVPayload<'a> {
    pub info: DltMessageInfo,
    pub iter: VerboseIter<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraceVPayload<'a> {
    pub info: DltMessageInfo,
    pub iter: VerboseIter<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NetworkVPayload<'a> {
    pub info: DltMessageInfo,
    pub iter: VerboseIter<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ControlVPayload<'a> {
    pub info: DltMessageInfo,
    pub iter: VerboseIter<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraceNvPayload<'a> {
    pub info: Option<DltMessageInfo>,
    pub msg_id: u32,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GenericNvPayload<'a> {
    pub info: Option<DltMessageInfo>,
    pub msg_id: u32,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NetworkNvPayload<'a> {
    pub info: Option<DltMessageInfo>,
    pub msg_id: u32,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LogNvPayload<'a> {
    pub info: Option<DltMessageInfo>,
    pub msg_id: u32,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ControlNvPayload<'a> {
    pub info: Option<DltMessageInfo>,
    pub msg_id: u32,
    pub payload: &'a [u8],
    pub control_message: Option<ControlMessage<'a>>,
}
