use crate::{
    verbose::VerboseIter, DltControlMessageType, DltLogLevel, DltNetworkType, DltTraceType,
};

/// Typed payload of a DLT log message based on the message info in the DLT
/// extended header.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DltTypedPayload<'a> {
    /// A non verbose message of unknown type.
    ///
    /// This type is used if the DLT message did not contain an extended
    /// header and the type of message can only be determined via the
    /// message id. In this the type of th message can only be determined
    /// based on the message id and an additional information source describing
    /// how to decode the message payloads and what type of message it is
    /// (e.g. a Fibex file).
    UnknownNv(NvPayload<'a>),

    /// Non verbose log message (does not contain a description of it's contents).
    ///
    /// Non verbose log messages cannot be decoded without additional
    /// additional informations how the message payloads can be decoded
    /// (e.g. a Fibex file).
    LogNv(LogNvPayload<'a>),

    /// Verbose log message (contain a description of it's contents).
    ///
    /// Verbose log messages can be decoded without additional
    /// informations.
    LogV(LogVPayload<'a>),

    /// Non verbose trace message.
    TraceNv(TraceNvPayload<'a>),

    /// Verbose trace message.
    TraceV(TraceVPayload<'a>),

    /// Non verbose network message.
    NetworkNv(NetworkNvPayload<'a>),

    /// Verbose network message.
    NetworkV(NetworkVPayload<'a>),

    /// Non verbose control message.
    ControlNv(ControlNvPayload<'a>),

    /// Verbose control message.
    ControlV(ControlVPayload<'a>),
}

/// A non verbose message of unknown type.
///
/// This type is used if the DLT message did not contain an extended
/// header and the type of message can only be determined via the
/// message id. In this the type of th message can only be determined
/// based on the message id and an additional information source describing
/// how to decode the message payloads and what type of message it is
/// (e.g. a Fibex file).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NvPayload<'a> {
    pub msg_id: u32,
    pub payload: &'a [u8],
}

impl<'a> From<LogNvPayload<'a>> for NvPayload<'a> {
    fn from(value: LogNvPayload<'a>) -> Self {
        NvPayload {
            msg_id: value.msg_id,
            payload: value.payload,
        }
    }
}

impl<'a> From<TraceNvPayload<'a>> for NvPayload<'a> {
    fn from(value: TraceNvPayload<'a>) -> Self {
        NvPayload {
            msg_id: value.msg_id,
            payload: value.payload,
        }
    }
}

impl<'a> From<NetworkNvPayload<'a>> for NvPayload<'a> {
    fn from(value: NetworkNvPayload<'a>) -> Self {
        NvPayload {
            msg_id: value.msg_id,
            payload: value.payload,
        }
    }
}

/// Non verbose log message (does not contain a description of it's contents).
///
/// Non verbose log messages cannot be decoded without additional
/// additional informations how the message payloads can be decoded
/// (e.g. a Fibex file).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LogNvPayload<'a> {
    pub log_level: DltLogLevel,
    pub msg_id: u32,
    pub payload: &'a [u8],
}

/// Verbose log message (contain a description of it's contents).
///
/// Verbose log messages can be decoded without additional
/// informations.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LogVPayload<'a> {
    pub log_level: DltLogLevel,
    pub iter: VerboseIter<'a>,
}

/// Non verbose trace message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraceNvPayload<'a> {
    pub trace_type: DltTraceType,
    pub msg_id: u32,
    pub payload: &'a [u8],
}

/// Verbose trace message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraceVPayload<'a> {
    pub trace_type: DltTraceType,
    pub iter: VerboseIter<'a>,
}

/// Non verbose network message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NetworkNvPayload<'a> {
    pub net_type: DltNetworkType,
    pub msg_id: u32,
    pub payload: &'a [u8],
}

/// Verbose network message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NetworkVPayload<'a> {
    pub net_type: DltNetworkType,
    pub iter: VerboseIter<'a>,
}

/// Non verbose control message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ControlNvPayload<'a> {
    pub msg_type: DltControlMessageType,
    pub service_id: u32,
    pub payload: &'a [u8],
}

/// Verbose control message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ControlVPayload<'a> {
    pub msg_type: DltControlMessageType,
    pub iter: VerboseIter<'a>,
}
