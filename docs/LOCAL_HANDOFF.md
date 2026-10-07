# Continue on the home PC

Same ChatGPT account does not put this remote checkout on the home filesystem.
The combined `veekpanel-development.patch` transfers code independently of GitHub
connector permissions. It includes control feedback, startup/background work,
Windows startup cleanup and Linux tray recovery. Do not separately reapply an
older development or feedback patch.

Patch base: `8e60df93119a9cd2697d07dd0f2beca52f4f821d`, the remote
`codex/profile-settings` schema-2 foundation. The published schema-1 `main` is not
the patch base. `gemini-test` and `QWEN-test` remain separate and unchanged.

Download the combined patch on the PC. Open local Codex in VeekPanel and supply
the file's actual path with this prompt:

```text
Continue VeekPanel from the mobile handoff. Inspect AGENTS.md, recent history,
and the dirty worktree first. Preserve all existing user/Qwen/Gemini changes.
The attached veekpanel-development.patch includes control feedback, startup,
Windows installer cleanup and Linux tray recovery, based on commit
8e60df93119a9cd2697d07dd0f2beca52f4f821d on codex/profile-settings.

Fetch the repository using the PC's normal configured Git credentials if needed.
If the existing checkout differs or is dirty, use an isolated sibling worktree
and a new codex branch at that exact base; do not reset or overwrite my checkout.
Check whether the patch is already applied. Run git apply --check before applying;
stop and explain conflicts instead of forcing them. Read docs/LOCAL_HANDOFF.md
and the newest docs/VERIFICATION.md entry after applying.

Run relevant Rust/desktop/frontend checks and the private native PipeWire and
GUI integrations on Nobara. Use dbus-run-session for the GUI harness. Do not
change my personal startup registration or audio services just to run tests.
Run the disposable-bus tray regression documented in docs/BACKGROUND_VALIDATION.md.
Record actual results; keep physical PCPanel and Windows/reboot/soak gates open.
Commit the verified changes on the new codex branch. Report the branch and result
before publishing a new release or changing main, QWEN-test, or gemini-test.
Then continue reliability/Linux-distribution work from the current requirements.
```

The patch does not include build outputs, an installer, credentials or private
hardware captures. This increment has not been pushed and has no new CI result.
