//! Basic example to stream all win32 api calls to an ES instance
//!
use std::collections::HashMap;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use chrono::Utc;
use elasticsearch::{BulkParts, Elasticsearch, http::request::JsonBody, http::transport::Transport};
use log::{debug, info, warn};
use pelite::pe::Pe;
use pelite::PeView;
use serde_json::{Value, json};

use ttd::replay::events::{DataAccessMask, EventType};
use ttd::replay::{MemoryWatchpointData, RegisterContext, ReplayCursor, ReplayEngine, ReplayPosition};

const TRACKED_MODULES: &[&str] = &[
    "kernel32.dll",
    "kernelbase.dll",
    "ntdll.dll",
    "advapi32.dll",
    "user32.dll",
    "ws2_32.dll",
    "wininet.dll",
    "shell32.dll",
];

const ES_INDEX: &str = "win32-api-calls";
const BULK_DOCS_PER_FLUSH: usize = 500;

#[derive(Debug, Clone)]
struct ApiSymbol {
    module: String,
    function: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        bail!("usage: {} <trace.run> <elasticsearch_url>", args[0]);
    }
    let trace_path = std::path::Path::new(&args[1]);
    let es_url = &args[2];

    let transport = Transport::single_node(es_url).context("building ES transport")?;
    let es = Elasticsearch::new(transport);
    info!("connected to elasticsearch at {es_url}");

    let started = Instant::now();
    let replay = ReplayEngine::open(trace_path).context("loading trace")?;
    let pid = replay.system_info()?.pid()?;
    info!("trace loaded (pid={pid})");

    let mut api_map: HashMap<u64, ApiSymbol> = HashMap::new();

    for event in replay.module_loaded_events()? {
        let full_name = event.module.name.to_lowercase();
        let basename = std::path::Path::new(&full_name)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| full_name.clone());

        if !TRACKED_MODULES.contains(&basename.as_str()) {
            continue;
        }

        let mut cur = replay.cursor()?;
        cur.set_position(&ReplayPosition::from(&event.position));
        let image = cur.read_memory(event.module.address, event.module.size as usize)?;
        // let mut reader = std::io::Cursor::new(image.as_slice());
        let pe = PeView::from_bytes(&image)?;
        let exports = pe.exports()?;

        let mut count = 0usize;

        for result in exports.by()?.iter_names() {
            if let (Ok(name), Ok(entry)) = result {
                if entry.forward().is_some() {
                    continue; // skip forwarders
                }
                if name.is_empty() {
                    continue;
                }

                if let Some(rva) = entry.symbol() {
                    api_map.insert(
                        event.module.address + rva as u64,
                        ApiSymbol {
                            module: basename.clone(),
                            function: name.to_string(),
                        },
                    );
                    count += 1;
                }
            }
        }
        debug!("indexed {count} exports from {basename}");
    }
    info!("resolved {} Win32 API entry points", api_map.len());

    let mut cursor = replay.cursor()?;
    let mut armed = 0usize;
    for &addr in api_map.keys() {
        if cursor.add_memory_watchpoint(&MemoryWatchpointData {
            Address: addr,
            Size: 1,
            AccessMask: DataAccessMask::Execute.bits(),
            ..Default::default()
        })? {
            armed += 1;
        }
    }
    info!("{armed} watchpoints armed, replaying...");
    let mut buffer: Vec<JsonBody<Value>> = Vec::with_capacity(BULK_DOCS_PER_FLUSH * 2);
    let mut total = 0u64;

    loop {
        let stop = cursor.replay_forward(None)?;
        if stop.stop_reason != EventType::MemoryWatchpoint {
            break;
        }

        let pc = cursor.pc()?;
        let Some(sym) = api_map.get(&pc) else {
            // Watchpoint hit but PC isn't an entry point we know — skip.
            continue;
        };

        let position = cursor.position()?.to_string();
        let tid = cursor.thread_info()?.Id;
        let args = read_first_four_args(&cursor)?;

        let doc = json!({
            "@timestamp": Utc::now().to_rfc3339(),
            "process": {
                "pid": pid,
                "thread": { "id": tid },
            },
            "ttd": {
                "position": position,
                "pc": format!("{pc:#x}"),
            },
            "win32": {
                "module":   sym.module,
                "function": sym.function,
                "args":     args.iter().map(|a| format!("{a:#x}")).collect::<Vec<_>>(),
            },
        });

        buffer.push(json!({ "index": {} }).into());
        buffer.push(doc.into());
        total += 1;

        if buffer.len() >= BULK_DOCS_PER_FLUSH * 2 {
            flush(&es, &mut buffer).await?;
        }
    }

    flush(&es, &mut buffer).await?;
    info!("indexed {total} API calls in {:?}", started.elapsed());
    Ok(())
}

fn read_first_four_args(cursor: &ReplayCursor<'_>) -> Result<[u64; 4]> {
    Ok(match cursor.thread_context()? {
        RegisterContext::X64(c) => [c.Rcx, c.Rdx, c.R8, c.R9],
        RegisterContext::ARM64(c) => [c.X[0], c.X[1], c.X[2], c.X[3]],
        RegisterContext::X86(c) => {
            // stdcall/cdecl: args at [esp+4], [esp+8], [esp+0xC], [esp+0x10].
            let read = |off: u32| -> Result<u64> {
                let bytes = cursor.read_memory((c.Esp + off) as u64, 4)?;
                let buf: [u8; 4] = bytes.as_slice().try_into()?;
                Ok(u32::from_le_bytes(buf) as u64)
            };
            [read(4)?, read(8)?, read(0xc)?, read(0x10)?]
        }
    })
}

async fn flush(es: &Elasticsearch, buf: &mut Vec<JsonBody<Value>>) -> Result<()> {
    if buf.is_empty() {
        return Ok(());
    }
    let body = std::mem::take(buf);
    let n_docs = body.len() / 2;
    let resp = es.bulk(BulkParts::Index(ES_INDEX)).body(body).send().await?;
    let status = resp.status_code();
    let resp_body: Value = resp.json().await?;

    if !status.is_success() {
        warn!("bulk index returned HTTP {}", status);
    } else if resp_body.get("errors").and_then(|v| v.as_bool()) == Some(true) {
        warn!("bulk index completed with item-level errors");
    } else {
        debug!("flushed {n_docs} docs");
    }
    Ok(())
}
