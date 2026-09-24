//! Contains the thin wrapper for the unsafe stuff

use derive_more::Display;
use std::ffi::{CString, c_void};
use std::ops::{Add, Sub};

use crate::bindings::root as ffi;
use crate::prelude::*;

pub mod events;

impl Add<u64> for ffi::TTD::Replay::Position {
    type Output = ffi::TTD::Replay::Position;

    fn add(mut self, rhs: u64) -> Self::Output {
        self.Steps += rhs;
        self
    }
}

impl Sub<u64> for ffi::TTD::Replay::Position {
    type Output = ffi::TTD::Replay::Position;

    fn sub(mut self, rhs: u64) -> Self::Output {
        self.Steps -= rhs;
        self
    }
}

impl std::fmt::Display for ffi::TTD::Replay::Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}:{:x}", self.Sequence, self.Steps)
    }
}

// region: EngineInfo

/// General information about the replay engine
pub struct EngineInfo {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
    pub license: String,
    pub author: String,
    pub banner: String,
    pub name: String,
}

impl Default for EngineInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineInfo {
    pub fn new() -> Self {
        let license = unsafe { CString::from_vec_unchecked(crate::bindings::root::TTD_FFI::LibraryLicense.to_vec()) };
        let author = unsafe { CString::from_vec_unchecked(crate::bindings::root::TTD_FFI::LibraryAuthor.to_vec()) };
        let banner = unsafe { CString::from_vec_unchecked(crate::bindings::root::TTD_FFI::LibraryBanner.to_vec()) };
        let name = unsafe { CString::from_vec_unchecked(crate::bindings::root::TTD_FFI::LibraryName.to_vec()) };

        // Expect banner format to be `ttd_sys v$mj.$mn.$patch`
        // Failing to extract correctly won't produce an error, just report the version as (0, 0, 0)
        let banner_str = banner.to_string_lossy();
        let binding = banner_str.replace("ttd_ffi v", "");
        let info: Vec<usize> = binding
            .split(".").map(|x| x.parse().unwrap_or_default()).collect();

        EngineInfo {
            major: *info.iter().nth(0).unwrap_or(&0),
            minor: *info.iter().nth(1).unwrap_or(&0),
            patch: *info.iter().nth(2).unwrap_or(&0),
            license: license.to_string_lossy().into(),
            author: author.to_string_lossy().into(),
            banner: banner_str.into_owned(),
            name: name.to_string_lossy().into(),
        }
    }
}
// endregion: EngineInfo

// region: ReplayEngine

pub struct ReplayEngine {
    inner: ffi::TTD_FFI::Replay::ReplayEngine,
}

impl Drop for ReplayEngine {
    fn drop(&mut self) {
        unsafe {
            self.inner.destruct();
        }
    }
}

impl ReplayEngine {
    /// Create and initialize a new [`ReplayEngine`], allocating the Replay Engine through
    /// FFI (equivalent to calling C++ `TTD::Replay::MakeReplayEngine()`)
    ///
    /// ## Returns
    /// - `Result<ReplayEngine>`: Ok with a constructed ReplayEngine on success; Err on failure (allocation or SDK error).
    ///
    /// ## Safety
    /// - Calls into FFI `TTD::Replay::IEngine::MakeReplayEngine()`
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: unsafe { ffi::TTD_FFI::Replay::ReplayEngine::new() },
        })
    }

    /// ## Description
    /// Load a TTD trace (UTF-16 path buffer) into the replay engine, initializing
    /// internal state from the specified trace path.
    ///
    /// Parameters:
    /// - `trace`: UTF-16 encoded path (slice of u16) pointing to the trace file or directory.
    ///
    /// ## Returns
    /// - `i32`: Raw SDK/FFI status code (0 for success in typical conventions; consult SDK docs for exact codes).
    ///
    /// ## Safety
    /// - Calls into FFI `TTD::Replay::IEngine::Load()`
    pub fn load(&self, trace: &[u16]) -> i32 {
        unsafe { self.inner.Load(trace.as_ptr()) }
    }

    /// ## Description
    /// Create a new ReplayCursor borrowed from the engine, allowing navigation and
    /// inspection of the loaded trace.
    ///
    /// ## Returns
    /// - `Result<ReplayCursor<'_>>`: Ok with a borrowed ReplayCursor on success; Err on failure.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::Load()`.
    pub fn cursor(&'_ self) -> Result<ReplayCursor<'_>> {
        let cursor = unsafe {
            let raw_cur = self.inner.NewCursor();
            if raw_cur == 0 {
                return Err(Error::ForeignFunctionError);
            }
            let mut cursor = ffi::TTD_FFI::Replay::ReplayCursor::new(raw_cur);

            // New cursors always should point to the start of the trace
            cursor.SetPosition(&self.get_lifetime().Min);

            cursor
        };

        Ok(ReplayCursor { inner: cursor, engine: self })
    }

    /// ## Description
    /// Return a reference to the replay's recorded lifetime range as an FFI
    /// [`ffi::TTD::Replay::PositionRange`], representing the earliest and latest valid
    /// replay positions captured in the trace.
    ///
    /// ## Returns
    /// - `&ffi::TTD::Replay::PositionRange`: Reference to the recorded position range.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::GetLifetime()`.
    pub fn get_lifetime(&self) -> &ffi::TTD::Replay::PositionRange {
        unsafe { &*self.inner.GetLifetime() }
    }

    /// ## Description
    /// Return a reference to the trace's captured system information (CPU, OS,
    /// address width, endianness, and related environment details) as an FFI
    /// [`ffi::TTD::SystemInfo`].
    ///
    /// ## Returns
    /// - `&ffi::TTD::SystemInfo`: Reference to the engine-owned SystemInfo for the loaded trace.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::GetSystemInfo()`.
    pub fn system_info(&self) -> &ffi::TTD::SystemInfo {
        unsafe { &*self.inner.GetSystemInfo() }
    }

    pub fn build_index(&self) -> u32 {
        unsafe { self.inner.BuildIndex() }
    }

    pub fn get_module_count(&self) -> usize {
        unsafe { self.inner.GetModuleCount() }
    }

    /// ## Description
    /// Return the list of modules observed in the loaded TTD trace as a Vec of FFI [`ffi::TTD::Replay::Module`],
    /// each containing metadata like base address, size, path, and timestamps.
    ///
    /// ## Returns
    /// - `Vec<ffi::TTD::Replay::Module>`: Module list extracted from the loaded trace.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::GetModuleList()`.
    pub fn get_module_list(&self) -> Vec<ffi::TTD::Replay::Module> {
        unsafe {
            let cnt = self.get_module_count();
            match cnt {
                0 => vec![],
                _ => {
                    let data = self.inner.GetModuleList();
                    assert!(!data.is_null());
                    core::slice::from_raw_parts(data, cnt).into()
                }
            }
        }
    }

    pub fn get_module_instance_count(&self) -> usize {
        unsafe { self.inner.GetModuleInstanceCount() }
    }

    /// ## Description
    /// Return the list of module instances observed in the loaded TTD trace as a
    /// Vec of FFI [`ffi::TTD::Replay::ModuleInstance`], each representing a specific
    /// in-process instantiation of a module with load base, relocation offset,
    /// and lifetime information.
    ///
    /// ## Returns
    /// - `Vec<ffi::TTD::Replay::ModuleInstance>`: Module instance list extracted
    ///   from the loaded trace.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::GetModuleInstanceList()`.
    pub fn get_module_instance_list(&self) -> Vec<ffi::TTD::Replay::ModuleInstance> {
        unsafe {
            let cnt = self.get_module_instance_count();
            match cnt {
                0 => vec![],
                _ => {
                    let data = self.inner.GetModuleInstanceList();
                    assert!(!data.is_null());
                    core::slice::from_raw_parts(data, cnt).into()
                }
            }
        }
    }

    pub fn get_thread_count(&self) -> usize {
        unsafe { self.inner.GetThreadCount() }
    }

    /// ## Description
    /// Return the list of threads observed in the loaded TTD trace as a Vec of FFI [`ffi::TTD::Replay::ThreadInfo`],
    /// each describing a recorded thread's identifier, creation/termination positions, and basic execution metadata.
    ///
    /// ## Returns
    /// - `Vec<ffi::TTD::Replay::ThreadInfo>`: Thread list extracted from the loaded trace.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::GetThreadList()`.
    pub fn get_thread_list(&self) -> Vec<ffi::TTD::Replay::ThreadInfo> {
        unsafe {
            let cnt = self.get_thread_count();
            match cnt {
                0 => vec![],
                _ => {
                    let data = self.inner.GetThreadList();
                    assert!(!data.is_null());
                    core::slice::from_raw_parts(data, cnt).into()
                }
            }
        }
    }

    pub fn get_module_loaded_event_count(&self) -> usize {
        unsafe { self.inner.GetModuleLoadedEventCount() }
    }

    /// ## Description
    /// Return the list of module-loaded events recorded in the loaded TTD trace as
    /// a Vec of FFI [`ffi::TTD::Replay::ModuleLoadedEvent`], each containing the replay
    /// position and associated module metadata.
    ///
    /// ## Returns
    /// - `Vec<ffi::TTD::Replay::ModuleLoadedEvent>`: Module-loaded event list extracted from the loaded trace.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::GetModuleLoadedEventList()`.
    pub fn get_module_loaded_event_list(&self) -> Vec<ffi::TTD::Replay::ModuleLoadedEvent> {
        unsafe {
            let cnt = self.get_module_loaded_event_count();
            match cnt {
                0 => vec![],
                _ => {
                    let data = self.inner.GetModuleLoadedEventList();
                    assert!(!data.is_null());
                    core::slice::from_raw_parts(data, cnt).into()
                }
            }
        }
    }

    pub fn get_module_unloaded_event_count(&self) -> usize {
        unsafe { self.inner.GetModuleUnloadedEventCount() }
    }

    /// ## Description
    /// Return the list of module-unloaded events recorded in the loaded TTD trace as a Vec of FFI
    /// [`ffi::TTD::Replay::ModuleUnloadedEvent`], each containing the replay position and associated module metadata.
    ///
    /// ## Returns
    /// - `Vec<ffi::TTD::Replay::ModuleUnloadedEvent>`: Module-unloaded event list extracted from the loaded trace.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::GetModuleUnloadedEventList()`.
    pub fn get_module_unloaded_event_list(&self) -> Vec<ffi::TTD::Replay::ModuleUnloadedEvent> {
        unsafe {
            let cnt = self.get_module_unloaded_event_count();
            match cnt {
                0 => vec![],
                _ => {
                    let data = self.inner.GetModuleUnloadedEventList();
                    assert!(!data.is_null());
                    core::slice::from_raw_parts(data, cnt).into()
                }
            }
        }
    }

    pub fn get_exception_event_count(&self) -> usize {
        unsafe { self.inner.GetExceptionEventCount() }
    }

    /// ## Description
    /// Return the list of exception events recorded in the loaded TTD trace as a Vec of FFI [`ffi::TTD::Replay::ExceptionEvent`],
    /// each containing the replay position, thread id, exception code/type, and captured context.
    ///
    /// ## Returns
    /// - `Vec<ffi::TTD::Replay::ExceptionEvent>`: Exception event list extracted from the loaded trace.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD_FFI::Replay::ReplayEngine::GetExceptionEventList()`.
    pub fn get_exception_event_list(&self) -> Vec<ffi::TTD::Replay::ExceptionEvent> {
        unsafe {
            let cnt = self.get_exception_event_count();
            match cnt {
                0 => vec![],
                _ => {
                    let data = self.inner.GetExceptionEventList();
                    assert!(!data.is_null());
                    core::slice::from_raw_parts(data, cnt).into()
                }
            }
        }
    }
}

// endregion: ReplayEngine

// region: ReplayCursor

pub struct ReplayCursor<'a> {
    inner: ffi::TTD_FFI::Replay::ReplayCursor,
    engine: &'a ReplayEngine,
}

impl<'a> Drop for ReplayCursor<'a> {
    fn drop(&mut self) {
        unsafe {
            self.inner.destruct();
        }
    }
}

impl<'a> ReplayCursor<'a> {
    /// Advance the cursor forward toward an optional target FFI position. If
    /// until is Some, replay proceeds until that FFI position or a stopping
    /// event; if None, it advances a default step or to the next event. Returns
    /// the raw FFI replay result containing stop reason and execution metrics.
    ///
    /// Parameters:
    /// - until: Optional [`ffi::TTD::Replay::Position`] target to stop at.
    ///
    /// ## Returns
    /// - Result<ffi::TTD::Replay::ICursorView_ReplayResult>
    ///
    /// ## Safety
    /// - Calls into unsafe FFI; ensure the replay session and referenced resources remain valid.
    pub fn replay_forward(&mut self, until: Option<ffi::TTD::Replay::Position>) -> Result<ffi::TTD::Replay::ICursorView_ReplayResult> {
        unsafe {
            let max_pos = ReplayCursor::max();
            let mut out = ffi::TTD::Replay::ICursorView_ReplayResult::default();
            let limit = match until {
                Some(pos) => pos,
                None => max_pos,
            };

            if self.inner.ReplayForward(&limit, &mut out) != 0 {
                return Err(Error::ForeignFunctionError);
            }

            Ok(out)
        }
    }

    /// Move the cursor backward toward an optional target FFI position. If `until`
    /// is Some, replay rewinds until that FFI position or a stopping event; if
    /// None, it rewinds a default step or to the previous event. Returns the raw
    /// FFI replay result with stop reason and execution metrics.
    ///
    /// Parameters:
    /// - until: Optional FFI `TTD::Replay::Position` target to stop at.
    ///
    /// ## Returns
    /// - Result<ffi::TTD::Replay::ICursorView_ReplayResult>
    ///
    /// ## Safety
    /// - Calls into unsafe FFI; ensure the replay session and referenced resources remain valid.
    pub fn replay_backward(&mut self, until: Option<ffi::TTD::Replay::Position>) -> Result<ffi::TTD::Replay::ICursorView_ReplayResult> {
        unsafe {
            let min_pos = ReplayCursor::min();
            let mut out = ffi::TTD::Replay::ICursorView_ReplayResult::default();
            let limit = match until {
                Some(pos) => pos,
                None => min_pos,
            };

            if self.inner.ReplayBackward(&limit, &mut out) != 0 {
                return Err(Error::ForeignFunctionError);
            }

            Ok(out)
        }
    }

    pub fn max() -> ffi::TTD::Replay::Position {
        ffi::TTD::Replay::Position {
            Sequence: ffi::TTD::SequenceId_Max,
            Steps: ffi::TTD::Replay::StepCount_Max,
        }
    }

    pub fn min() -> ffi::TTD::Replay::Position {
        ffi::TTD::Replay::Position {
            Sequence: ffi::TTD::SequenceId_Min,
            Steps: ffi::TTD::Replay::StepCount_Min,
        }
    }

    /// Set the cursor immediately to the given FFI replay position; subsequent
    /// operations begin from `pos`.
    ///
    /// Parameters:
    /// - pos: Reference to the target FFI `TTD::Replay::Position` to set.
    ///
    /// ## Safety
    /// - Caller must ensure `pos` is valid for the current replay session; passing an invalid position may cause undefined behavior.
    pub fn set_position(&mut self, pos: &ffi::TTD::Replay::Position) {
        unsafe { self.inner.SetPosition(pos) };
    }

    /// Return a reference to the cursor's current FFI `ffi::TTD::Replay::Position`.
    ///
    /// ## Returns
    /// - &ffi::TTD::Replay::Position: Reference to the current replay position.
    ///
    /// ## Safety
    /// - Returned reference aliases FFI-owned data; ensure the parent replay session outlives its use.
    pub fn get_position(&self) -> &ffi::TTD::Replay::Position {
        unsafe { &*self.inner.GetPosition() }
    }

    pub fn get_previous_position(&self) -> &ffi::TTD::Replay::Position {
        unsafe { &*self.inner.GetPreviousPosition() }
    }

    /// Return a reference to the current FFI `ffi::TTD::Replay::ThreadInfo` for the
    /// cursor's thread context.
    ///
    /// ## Returns
    /// - &ffi::TTD::Replay::ThreadInfo: Reference to the thread info for the current replay position.
    ///
    /// ## Safety
    /// - Returned reference aliases FFI-owned data; ensure the parent replay session and cursor
    ///   remain valid while the reference is used.
    pub fn get_thread_info(&self) -> &ffi::TTD::Replay::ThreadInfo {
        unsafe { &*self.inner.GetThreadInfo() }
    }

    pub fn get_teb_address(&self) -> u64 {
        unsafe { self.inner.GetTebAddress() }
    }

    pub fn get_program_counter(&self) -> u64 {
        unsafe { self.inner.GetProgramCounter() }
    }

    pub fn get_stack_pointer(&self) -> u64 {
        unsafe { self.inner.GetStackPointer() }
    }

    pub fn get_frame_pointer(&self) -> u64 {
        unsafe { self.inner.GetFramePointer() }
    }

    /// Retrieve the register context for the cursor's current thread at its
    /// current replay position, returning a borrowed RegisterContext that exposes
    /// register values and architecture-specific state.
    ///
    /// ## Returns
    /// - [`Result<RegisterContext<'_>>`]
    ///
    /// ## Safety
    /// - May borrow FFI-owned register data; ensure the replay session and cursor outlive the returned context.
    pub fn get_thread_context(&self) -> Result<RegisterContext> {
        unsafe {
            let arch: ProcessorArchitecture = self.engine.system_info().System.ProcessorArchitecture.try_into()?;
            Ok(match arch {
                ProcessorArchitecture::X64 => {
                    let _ref: &ffi::AMD64_CONTEXT = &*self.inner.GetX64RegisterContext();
                    RegisterContext::X64(_ref.to_owned())
                }
                ProcessorArchitecture::X86 => {
                    let _ref: &ffi::X86_NT5_CONTEXT = &*self.inner.GetX86RegisterContext();
                    RegisterContext::X86(_ref.to_owned())
                }
                ProcessorArchitecture::ARM64 => {
                    let _ref: &ffi::ARM64_CONTEXT = &*self.inner.GetArm64RegisterContext();
                    RegisterContext::ARM64(_ref.to_owned())
                }
            })
        }
    }

    /// Retrieve the extended register context (including SIMD/vector and other
    /// architecture-specific extended state) for the cursor's current thread at
    /// its current replay position. Returns an owned ExtendedRegisterContext or an
    /// error if retrieval fails.
    ///
    /// ## Returns
    /// - [`Result<ExtendedRegisterContext>`]
    ///
    /// ## Safety
    /// - May copy or borrow FFI-owned extended state; ensure the replay session and cursor remain valid during retrieval.
    pub fn get_thread_extended_context(&self) -> Result<ExtendedRegisterContext> {
        unsafe {
            let arch: ProcessorArchitecture = self.engine.system_info().System.ProcessorArchitecture.try_into()?;
            Ok(match arch {
                ProcessorArchitecture::X64 => {
                    let _ref: &ffi::AVX_EXTENDED_CONTEXT = &*self.inner.GetX64ExtendedRegisterContext();
                    ExtendedRegisterContext::X64(_ref.to_owned())
                }
                ProcessorArchitecture::X86 => {
                    let _ref: &ffi::AVX_EXTENDED_CONTEXT = &*self.inner.GetX86ExtendedRegisterContext();
                    ExtendedRegisterContext::X86(_ref.to_owned())
                }
                ProcessorArchitecture::ARM64 => {
                    let _ref: &ffi::ARM64_NEON128 = &*self.inner.GetArm64ExtendedRegisterContext();
                    ExtendedRegisterContext::ARM64(_ref.to_owned())
                }
            })
        }
    }

    /// Read `size` bytes from the replay's current memory view at `address` and
    /// return them as a `Vec<u8>`, reflecting memory as observed at the cursor's
    /// current position.
    ///
    /// Parameters:
    /// - `address`: Starting virtual address to read from.
    /// - `size`: Number of bytes to read.
    ///
    /// ## Returns
    /// - `Result<Vec<u8>>`
    ///
    /// ## Safety
    /// - Calls into unsafe FFI; ensure the replay session and cursor remain valid
    ///   and the requested address range is within the trace's captured memory.
    pub fn read_memory(&self, address: u64, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; size];
        let res = unsafe { self.inner.QueryMemoryBuffer(address, buffer.as_mut_ptr(), buffer.len() as u64) };

        match res {
            0 => Ok(buffer),
            _ => Err(Error::ForeignFunctionError),
        }
    }

    /// Return the current replay configuration flags controlling behavior (e.g.,
    /// logging, determinism, performance options) as a ReplayFlags value.
    ///
    /// ## Safety
    /// - Calls into unsafe FFI `TTD::Replay::ICursor::GetReplayFlags()`
    pub fn get_replay_flags(&self) -> ReplayFlags {
        unsafe { self.inner.GetReplayFlags().into() }
    }

    /// Set the replay behavior flags to the provided ReplayFlags value, updating
    /// runtime options that affect replay semantics (e.g., logging, determinism,
    /// or performance-related toggles).
    ///
    /// Parameters:
    /// - flags: ReplayFlags value specifying the new replay configuration.
    ///
    /// ## Safety
    /// - Calls into unsafe FFI `TTD::Replay::ICursor::SetReplayFlags()`
    pub fn set_replay_flags(&mut self, flags: ReplayFlags) {
        unsafe { self.inner.SetReplayFlags(flags.into()) }
    }

    /// Add a memory watchpoint that triggers when the specified memory region is
    /// accessed during replay. Returns true if the watchpoint was registered and
    /// active, false if ignored or already present.
    ///
    /// Parameters:
    /// - `watch_point`: Reference to FFI [`ffi::TTD::Replay::MemoryWatchpointData`]
    ///   describing address, size, and access type.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD::Replay::ICursor::AddMemoryWatchpoint()`
    pub fn add_memory_watchpoint(&mut self, watch_point: &ffi::TTD::Replay::MemoryWatchpointData) -> bool {
        unsafe { self.inner.AddMemoryWatchpoint(watch_point) }
    }

    /// Remove a previously registered memory watchpoint matching the provided FFI
    /// [`ffi::TTD::Replay::MemoryWatchpointData`]. Returns true if a watchpoint was found
    /// and removed, false if none matched.
    ///
    /// Parameters:
    /// - `watch_point`: Reference to FFI [`ffi::TTD::Replay::MemoryWatchpointData`] identifying the watchpoint to remove.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD::Replay::ICursor::RemoveMemoryWatchpoint()`
    pub fn remove_memory_watchpoint(&mut self, watch_point: &ffi::TTD::Replay::MemoryWatchpointData) -> bool {
        unsafe { self.inner.RemoveMemoryWatchpoint(watch_point) }
    }

    /// Register a position-based watchpoint that triggers when the replay reaches
    /// the specified FFI [`ffi::TTD::Replay::PositionWatchpointData`]. Returns true if
    /// the watchpoint was registered, false if ignored or already present.
    ///
    /// Parameters:
    /// - `watch_point`: Reference to FFI [`ffi::TTD::Replay::PositionWatchpointData`] specifying the target position and trigger criteria.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD::Replay::ICursor::AddPositionWatchpoint()`
    pub fn add_position_watchpoint(&mut self, watch_point: &ffi::TTD::Replay::PositionWatchpointData) -> bool {
        unsafe { self.inner.AddPositionWatchpoint(watch_point) }
    }

    /// Remove a previously registered position-based watchpoint matching the
    /// provided FFI [`ffi::TTD::Replay::PositionWatchpointData`]. Returns true if a
    /// matching watchpoint was found and removed, false otherwise.
    ///
    /// Parameters:
    /// - watch_point: Reference to FFI [`ffi::TTD::Replay::PositionWatchpointData`] identifying the watchpoint to remove.
    ///
    /// ## Safety
    /// - Calls into FFI `TTD::Replay::ICursor::RemovePositionWatchpoint()`
    pub fn remove_position_watchpoint(&mut self, watch_point: &ffi::TTD::Replay::PositionWatchpointData) -> bool {
        unsafe { self.inner.RemovePositionWatchpoint(watch_point) }
    }

    pub fn set_register_changed_callback(&mut self, cb: RegisterChangedCallbackUnsafe) {
        unsafe { self.inner.SetRegisterChangedCallback(Some(cb), 0) }
    }
    pub fn set_replay_progress_callback(&mut self, cb: ReplayProgressCallbackUnsafe) {
        unsafe { self.inner.SetReplayProgressCallback(Some(cb), 0) }
    }
}

// endregion: ReplayCursor

// region: Cursor callbacks

pub type RegisterChangedCallbackUnsafe = unsafe extern "C" fn(
    context: usize,
    reg_id: u8,
    old_data: *const c_void,
    new_data: *const c_void,
    data_size_in_bytes: usize,
    thread: *const ffi::TTD::Replay::IThreadView,
);

pub type ReplayProgressCallbackUnsafe = unsafe extern "C" fn(ctx: usize, pos: *const ffi::TTD::Replay::Position);

// endregion: Cursor callbacks

// region: ReplayFlags

#[repr(u32)]
#[derive(Default, Display)]
pub enum ReplayFlags {
    #[default]
    None = 0,
    ReplayOnlyCurrentThread = 0x00000001,
    ReplayAllSegmentsWithoutFiltering = 0x00000002,
    ReplaySegmentsSequentially = 0x00000004,
}
impl From<ReplayFlags> for u32 {
    fn from(val: ReplayFlags) -> Self {
        val as u32
    }
}

impl From<u32> for ReplayFlags {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::default(),
            1 => ReplayFlags::ReplayOnlyCurrentThread,
            2 => ReplayFlags::ReplayAllSegmentsWithoutFiltering,
            4 => ReplayFlags::ReplaySegmentsSequentially,
            _ => unimplemented!(),
        }
    }
}
// endregion: ReplayFlags

// region: ReplayModule

/// Represents a module (loaded binary or library) observed during a TTD record
/// or replay session. Encapsulates metadata such as module base address,
/// size, file path, timestamp/version info, and identifiers used by the TTD
/// SDK to correlate module load/unload events. Use this struct to inspect
/// which modules were present at specific replay positions, resolve symbols,
/// or present module lists to users.
#[derive(Debug)]
pub struct ReplayModule {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub checksum: u32,
    pub timestamp: u32,
}

impl TryFrom<&crate::bindings::root::TTD::Replay::Module> for ReplayModule {
    fn try_from(value: &crate::bindings::root::TTD::Replay::Module) -> Result<Self> {
        let name_slice = unsafe { std::slice::from_raw_parts(value.pName, value.NameLength) };

        Ok(Self {
            name: String::from_utf16(name_slice)?.to_string(),
            address: value.Address,
            size: value.Size,
            checksum: value.Checksum,
            timestamp: value.Timestamp,
        })
    }

    type Error = crate::error::Error;
}

// endregion: ReplayModule

// region: RegisterContext / RegisterExtendedContext

#[allow(clippy::large_enum_variant)]
pub enum RegisterContext {
    ARM64(ffi::ARM64_CONTEXT),
    X64(ffi::AMD64_CONTEXT),
    X86(ffi::X86_NT5_CONTEXT),
}

impl std::fmt::Display for RegisterContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterContext::X64(ctx) => {
                write!(
                    f,
                    "rax={:016x} rbx={:016x} rcx={:016x}
rdx={:016x} rsi={:016x} rdi={:016x}
rip={:016x} rsp={:016x} rbp={:016x}
 r8={:016x}  r9={:016x} r10={:016x}
r11={:016x} r12={:016x} r13={:016x}
r14={:016x} r15={:016x}",
                    ctx.Rax,
                    ctx.Rbx,
                    ctx.Rcx,
                    ctx.Rdx,
                    ctx.Rsi,
                    ctx.Rdi,
                    ctx.Rip,
                    ctx.Rsp,
                    ctx.Rbp,
                    ctx.R8,
                    ctx.R9,
                    ctx.R10,
                    ctx.R11,
                    ctx.R12,
                    ctx.R13,
                    ctx.R14,
                    ctx.R15
                )
            }
            RegisterContext::X86(ctx) => write!(
                f,
                "eax={:08x} ebx={:08x} ecx={:08x} edx={:08x} esi={:08x} edi={:08x}
eip={:08x} esp={:08x} ebp={:08x} ",
                ctx.Eax, ctx.Ebx, ctx.Ecx, ctx.Edx, ctx.Esi, ctx.Edi, ctx.Eip, ctx.Esp, ctx.Ebp,
            ),
            RegisterContext::ARM64(ctx) => {
                write!(
                    f,
                    "x0={:016x}   x1={:016x}   x2={:016x}   x3={:016x}
x4={:016x}   x5={:016x}   x6={:016x}   x7={:016x}
x8={:016x}   x9={:016x}  x10={:016x}  x11={:016x}
x12={:016x}  x13={:016x}  x14={:016x}  x15={:016x}
x16={:016x}  x17={:016x}  x18={:016x}  x19={:016x}
x20={:016x}  x21={:016x}  x22={:016x}  x23={:016x}
x24={:016x}  x25={:016x}  x26={:016x}  x27={:016x}
x28={:016x}   fp={:016x}   lr={:016x}   sp={:016x}
 pc={:016x}",
                    ctx.X[0],
                    ctx.X[1],
                    ctx.X[2],
                    ctx.X[3],
                    ctx.X[4],
                    ctx.X[5],
                    ctx.X[6],
                    ctx.X[7],
                    ctx.X[8],
                    ctx.X[9],
                    ctx.X[10],
                    ctx.X[11],
                    ctx.X[12],
                    ctx.X[13],
                    ctx.X[14],
                    ctx.X[15],
                    ctx.X[16],
                    ctx.X[17],
                    ctx.X[18],
                    ctx.X[19],
                    ctx.X[20],
                    ctx.X[21],
                    ctx.X[22],
                    ctx.X[23],
                    ctx.X[24],
                    ctx.X[25],
                    ctx.X[26],
                    ctx.X[27],
                    ctx.X[28],
                    ctx.Fp,
                    ctx.Lr,
                    ctx.Sp,
                    ctx.Pc
                )
            }
        }
    }
}

pub enum ExtendedRegisterContext {
    ARM64(ffi::ARM64_NEON128),
    X64(ffi::AVX_EXTENDED_CONTEXT),
    X86(ffi::AVX_EXTENDED_CONTEXT),
}

// endregion: RegisterContext / RegisterExtendedContext

// region: ProcessorArchitecture

#[repr(u16)]
pub enum ProcessorArchitecture {
    X64 = 9,
    X86 = 0,
    ARM64 = 12,
}
impl TryFrom<u16> for ProcessorArchitecture {
    type Error = crate::error::Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            12 => Ok(ProcessorArchitecture::ARM64),
            9 => Ok(ProcessorArchitecture::X64),
            0 => Ok(ProcessorArchitecture::X86),
            _ => Err(Error::ConversionError),
        }
    }
}

// endregion: ProcessorArchitecture

#[cfg(test)]
mod test {
    use crate::replay::{EngineInfo, ReplayEngine};

    fn get_test_trace() -> Vec<u16> {
        let mut trace_path = std::path::PathBuf::from(std::env::var("TEMP").expect("failed to get TEMP env var").as_str());
        trace_path.push("test.run\0");
        trace_path.to_string_lossy().encode_utf16().collect()
    }

    #[test]
    fn test_version() {
        let info = EngineInfo::new();
        assert_eq!((info.major, info.minor, info.patch), (0, 1, 0));
        assert_ne!(info.license.len(), 0);
        assert_ne!(info.author.len(), 0);
        assert_ne!(info.banner.len(), 0);
        assert_ne!(info.name.len(), 0);
    }

    #[test]
    fn test_ffi_load_simple() {
        let replay = ReplayEngine::new().expect("failed to create a new replayer");

        let trace_path = get_test_trace();
        assert_eq!(replay.load(trace_path.as_ref()), 0);

        let lt = replay.get_lifetime();

        // position navigation
        for _i in 1..10 {
            let mut cursor = replay.cursor().unwrap();
            let pos = cursor.get_position();
            assert_eq!(*pos, lt.Min);

            cursor.set_position(&lt.Max);
            assert_eq!(*cursor.get_position(), lt.Max);

            cursor.set_position(&lt.Min);
            assert_eq!(*cursor.get_position(), lt.Min);
        }

        // replay navigation
        for _i in 1..10 {
            let mut cursor = replay.cursor().unwrap();

            // new cursor point to the lifetime min
            let pos = cursor.get_position();
            assert_eq!(*pos, lt.Min);

            // replay until the litetime end
            let res = cursor.replay_forward(None).expect("failed to replay forward");
            assert_eq!(res.StopReason, 7); // end of process
            assert_ne!(res.InstructionsExecuted, 0);

            // TTD ends execution when reaching the last possible executable instruction
            // which is the previous Position (i.e. maximum position - 1)
            assert_eq!(*cursor.get_previous_position(), lt.Max);

            // rewind to start
            let res = cursor.replay_backward(None).expect("failed to replay backward");
            assert_eq!(res.StopReason, 7); // start of process
            assert_ne!(res.InstructionsExecuted, 0);
            assert_eq!(*cursor.get_position(), lt.Min);
        }
    }

    #[test]
    fn sys_ffi_system_info() {
        let engine = ReplayEngine::new().expect("failed to create a new replayer");

        // Note: a trace is needed to have TTD::Replay::SystemInfo populated
        let trace_path = get_test_trace();
        assert_eq!(engine.load(trace_path.as_ref()), 0);

        let info = engine.system_info();
        assert_eq!(info.MajorVersion, 1);
        assert!(info.MinorVersion > 0);
        assert!(info.MinorVersion < 12);
        assert_ne!(info.ProcessId, 0);
        assert_eq!(info.SystemName.len(), 64);
        assert_eq!(info.UserName.len(), 64);
    }
}
