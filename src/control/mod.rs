
const SET_LOG_LEVEL: &str = "set_log_level";
const SET_TRACE_STATUS: &str = "set_trace_status";
const GET_LOG_INFO: &str = "get_log_info";
const GET_DEFAULT_LOG_LEVEL: &str = "get_default_log_level";
const STORE_CONFIGURATION: &str = "store_configuration";
const RESET_TO_FACTORY_DEFAULT: &str = "reset_to_factory_default";
const SET_MESSAGE_FILTERING: &str = "set_message_filtering";
const SET_DEFAULT_LOG_LEVEL: &str = "set_default_log_level";
const SET_DEFAULT_TRACE_STATUS: &str = "set_default_trace_status";
const GET_SOFTWARE_VERSION: &str = "get_software_version";
const GET_DEFAULT_TRACE_STATUS: &str = "get_default_trace_status";
const GET_LOG_CHANNEL_NAMES: &str = "get_log_channel_names";
const GET_TRACE_STATUS: &str = "get_trace_status";
const SET_LOG_CHANNEL_ASSIGNMENT: &str = "set_log_channel_assignment";
const SET_LOG_CHANNEL_THRESHOLD: &str = "set_log_channel_threshold";
const GET_LOG_CHANNEL_THRESHOLD: &str = "get_log_channel_threshold";
const BUFFER_OVERFLOW_NOTIFICATION: &str = "buffer_overflow_notification";
const SYNC_TIME_STAMP: &str = "sync_time_stamp";
const CALL_SWC_INJECTIONS: &str = "call_swc_injections";

/// Get the name of the service based on the service id given.
pub fn get_control_service_name(service_id: u32) -> Option<&'static str> {
    match service_id {
        0x01 => Some(SET_LOG_LEVEL),
        0x02 => Some(SET_TRACE_STATUS),
        0x03 => Some(GET_LOG_INFO),
        0x04 => Some(GET_DEFAULT_LOG_LEVEL),
        0x05 => Some(STORE_CONFIGURATION),
        0x06 => Some(RESET_TO_FACTORY_DEFAULT),
        0x0A => Some(SET_MESSAGE_FILTERING),
        0x11 => Some(SET_DEFAULT_LOG_LEVEL),
        0x12 => Some(SET_DEFAULT_TRACE_STATUS),
        0x13 => Some(GET_SOFTWARE_VERSION),
        0x15 => Some(GET_DEFAULT_TRACE_STATUS),
        0x17 => Some(GET_LOG_CHANNEL_NAMES),
        0x1F => Some(GET_TRACE_STATUS),
        0x20 => Some(SET_LOG_CHANNEL_ASSIGNMENT),
        0x21 => Some(SET_LOG_CHANNEL_THRESHOLD),
        0x22 => Some(GET_LOG_CHANNEL_THRESHOLD),
        0x23 => Some(BUFFER_OVERFLOW_NOTIFICATION),
        0x24 => Some(SYNC_TIME_STAMP),
        0xFFF..=0xFFFFFFFF => Some(CALL_SWC_INJECTIONS),
        _ => None,
    }
}
