# luau-perf

A static performance linter for Luau.

## What it does

Scans `.lua` / `.luau` files for performance antipatterns - allocations in loops, untracked connections, deprecated APIs, missing `--!native` headers, and things that prevent FASTCALL/GETIMPORT optimizations.

## Install

```bash
cargo install luauperf
```

Requires [Rust](https://rustup.rs). Or build from source (see below).

## Getting started

```bash
# Lint a directory
luauperf src/

# JSON output for CI or editor integration
luauperf src/ --format json

# See all available rules.
luauperf --list-rules

# Generate a config file.
luauperf --init

# Attempt to automatically fix warnings and errors
luauperf src/ --fix
```

## Config

Add a `luauperf.toml` to your project root:

```toml
# Exclude paths (substring match).
exclude = ["Packages/", "Generated/", "node_modules/"]

[rules]
# Override severity per rule: "error", "warn", or "allow".
"roblox::missing_strict" = "allow"
"cache::magnitude_over_squared" = "error"
"roblox::pcall_in_loop" = "warn"
```

## Rules

Rules are grouped into categories:

- **complexity** - O(n²) patterns, unnecessary work in hot loops.
- **cache** - Things you should compute once and reuse.
- **memory** - Connection leaks, untracked threads, missing cleanup.
- **roblox** - Deprecated APIs, engine-specific footguns.
- **alloc** - Heap allocations in hot paths (strings, closures, tables).
- **native** - Things that break `--!native` codegen.
- **math** - Deprecated math APIs, manual implementations of builtins.
- **string** - String allocations, deprecated patterns.
- **table** - Deprecated table APIs, O(n) shifts.
- **network** - Remote events/functions fired in loops.
- **physics** - Expensive spatial queries in loops.
- **render** - GUI/particle/beam creation in loops.
- **instance** - Instance API misuse.
- **style** - Code quality signals with perf implications.

Full list: [RULES.md](RULES.md), or run `luauperf --list-rules`.

**Severity levels:**
- `error` - Almost certainly a bug or major perf issue.
- `warn` - Probably a problem, worth looking at.
- `allow` - Off by default, turn on in config if you want them.

## Inline ignores

Suppress specific rules per-line, per-next-line, or per-file with comments:

```lua
-- Ignore specific rules on this line
local x = wait() -- luauperf-ignore: roblox::deprecated_wait

-- Ignore all rules on this line
local y = wait() -- luauperf-ignore

-- Ignore specific rules on the next line
-- luauperf-ignore-next-line: alloc::closure_in_loop
local fn = function() end

-- Ignore rules for the entire file (must be in the file header)
--!native
--!strict
-- luauperf-ignore-file: roblox::deprecated_wait, style::print_in_hot_path
```

File-level ignores must appear in the header (before any code), but can be placed after `--!native`, `--!strict`, `--!optimize` directives and other comments.

## Building from source

```bash
git clone https://github.com/bopmite/luauperf
cd luauperf
cargo build --release
# Binary at target/release/luauperf
```

## License

MIT - see [LICENSE](LICENSE).
