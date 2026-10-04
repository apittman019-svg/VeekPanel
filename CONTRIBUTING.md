# Contributing

Read `AGENTS.md` before changing scope. Implement only the authorized milestone.
The source brief is preserved in `docs/PROJECT_REQUIREMENTS.md`; current priorities
and prototype limitations are in `AGENTS.md` and `docs/VERIFICATION.md`.

Use the pinned Rust toolchain. Before submitting:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
```

On Linux also run `tests/serial_pty.py`, `tests/capture_pty.py` with the hardware
binary, and `tests/audio/pipewire_integration.py` with the audio binary. Run Python
scripts with `python3`. Native audio integration must use its isolated test daemon;
never point automated mutation tests at a user's desktop audio server.
Tests must be hardware-free by default. Record physical results separately using
`docs/HARDWARE_VALIDATION.md`; preserve useful redacted raw captures and provenance.

Keep parsers independent of OS handles. Reject malformed bytes, bound buffers and
queues, and keep button presses separate from analog positions. Include regression
tests for protocol or connection fixes. Do not port vendor/decompiled code or import
unlicensed code. Cite packet facts and preserve notices for any licensed code reuse.

Do not commit USB serial numbers, private paths, credentials, personal captures,
build outputs or generated logs. `list`/`--raw` output is for local debugging and is
not a privacy-filtered diagnostic report. No root/admin runtime, arbitrary serial
probing, blanket USB permissions, firmware flashing, or undocumented write commands.
