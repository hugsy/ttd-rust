# `ttd-rust`

Rust bindings for the [**Microsoft Time Travel Debugging (TTD) SDK**](https://github.com/microsoft/WinDbg-Samples/blob/master/TTD/README.md).

This crate provides a safe and ergonomic Rust interface to the TTD SDK, enabling developers to record, replay, and analyze program execution using Microsoft's time-travel debugging technology.

> [!WARNING]
> TTD-Rust is currently a work in progress, the API (especially in the `ttd` crate) may change between commits.
> If you start using the library, prefer locking it to a specific commit

## Overview

[Microsoft Time Travel Debugging](https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/time-travel-debugging-overview) (TTD) is a powerful debugging tool that allows you to step forward and backward through program execution.

`ttd-rust` wraps the native TTD SDK, making it accessible from Rust without requiring manual FFI boilerplate.

## Features

- Safe Rust wrappers around TTD SDK functions.
- Multi replay engine/multi-cursor support
- Thread-safe
- Replay program execution from created traces. Trace recording bindings will be added soon.
- Query events, memory state, and call stacks at any point in time.
- Integration with existing Rust debugging workflows.

## Build Requirements

- Rust 1.90+
- Windows 10 or later (TTD is Windows-only).
  - `winget` must be present
- C++ build tools (for compiling native bindings) including
  - `cmake`
  - VisualStudio 2019 or later

## Installation

Add `ttd-rust` to your `Cargo.toml`:

```toml
[dependencies]
ttd-rust = { git = "https://github.com/calladoum-elastic/ttd-rust" }
```

## Build

`cargo` will entirely handle the build. If the libraries/dlls are missing, it will automatically download them.

```pwsh
cargo build --all-targets
```

## Test

### Test the C++ layer

```pwsh
cargo build
ctest -C Debug -T test --test-dir .\ttd_sys\ttd_ffi\build\tests
```

### Test the Rust layer

```pwsh
cargo test
```

## Documentation

* `cargo doc`
* [Microsoft TTD Documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/time-travel-debugging)
* [Microsoft TTD SDK Documentation](https://github.com/microsoft/WinDbg-Samples/blob/master/TTD/README.md)


## Examples

### Usage

```rust
use ttd::replay::ReplayEngine;
use ttd::replay::events::EventType;

fn main() -> Result<(), ttd::error::Error> {
  // Open a recording
  let mut engine = ReplayEngine::open(r"c:\path\to\my_trace.run")?;

  // Print some info
  dbg!(engine.system_info);

  // Navigation through the trace can be done using cursors
  let mut cursor = engine.cursor()?;

  // Replay forward until the end of the trace
  let replay_result = cursor.replay_forward(None)?;
  assert_eq!(res.stop_reason, EventType::Process);

  // Inspect memory state at a specific point
  let memory_dump = cursor.read_memory(0x12345678)?;
  println!("Memory dump: {:?}", memory_dump);

  // You can also retrieve the register context at current point
  let ctx = cursor.thread_context()?;
  println!("Regsisters\n{}", ctx);

  Ok(())
}
```

### Binaries
Several examples were provided to illustrate how to use the Rust API:
 - `auto_unpack` - an automatic unpack that will extract encoded/encrypted payload injected using `VirtualAlloc(RWX)`/`VirtualProtect(RWX)`

## License

This project is licensed under the Apache-2.0 License.
See LICENSE for details.

## Disclaimer

`ttd-rust` is not a Microsoft product.
It is a community maintained project that depends on the Microsoft TTD SDK.
