# luau-perf

A static performance linter for Luau.

## What it does

Scans your `.lua` / `.luau` files and flags performance antipatterns - allocations in loops, untracked connections (memory leaks), deprecated APIs, missing `--!native` headers, and things that prevent the Luau VM from using FASTCALL/GETIMPORT optimizations.

Not a type checker. Not a formatter. Just perf.

## Install

```bash
cargo install --path .
```

Or grab a binary from releases.

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

Drop a `luauperf.toml` in your project root:

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

## How it works

Parses Luau with [full_moon](https://github.com/Kampfkarren/full-moon), walks the AST with a visitor that tracks loop and function depth. Some rules use source text matching for patterns that are easier to detect outside the AST. File processing is parallelized with Rayon.

No LSP, no daemon, no background process. Runs, prints, exits.

## CI

```yaml
- name: Perf lint
  run: luauperf src/ --format json > perf-lint.json
```

Exits 1 if any `error` severity issues are found.

## Building from source

```bash
git clone https://github.com/bopmite/luauperf
cd luauperf
cargo build --release
# Binary at target/release/luauperf
```

## License

MIT - see [LICENSE](LICENSE).
