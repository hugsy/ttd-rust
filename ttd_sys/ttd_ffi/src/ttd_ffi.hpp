#pragma once

#ifndef __HAS_TTD_FFI
#define __HAS_TTD_FFI

#include "constants.hpp"

using i8    = char;
using u8    = unsigned char;
using i16   = short;
using u16   = unsigned short;
using i32   = int;
using u32   = unsigned int;
using i64   = long long;
using u64   = unsigned long long;
using isize = i64;
using usize = u64;
using uptr  = usize;

#include <wrl.h>

#include <vector>

#include "TTD/IReplayEngine.h"
#include "TTD/IReplayEngineRegisters.h"
#include "TTD/IReplayEngineStl.h"
#include "TTD/TTDLiveRecorder.h"


#define LIBTTD_INVALID_VALUE ((i32) - 1)
#define LIBTTD_ERROR_GENERIC ((i32) - 1)
#define LIBTTD_ERROR_NOT_FOUND ((i32) - 2)
#define LIBTTD_ERROR_INITIALIZATION ((i32) - 3)
#define LIBTTD_ERROR_INVALID_INDEX ((i32) - 4)

namespace TTD_FFI
{
namespace Replay
{

const static size_t MAX_ENGINE = 256;
const static size_t MAX_CURSOR = 256;


class ReplayEngine
{
private:
    TTD::Replay::UniqueReplayEngine m_Engine;

public:
    /// Initialize the engine
    ReplayEngine();

    /// Destroy the engine
    ~ReplayEngine();

    /// Creates a new cursor on the heap, and returns the pointer as raw value.
    const uptr
    NewCursor() const;

    /// Load trace from the `.run` filepath passed as argument
    i32
    Load(const u16* trace) const;

    /// Get the trace entire life range
    TTD::Replay::PositionRange const&
    GetLifetime() const;

    /// Get system info from the replay engine
    TTD::SystemInfo const&
    GetSystemInfo() const;

    /// Build a trace index, to boost search speed
    u32
    BuildIndex() const;

    size_t
    GetModuleCount() const;

    TTD::Replay::Module const*
    GetModuleList() const;

    size_t
    GetModuleInstanceCount() const;

    TTD::Replay::ModuleInstance const*
    GetModuleInstanceList() const;

    size_t
    GetThreadCount() const;

    TTD::Replay::ThreadInfo const*
    GetThreadList() const;

    size_t
    GetModuleLoadedEventCount() const;

    TTD::Replay::ModuleLoadedEvent const*
    GetModuleLoadedEventList() const;

    size_t
    GetModuleUnloadedEventCount() const;

    TTD::Replay::ModuleUnloadedEvent const*
    GetModuleUnloadedEventList() const;

    size_t
    GetExceptionEventCount() const;

    TTD::Replay::ExceptionEvent const*
    GetExceptionEventList() const;
};


class ReplayCursor
{
private:
    TTD::Replay::UniqueCursor m_Cursor;

public:
    /// Initialize a `ReplayCursor`. The argument is a raw pointer to a `TTD::Replay::ICursor`
    /// which will be owned and managed by this instance.
    ReplayCursor(const uptr);

    ~ReplayCursor();

    /// Replay the trace forward.
    /// Note: it is the responsibility of the caller to allocate and manage the `ReplayResult*` structure
    /// @returns 0 on success
    i32
    ReplayForward(TTD::Replay::Position const&, TTD::Replay::ICursorView::ReplayResult*);

    /// Replay the trace backward.
    /// Note: it is the responsibility of the caller to allocate and manage the `ReplayResult*` structure
    /// @returns 0 on success
    i32
    ReplayBackward(TTD::Replay::Position const&, TTD::Replay::ICursorView::ReplayResult*);

    void
    SetPosition(TTD::Replay::Position const& pos);

    TTD::Replay::Position const&
    GetPosition() const;

    TTD::Replay::Position const&
    GetPreviousPosition() const;

    TTD::Replay::ThreadInfo const&
    GetThreadInfo() const;

    u64
    GetTebAddress() const;

    u64
    GetProgramCounter() const;

    u64
    GetStackPointer() const;

    u64
    GetFramePointer() const;

    const X86_NT5_CONTEXT*
    GetX86RegisterContext() const;

    const AVX_EXTENDED_CONTEXT*
    GetX86ExtendedRegisterContext() const;

    const AMD64_CONTEXT*
    GetX64RegisterContext() const;

    const AVX_EXTENDED_CONTEXT*
    GetX64ExtendedRegisterContext() const;

    const ARM64_CONTEXT*
    GetArm64RegisterContext() const;

    const ARM64_NEON128*
    GetArm64ExtendedRegisterContext() const;

    void
    SetReplayFlags(TTD::Replay::ReplayFlags flags);

    TTD::Replay::ReplayFlags
    GetReplayFlags() const;

    i32
    QueryMemoryBuffer(u64 address, u8* buf, usize bufsz) const;

    bool
    AddMemoryWatchpoint(TTD::Replay::MemoryWatchpointData const&);

    bool
    RemoveMemoryWatchpoint(TTD::Replay::MemoryWatchpointData const&);

    bool
    AddPositionWatchpoint(TTD::Replay::PositionWatchpointData const&);

    bool
    RemovePositionWatchpoint(TTD::Replay::PositionWatchpointData const&);


#pragma region Cursor Callbacks
    void
    SetReplayProgressCallback(TTD::Replay::ICursorView::ReplayProgressCallback* cb, uptr context);

    void
    SetRegisterChangedCallback(TTD::Replay::ICursorView::RegisterChangedCallback*, uptr context);
#pragma endregion Cursor Callbacks
};


} // namespace Replay


namespace Record
{

const static size_t MAX_RECORDER = 256;

class ScopedRecorder
{
private:
    const Microsoft::WRL::ComPtr<TTD::ILiveRecorder> m_pRecorder;

public:
    ScopedRecorder(Microsoft::WRL::ComPtr<TTD::ILiveRecorder> const&);

    ~ScopedRecorder();

    ScopedRecorder(const ScopedRecorder&) = delete;

    ScopedRecorder(ScopedRecorder&&) = delete;

    ScopedRecorder&
    operator=(const ScopedRecorder&) = delete;

    ScopedRecorder&
    operator=(ScopedRecorder&&) = delete;

    void
    Start() const;

    void
    Stop() const;
};

class RecorderEngine
{
private:
    Microsoft::WRL::ComPtr<TTD::ILiveRecorder> m_Engine;

public:
    RecorderEngine(u8 const* name);

    const ScopedRecorder
    Recorder() const;

    bool
    Save(u16 const*, usize) const;
};

} // namespace Record

} // namespace TTD_FFI
#endif // !__HAS_TTD_FFI
