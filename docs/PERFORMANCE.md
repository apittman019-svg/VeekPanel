# First measured optimization pass

This pass reduces software overhead. It does not establish Windows/Nobara idle
CPU, memory, startup or physical-input latency budgets. Animations remain a
separate planned Astra pass.

## What changed

- Successful input/write commands publish the complete native observation already
  used for target validation/readback. Extra immediate enumerations were removed;
  every write still checks generation, capability, target presence and readback.
  Failures reconcile again and rearm input. Group writes remain non-transactional.
- The runtime shares immutable, versioned publications. Equal observations do not
  allocate/copy another publication. Comparison and copying occur outside the
  publication mutex; the UI reads a consistent pointer/version cheaply.
- Visible UI polling sends its last publication version. Unchanged replies omit
  runtime/config/mapping payloads, preserve frontend object references and still
  report current tray/startup status. Unknown versions receive a full snapshot.
- After handling commands, the owner dispatches native events without sleeping
  and waits on the command queue for up to 20 ms. Follow-up requests wake it
  immediately. Idle native waits and hardware scheduling retain their 20 ms limit.
  An isolated first request can still encounter an existing idle wait/native call.

The 250 ms observed-audio reconciliation and 400 ms visible UI refresh schedules
are retained. Button barriers, expired input, profile-change invalidation, pickup,
generation checks and configuration revisions keep their existing semantics.

## Synthetic results: 2026-10-07

Ubuntu 24.04 remote workspace, pinned Rust 1.99.0, release profile. The injected
mock backend has 100 targets and 8 profiles; one armed analog control alternates
100 confirmed writes. Each phase has three sequential measured runs. Baseline
source is local commit `13a4903e698d576eedf8b10ac2b607e84f53b110` plus the
baseline-only probe. No real USB, native audio or native GUI runs in this probe.

| Metric | Before | After |
| --- | --- | --- |
| Full audio snapshots for 100 updates | 498–500 | 300 |
| Confirmed writes | 100 | 100 |
| Burst request p50 | 20.415–20.439 ms | 0.118–0.125 ms |
| Burst request p95 | 20.562–20.637 ms | 0.163–0.227 ms |
| Burst request p99 | 20.605–20.766 ms | 0.263–0.418 ms |
| 1,000 frontend state acquisitions | 25.102–28.161 ms copying | 0.008–0.009 ms sharing |

This fixture uses about 40% fewer full audio snapshots. The legacy copying API
remains available; the desktop now uses shared acquisition. Idle observation still
read audio three times in the one-second sampling windows, but produced zero new
publications after initialization. The fixture's serialized state is 27,224 bytes;
unchanged IPC replies omit it and mapping resolution while retaining desktop status.

These are small synthetic measurements, sensitive to scheduling and workload.
An early baseline batched requests and used 403 snapshots; the three isolated
baseline runs above used 498–500. Under concurrent compilation, optimized p99
was higher, up to 5.837 ms. Do not turn burst timings or a pointer-copy benchmark
into a physical latency guarantee or a claimed overall application speedup.
Raw measured runs are in `tests/perf/results-2026-10-07.json`.

## Reproduce

Run alone after build activity settles. It uses a temporary configuration and an
injected mock backend; personal audio/startup/USB state is not touched.

```sh
cargo test -p veek-runtime --release --locked runtime_performance_probe -- --ignored --nocapture --test-threads=1
pnpm --dir ui test
```

For the recorded baseline, use its original commit in a separate worktree, then
apply only `tests/perf/runtime-baseline.patch`. The handoff bundle includes that
commit; a patch-only import does not preserve local commit history.

```sh
veek_perf_baseline_dir=$(mktemp -d /tmp/veek-perf-before.XXXXXX)
git worktree add --detach "$veek_perf_baseline_dir" 13a4903e698d576eedf8b10ac2b607e84f53b110
git -C "$veek_perf_baseline_dir" apply --unidiff-zero "$PWD/tests/perf/runtime-baseline.patch"
cargo test --manifest-path "$veek_perf_baseline_dir/Cargo.toml" -p veek-runtime --release --locked runtime_performance_probe -- --ignored --nocapture --test-threads=1
```

## Remaining native measurements

On ordinary Windows 10/11 and Nobara KDE sessions, record source/install path,
target/mapping counts and workload. Measure idle CPU/RSS with the window visible
and in tray, startup to usable dashboard, and event-to-native-readback p50/p95/p99.
Distinguish runtime scheduling, native audio enumeration/write time and visible
render time. Run the existing private PipeWire/native GUI integrations first,
including external volume changes, app relaunch, groups, profile switches and
service recovery. Keep real hardware trials and long-run soak separate.

The remote workspace cannot run those integrations because it denies private Unix
sockets. Numerical native release budgets remain unset until those measurements
exist. Native enumeration, per-group writes and backend event/idle wakeups remain
future profiling candidates; no Windows COM/PipeWire cache rewrite is included.
