//! Module that directly handles the unsafe interaction with the C++ API

use crate::prelude::*;

use crate::bindings::root as ffi;

/// ## Description
/// High-level handle representing a recording session or view of a captured
/// TTD trace. Provides APIs to control or inspect recording-related data such
/// as live recording metadata, active recording lifetime, and helpers to
/// produce ReplayEngine/ReplayCursor instances from a recorded trace.
///
pub struct Recorder<'a> {
    /// The associated FFI object
    inner: ffi::TTD_FFI::Record::ScopedRecorder,

    /// The associated [`RecorderEngine`]
    #[allow(dead_code)]
    engine: &'a RecorderEngine,
}

impl<'a> Recorder<'a> {

    /// ## Description
    /// Begin or attach to a recording session context represented by this Recorder.
    /// Transitions the recorder into an active state where recording metadata is
    /// available and replay/inspection helpers can be created.
    ///
    pub fn start(&'a self) {
        unsafe {
            self.inner.Start();
        }
    }


    /// ## Description
    /// Stop or detach from the active recording session associated with this
    /// Recorder, finalizing any in-progress metadata and making the recorded trace
    /// ready for replay or inspection.
    ///
    pub fn stop(&'a self) {
        unsafe {
            self.inner.Stop();
        }
    }
}

/// ## Description
/// Owned engine managing a TTD recording session and its resources. Responsible
/// for starting, stopping, and configuring recordings, allocating recorder
/// state, and producing Recorder handles or persisted trace output.
///
pub struct RecorderEngine {
    name: String,
    inner: ffi::TTD_FFI::Record::RecorderEngine,
}

impl RecorderEngine {
    /// ## Description
    /// Initialize a new [`RecorderEngine`]
    ///
    /// ## Returns
    /// - A [`Result`] of [`RecorderEngine`]
    ///
    /// ## Safety
    /// Calls into `TTD_FFI::Record::RecorderEngine`
    pub fn new(name: &[u8]) -> Result<Self> {
        Ok(Self {
            inner: unsafe { ffi::TTD_FFI::Record::RecorderEngine::new(name.as_ptr()) },
            name: std::str::from_utf8(name)?.to_string(),
        })
    }

    /// ## Description
    /// Create a new recorder
    ///
    /// ## Returns
    ///  - A [`Result`] a result to [`Recorder`] whose lifetime is tied to this [`RecorderEngine`]
    pub fn recorder(&'_ self) -> Result<Recorder<'_>> {
        let recorder = unsafe { self.inner.Recorder() };
        Ok(Recorder { inner: recorder, engine: self })
    }

    /// ## Description
    /// Save content into the file path given in arguments
    ///
    /// ## Parameters
    ///  - `path` a mutable slice that will store the recorded trace path
    ///
    /// ## Returns
    ///  - bool : `true` on success
    ///
    /// ## Safety
    /// Calls into FFI `TTD_FFI::Record::RecorderEngine::Save`
    pub fn save(&self, path: &mut [u16]) -> bool {
        unsafe { self.inner.Save(path.as_ptr(), path.len() as u64) }
    }
}

impl std::fmt::Display for RecorderEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RecorderEngine({})", self.name)
    }
}

impl std::fmt::Debug for RecorderEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecorderEngine").field("name", &self.name).finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use crate::record::{RecorderEngine};

    #[test]
    #[ignore = "Record API seems buggy for now"]
    fn record_self() {
        let eng = RecorderEngine::new("TestRecorder".as_ref()).unwrap();
        let rec = eng.recorder().unwrap();

        rec.start();
        for _i in 1..10_000 {}
        rec.stop();

        let mut trace_path = [0u16; 1024];
        eng.save(&mut trace_path);
    }
}
