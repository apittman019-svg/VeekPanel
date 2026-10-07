# Dependency license inventory

Generated from `cargo metadata --locked` for the checked-in lockfile. This lists
declared SPDX/license expressions across targets, including dependencies not built
on the current platform. It is not a bundled binary notice file; packaging must
include applicable license texts/native-library notices. VeekPanel contains no
vendored PCPanel source or assets. `serialport` declares MPL-2.0; retain its notices
and satisfy applicable source obligations when distributing binaries.

Native M2 uses `windows`/`windows-core` 0.62.2 (MIT OR Apache-2.0) and
`pipewire`/`libspa` 0.10.1 (MIT), with system libpipewire on Linux. Native libraries
remain system-managed; the diagnostic archive does not ship PipeWire itself.
`packaging/third-party/cookie-factory-0.3.3` supplies the exact-version upstream
MIT notice omitted from that registry archive; provenance is recorded beside it.

Desktop startup/single-instance dependencies are pinned in the separate `app/Cargo.lock`:
`tauri-plugin-single-instance` 2.5.2, Linux `zbus` 5.19.0, and Windows
`windows-registry` 0.6.1. Distribution notice generation must use the app's reachable
dependency graph. No autostart-plugin implementation or external startup code is copied;
the small native registration adapters are original VeekPanel code.

| Package | Version | Declared license |
| --- | --- | --- |
| aho-corasick | 1.1.5 | Unlicense OR MIT |
| annotate-snippets | 0.11.5 | MIT OR Apache-2.0 |
| anstream | 1.0.0 | MIT OR Apache-2.0 |
| anstyle | 1.0.14 | MIT OR Apache-2.0 |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 |
| anstyle-wincon | 3.0.11 | MIT OR Apache-2.0 |
| bindgen | 0.72.1 | BSD-3-Clause |
| bitflags | 1.3.2 | MIT/Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| block2 | 0.6.2 | MIT |
| cc | 1.6.0 | MIT OR Apache-2.0 |
| cexpr | 0.6.0 | Apache-2.0/MIT |
| cfg-expr | 0.20.10 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| cfg_aliases | 0.2.2 | MIT |
| clang-sys | 1.9.1 | Apache-2.0 |
| clap | 4.6.7 | MIT OR Apache-2.0 |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 |
| clap_derive | 4.6.7 | MIT OR Apache-2.0 |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 |
| cookie-factory | 0.3.3 | MIT |
| core-foundation | 0.10.1 | MIT OR Apache-2.0 |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 |
| ctrlc | 3.5.2 | MIT/Apache-2.0 |
| dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT |
| either | 1.18.0 | MIT OR Apache-2.0 |
| equivalent | 1.0.2 | Apache-2.0 OR MIT |
| errno | 0.3.14 | MIT OR Apache-2.0 |
| find-msvc-tools | 0.1.14 | MIT OR Apache-2.0 |
| glob | 0.3.4 | MIT OR Apache-2.0 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| hidapi | 2.6.7 | MIT |
| indexmap | 2.14.2 | Apache-2.0 OR MIT |
| io-kit-sys | 0.4.1 | MIT / Apache-2.0 |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 |
| itertools | 0.13.0 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| libc | 0.2.190 | MIT OR Apache-2.0 |
| libloading | 0.8.9 | ISC |
| libspa | 0.10.1 | MIT |
| libspa-sys | 0.10.1 | MIT |
| libudev | 0.3.0 | MIT |
| libudev-sys | 0.1.4 | MIT |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| mach2 | 0.4.3 | BSD-2-Clause OR MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| minimal-lexical | 0.2.1 | MIT/Apache-2.0 |
| nix | 0.26.4 | MIT |
| nix | 0.31.3 | MIT |
| nom | 7.1.3 | MIT |
| nom | 8.0.0 | MIT |
| objc2 | 0.6.4 | MIT |
| objc2-encode | 4.1.0 | MIT |
| once_cell_polyfill | 1.70.2 | MIT OR Apache-2.0 |
| pipewire | 0.10.1 | MIT |
| pipewire-sys | 0.10.1 | MIT |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| regex | 1.13.1 | MIT OR Apache-2.0 |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT |
| rustix | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.151 | MIT OR Apache-2.0 |
| serde_spanned | 1.1.1 | MIT OR Apache-2.0 |
| serialport | 4.10.1 | MPL-2.0 |
| shlex | 1.3.0 | MIT OR Apache-2.0 |
| shlex | 2.0.1 | MIT OR Apache-2.0 |
| smallvec | 1.16.2 | MIT OR Apache-2.0 |
| strsim | 0.11.1 | MIT |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.6 | MIT OR Apache-2.0 |
| system-deps | 7.0.8 | MIT OR Apache-2.0 |
| target-lexicon | 0.13.5 | Apache-2.0 WITH LLVM-exception |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| toml | 1.1.6+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_datetime | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_parser | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_writer | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 |
| unescaper | 0.1.10 | MIT OR GPL-3.0-only |
| unicode-ident | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| unicode-width | 0.2.2 | MIT OR Apache-2.0 |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT |
| version-compare | 0.2.1 | MIT |
| windows | 0.62.2 | MIT OR Apache-2.0 |
| windows-collections | 0.3.2 | MIT OR Apache-2.0 |
| windows-core | 0.62.2 | MIT OR Apache-2.0 |
| windows-future | 0.3.2 | MIT OR Apache-2.0 |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 |
| windows-link | 0.2.1 | MIT OR Apache-2.0 |
| windows-numerics | 0.3.1 | MIT OR Apache-2.0 |
| windows-result | 0.4.1 | MIT OR Apache-2.0 |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 |
| windows-threading | 0.2.1 | MIT OR Apache-2.0 |
| windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| winnow | 1.0.4 | MIT |
| zmij | 1.0.23 | MIT |
