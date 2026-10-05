//! Explicit local stdin/stdout harness for integration tests; no network server.
use std::{
    io::{self, BufRead, Read, Write},
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--config") {
        return Err(
            "Usage: veek-runtime --config PATH (JSON lines on stdin; type=state or type=quit)"
                .into(),
        );
    }
    let path = PathBuf::from(args.next().ok_or("Missing config path")?);
    if args.next().is_some() {
        return Err("Unexpected argument".into());
    }
    let runtime = veek_runtime::start(path).map_err(io::Error::other)?;
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    loop {
        let mut line = String::new();
        let n = stdin.by_ref().take(2_097_153).read_line(&mut line)?;
        if n == 0 {
            break;
        }
        if n > 2_097_152 {
            return Err("Request exceeds 2 MiB".into());
        }
        let result = (|| -> Result<serde_json::Value, String> {
            let value: serde_json::Value =
                serde_json::from_str(&line).map_err(|e| e.to_string())?;
            if value["type"] == "quit" {
                return Ok(serde_json::json!({"quit":true}));
            }
            if value["type"] != "state" {
                runtime
                    .handle
                    .request(serde_json::from_value(value).map_err(|e| e.to_string())?)?;
            }
            serde_json::to_value(runtime.handle.state()).map_err(|e| e.to_string())
        })();
        let quit = result.as_ref().is_ok_and(|v| v["quit"] == true);
        writeln!(
            stdout,
            "{}",
            match result {
                Ok(v) => serde_json::json!({"ok":true,"state":v}),
                Err(e) => serde_json::json!({"ok":false,"error":e}),
            }
        )?;
        stdout.flush()?;
        if quit {
            break;
        }
    }
    Ok(())
}
