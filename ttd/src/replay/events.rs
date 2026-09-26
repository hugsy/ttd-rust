use crate::prelude::*;

use derive_more::Display;
use ttd_sys::bindings;

pub type ModuleLoaded = ttd_sys::replay::events::ModuleLoaded;
pub type ModuleUnloaded = ttd_sys::replay::events::ModuleUnloaded;

use bitflags::bitflags;

// region: EventType

/// Enumeration of event kinds produced by the TTD replay system. Each variant
/// represents a distinct runtime event category (e.g.,
/// memory/position watchpoint, exception, interruption, etc.)
/// used for filtering, branching, and reporting during replay.
#[derive(Display, Debug, PartialEq)]
pub enum EventType {
    MemoryWatchpoint,
    PositionWatchpoint,
    Exception,
    Gap,
    Thread,
    StepCount,
    Position,
    Process,
    Interrupted,
    Error,
    Count,
    Invalid,
}

impl From<u8> for EventType {
    fn from(value: u8) -> Self {
        match value {
            bindings::root::TTD::Replay::EventType_MemoryWatchpoint => EventType::MemoryWatchpoint,
            bindings::root::TTD::Replay::EventType_PositionWatchpoint => EventType::PositionWatchpoint,
            bindings::root::TTD::Replay::EventType_Exception => EventType::Exception,
            bindings::root::TTD::Replay::EventType_Gap => EventType::Gap,
            bindings::root::TTD::Replay::EventType_Thread => EventType::Thread,
            bindings::root::TTD::Replay::EventType_StepCount => EventType::StepCount,
            bindings::root::TTD::Replay::EventType_Position => EventType::Position,
            bindings::root::TTD::Replay::EventType_Process => EventType::Process,
            bindings::root::TTD::Replay::EventType_Interrupted => EventType::Interrupted,
            bindings::root::TTD::Replay::EventType_Error => EventType::Error,
            bindings::root::TTD::Replay::EventType_Count => EventType::Count,
            _ => EventType::Invalid,
        }
    }
}

// endregion: EventType

// region: DataAccessType/DataAccessMask

bitflags! {
    /// Small integer-backed enum representing specific data access kinds (e.g.,
    /// read, write, execute) used in access filtering and watchpoint configuration.
    /// The underlying u8 stores the bit or value for each access type for compact
    /// FFI-friendly representation.
    pub struct DataAccessType: u8 {
        const Read          =     bindings::root::TTD::Replay::DataAccessType_Read;
        const Write         =     bindings::root::TTD::Replay::DataAccessType_Write;
        const Execute       =     bindings::root::TTD::Replay::DataAccessType_Execute;
        const CodeFetch     =     bindings::root::TTD::Replay::DataAccessType_CodeFetch;
        const Overwrite     =     bindings::root::TTD::Replay::DataAccessType_Overwrite;
        const DataMismatch  =     bindings::root::TTD::Replay::DataAccessType_DataMismatch;
        const NewData       =     bindings::root::TTD::Replay::DataAccessType_NewData;
        const RedundantData =     bindings::root::TTD::Replay::DataAccessType_RedundantData;
    }
}

bitflags! {
    /// Bitmask type representing which categories of data access are monitored or
    /// permitted (for example: reads, writes, execute, or metadata). Use this mask
    /// to specify or query access filters for memory watchpoints, logging, or
    /// permission checks within the replay engine.
    pub struct DataAccessMask: u8 {
    const Read          = bindings::root::TTD::Replay::DataAccessMask_Read;
    const Write         = bindings::root::TTD::Replay::DataAccessMask_Write;
    const Execute       = bindings::root::TTD::Replay::DataAccessMask_Execute;
    const CodeFetch     = bindings::root::TTD::Replay::DataAccessMask_CodeFetch;
    const Overwrite     = bindings::root::TTD::Replay::DataAccessMask_Overwrite;
    const DataMismatch  = bindings::root::TTD::Replay::DataAccessMask_DataMismatch;
    const NewData       = bindings::root::TTD::Replay::DataAccessMask_NewData;
    const RedundantData = bindings::root::TTD::Replay::DataAccessMask_RedundantData;
    const None      = bindings::root::TTD::Replay::DataAccessMask_None;
    const ReadWrite = bindings::root::TTD::Replay::DataAccessMask_ReadWrite;
    const All       = bindings::root::TTD::Replay::DataAccessMask_All;
}
}

// endregion: DataAccessType/DataAccessMask

// region: Exception type

#[derive(Debug)]
pub struct Exception {}

impl TryFrom<&ttd_sys::bindings::root::TTD::Replay::ExceptionEvent> for Exception {
    fn try_from(_value: &ttd_sys::bindings::root::TTD::Replay::ExceptionEvent) -> Result<Self> {
        todo!()
    }
    type Error = crate::error::Error;
}

// endregion: Exception type
