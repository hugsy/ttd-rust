# `ttd-rust`

A Rust library for handling Microsoft Time Travel Debugging traces.

The purpose is to provide safe, ergonomic Rust bindings and higher-level helpers for the Microsoft TTD SDK so you can load traces, inspect recorded state (memory, threads, modules, registers), navigate forward/backward through execution, and register watchpoints or event observers. This crate interacts with tThe lower level crate `ttd_sys` which handles the FFI bindings and expose them to Rust as more friendly types (`ReplayEngine`, `ReplayCursor`, `ReplayPosition`, `ReplayModule`, `SystemInfo`, etc.) which Rust  manages ownership, lifetimes on.

Key features already implemented are:
 * Load and open TTD traces from disk and query trace metadata (system info, process id, modules, threads).
 * Create borrowed cursors to navigate recorded execution forward and backward, seek to positions, and read memory/register state as captured.
 * Register memory-based and position-based watchpoints to observe when particular addresses or positions are reached during replay.
 * Enumerate recorded events (module load/unload, exceptions, and other SDK event types) with both high-level Rust types and raw FFI event records.
* Rely on Rust lifetime management to ensure no corruption happens at the lower level.

## Usage

### Replay

#### Get trace info

```rust no_run
use ttd::replay::ReplayEngine;

fn main() -> Result<(), ttd::error::Error> {
  // Open a recording
  let mut engine = ReplayEngine::open(r"c:\path\to\my_trace.run")?;

  // Print some info
  let si = engine.system_info()?;
  dbg!(&si);

  Ok(())
}
```


#### Navigate in a trace

```rust no_run
use ttd::replay::ReplayEngine;
use ttd::replay::events::EventType;

fn main() -> Result<(), ttd::error::Error> {
  // Open a recording
  let mut engine = ReplayEngine::open(r"c:\path\to\my_trace.run")?;

  // Open a cursor to navigate the trace
  let mut cursor = engine.cursor()?;

  // Replay forward until the end of the trace
  let replay_result = cursor.replay_forward(None)?;
  assert_eq!(replay_result.stop_reason, EventType::Process);

  // Or you can jump to a position directly
  // cursor.set_position( ReplayPosition{...});

  Ok(())
}
```


#### Accessing memory

```rust no_run
use ttd::replay::ReplayEngine;

fn main() -> Result<(), ttd::error::Error> {
  // Open a recording
  let mut engine = ReplayEngine::open(r"c:\path\to\my_trace.run")?;

  // Open a cursor to navigate the trace
  let cursor = engine.cursor()?;

  // Inspect memory state at a specific point
  let memory_dump = cursor.read_memory(0x12345678, 16)?;
  println!("Memory snapshot: {:?}", memory_dump);

  Ok(())
}
```