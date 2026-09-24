//! For this demonstation unpacker, we want to extract any shellcode that is encoded
// or encrypted in the trace. We achieve this by watching VirtualProtect
// calls with the PAGE_EXECUTE_READWRITE.
// For each of those calls, we jump ahead to the time the culprit address
// is being executed, print the disassembled payload and Yara-scan it.
//
use std::io::{Read, Write};

use iced_x86::{DecoderOptions, Formatter, IntelFormatter};

use anyhow::{Ok, Result, bail};
use extfmt::{AsHexdump, hexdump};
use log::{debug, info};

use ttd::replay::events::{DataAccessMask, EventType};
use ttd::replay::{MemoryWatchpointData, RegisterContext, ReplayEngine, ReplayPosition};

const NUMBER_OF_INSTRUCTIONS_TO_PRINT: usize = 20;

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    #[cfg(not(debug_assertions))]
    env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        bail!("No trace provided");
    }

    let now = std::time::Instant::now();

    let replay = ReplayEngine::new()?;
    let trace_path = std::path::Path::new(args[1].as_str());
    replay.load(trace_path)?;
    info!("loaded trace {:?}", &trace_path);

    //
    // 1. Locate kernel32!VirtualProtect
    //
    let find_load_event_for_module = |mod_lower: &str| {
        Ok(replay
            .module_loaded_events()?
            .into_iter()
            .find(|e| e.module.name.to_lowercase().ends_with(&mod_lower))
            .ok_or(ttd::error::Error::NotFound)?)
    };

    let find_export = |module: &[u8], export_name: &str| {
        let pe = pelite::PeView::from_bytes(module)?;
        match pe.get_export_by_name(export_name)? {
            pelite::pe::exports::Export::Symbol(value) => Ok(*value),
            pelite::pe::exports::Export::Forward(_) => todo!(),
        }
    };

    let get_module_base_address = |module_name: &str, function_name: &str| -> Result<u64> {
        let event = find_load_event_for_module(module_name)?;
        let mut cur = replay.cursor()?;
        cur.set_position(&(ReplayPosition::from(&event.position)));
        let data = cur.read_memory(event.module.address, event.module.size as usize)?;
        debug!("data@{:#x}\n{}", event.module.address, hexdump!(&data[..64]));
        let rva = find_export(&data, function_name)? as u64;
        debug!("rva={:#x}", rva);
        Ok(event.module.address + rva)
    };

    debug!("looking for kernelbase!VirtualProtect()");
    let virtualprotect_address = get_module_base_address("kernelbase.dll", "VirtualProtect")?;
    info!("Watching calls to kernelbase!VirtualProtect(RWX) (at {:#x})", virtualprotect_address);

    //
    // 2. Set a (execute) memory breakpoint to kernelbase!virtualprotect
    //
    let mut cursor = replay.cursor()?;
    let watch_point = MemoryWatchpointData {
        Address: virtualprotect_address,
        Size: 1,
        AccessMask: DataAccessMask::Execute.bits(),
        ..Default::default()
    };

    cursor.add_memory_watchpoint(&watch_point)?;
    info!("Watching for execution at {:#x} set", virtualprotect_address);

    let read_u64_at = |addr: u64, pos: &ReplayPosition| -> Result<u32> {
        let mut _cur = replay.cursor()?;
        _cur.set_position(pos);
        let data: [u8; 4] = _cur.read_memory(addr, _cur.pointer_size()?)?.try_into().expect("Slice with incorrect length");
        Ok(u32::from_le_bytes(data))
    };

    let bitness = match cursor.thread_context()? {
        RegisterContext::X86(_) => 32u32,
        RegisterContext::X64(_) => 64u32,
        RegisterContext::ARM64(_) => todo!("use bad64"),
    };

    let mut yara_compiler = yara_x::Compiler::new();

    if let std::result::Result::Ok(mut fd) = std::fs::File::open(r"C:\git\ttd\auto_unpack\rules\redline.yar") {
        let mut source: Vec<u8> = vec![];
        fd.read_to_end(&mut source)?;
        yara_compiler.add_source(&*source)?;
    }

    let yara_rules = yara_compiler.build();
    let mut yara_scanner = yara_x::Scanner::new(&yara_rules);

    loop {
        //
        // 3. Play the trace
        //
        let result = &cursor.replay_forward(None)?;
        if result.stop_reason != EventType::MemoryWatchpoint {
            break;
        }
        debug!("fwdreplay reason: {}, executed {} insns", result.stop_reason, result.instructions_executed);

        //
        // 4. Inspect the `flNewProtect` argument, looking PAGE_EXECUTE_READWRITE
        //
        const PAGE_EXECUTE_READWRITE: u32 = 0x40;

        let (arg0, arg1, arg2, arg3) = {
            let ctx = cursor.thread_context()?;
            let curpos = cursor.position()?;
            debug!("context at {}:\n{}", &curpos, &ctx);
            match ctx {
                RegisterContext::X86(x86_ctx) => (
                    read_u64_at((x86_ctx.Esp + 0x04) as u64, &curpos)?,
                    read_u64_at((x86_ctx.Esp + 0x08) as u64, &curpos)?,
                    read_u64_at((x86_ctx.Esp + 0x0c) as u64, &curpos)?,
                    read_u64_at((x86_ctx.Esp + 0x10) as u64, &curpos)?,
                ),
                RegisterContext::X64(x64_ctx) => (
                    read_u64_at(x64_ctx.Rcx, &curpos)?,
                    read_u64_at(x64_ctx.Rdx, &curpos)?,
                    read_u64_at(x64_ctx.R8, &curpos)?,
                    read_u64_at(x64_ctx.R9, &curpos)?,
                ),
                RegisterContext::ARM64(arm64_ctx) => (
                    read_u64_at(arm64_ctx.X[0], &curpos)?,
                    read_u64_at(arm64_ctx.X[1], &curpos)?,
                    read_u64_at(arm64_ctx.X[2], &curpos)?,
                    read_u64_at(arm64_ctx.X[3], &curpos)?,
                ),
            }
        };

        if arg2 != PAGE_EXECUTE_READWRITE {
            debug!("kernelbase!VirtualProtect({arg0:x}, {arg1:x}, {arg2:x}, {arg3:x}) not RWX, skipping...");
            continue;
        }

        info!("kernelbase!VirtualProtect({arg0:x}, {arg1:x}, {arg2:x}, {arg3:x}) hit, setting watch point");

        //
        // 5. Set an execution watchpoint on execution to that address
        // And advance to this address
        //
        cursor.remove_memory_watchpoint(&watch_point)?;
        cursor.add_memory_watchpoint(&MemoryWatchpointData {
            Address: arg0.into(),
            Size: 1,
            AccessMask: DataAccessMask::Execute.bits(),
            ..Default::default()
        })?;

        cursor.replay_forward(None)?;

        let pc = match cursor.thread_context()? {
            RegisterContext::X86(ctx) => ctx.Eip as u64,
            RegisterContext::X64(ctx) => ctx.Rip,
            RegisterContext::ARM64(ctx) => ctx.Pc,
        };
        let mem = cursor.read_memory(pc, arg1 as usize)?;

        //
        // 6. When the watchpoint is hit, analyze the shellcode
        //

        //
        // 6.0. Dump the payload to disk for later analysis
        //
        {
            let mut tempfile = std::path::PathBuf::from(std::env::var("TEMP")?);
            tempfile.push(format!("Payload-{}.dmp", cursor.position()?.to_string().replace(":", "_")));
            std::fs::File::create(tempfile)?.write_all(&mem)?;
        }

        //
        // 6.1. Disassemble the instructions
        //
        {
            info!("Dumping shellcode at IP={:x} (sz={}) executed at {}", pc, arg1, cursor.position()?);
            let mut decoder = iced_x86::Decoder::with_ip(bitness, &mem, pc, DecoderOptions::NONE);
            let mut formatter = IntelFormatter::new();
            let mut output = String::new();
            for (idx, insn) in decoder.iter().enumerate() {
                if idx == NUMBER_OF_INSTRUCTIONS_TO_PRINT {
                    println!("...");
                    break;
                }
                output.clear();
                formatter.format(&insn, &mut output);
                println!("{:#010x} {}", insn.ip(), output);
            }
        }

        //
        // 6.2. Launch a Yara scan
        //
        {
            let results = yara_scanner.scan(&mem)?;
            if results.matching_rules().len() > 0 {
                info!("Yara hit:");
                results.matching_rules().for_each(|rule| info!("- {}", rule.identifier()));
            } else {
                debug!("No Yara hit");
            }
        }

        break;
    }

    debug!("end position {}", &cursor.position()?);
    debug!("done in {:.4?}", now.elapsed());
    Ok(())
}
