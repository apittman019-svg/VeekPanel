# Continue on the home PC

Same ChatGPT account does not put this remote checkout on the home filesystem.
The ZIP transfers code independently of GitHub connector permissions. It includes
`VeekPanel-development.bundle` with local commit history and a combined
`veekpanel-development.patch` fallback. Import one, not both. Both include
control feedback, startup/background work,
Windows startup cleanup, Linux tray recovery and measured runtime/IPC optimization.
Do not separately reapply an
older development or feedback patch.

Patch base: `8e60df93119a9cd2697d07dd0f2beca52f4f821d`, the remote
`codex/profile-settings` schema-2 foundation. The published schema-1 `main` is not
the patch base. `gemini-test` and `QWEN-test` remain separate and unchanged.

The bundle retains the original commits, including the performance baseline. It
requires the patch base above to exist locally. Verify the bundle and import it
into a fresh branch/worktree. The patch transfers the final code without history.
Download/extract the ZIP on the PC. Open local Codex in VeekPanel and supply
the extracted folder's actual path with this prompt:

```text
Continue VeekPanel from the mobile handoff. Inspect AGENTS.md, recent history,
and the dirty worktree first. Preserve all existing user/Qwen/Gemini changes.
The handoff includes a Git bundle and an equivalent patch with control feedback,
startup, installer cleanup, tray recovery and runtime/IPC optimization, based on commit
8e60df93119a9cd2697d07dd0f2beca52f4f821d on codex/profile-settings.

Fetch the repository using the PC's normal configured Git credentials if needed.
Preserve the existing checkout and use an isolated sibling worktree; do not reset
or overwrite my checkout.
Prefer git bundle verify and fetch the bundled codex/profile-settings head into
a fresh codex branch, then create an isolated worktree there. Do not also apply
the patch. If using the patch fallback, start a new codex branch at the exact base,
check whether it is already applied and
run git apply --check before applying;
stop and explain conflicts instead of forcing them. Read docs/LOCAL_HANDOFF.md
and the newest docs/VERIFICATION.md entry after applying.

Run relevant Rust/desktop/frontend checks and the private native PipeWire and
GUI integrations on Nobara. Use dbus-run-session for the GUI harness. Do not
change my personal startup registration or audio services just to run tests.
Run the disposable-bus tray regression documented in docs/BACKGROUND_VALIDATION.md.
Run pnpm --dir ui test and the synthetic release probe in docs/PERFORMANCE.md.
Measure native idle CPU/memory and response latency separately from that mock probe.
Record actual results; keep physical PCPanel and Windows/reboot/soak gates open.
Commit the verified changes on the new codex branch. Report the branch and result
before publishing a new release or changing main, QWEN-test, or gemini-test.
Then continue reliability/Linux-distribution work from the current requirements.
The user plans to use Astra for animations; this pass prioritizes optimization.
```

The handoff does not include build outputs, an installer, credentials or private
hardware captures. This increment has not been pushed and has no new CI result.
