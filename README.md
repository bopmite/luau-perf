# luauperf

static performance linter for luau. catches the shit that makes your roblox game run at 12fps.

## what it does

scans your `.lua` / `.luau` files and flags performance antipatterns — allocations in loops, untracked connections (memory leaks), deprecated APIs, missing `--!native` headers, and things that prevent the luau VM from using FASTCALL/GETIMPORT optimizations.

not a type checker. not a formatter. just perf.

## install

```bash
cargo install --path .
```

or grab a binary from releases.

## usage

```bash
# lint a directory
luauperf src/

# lint a single file
luauperf src/Server/Services/GunService.luau

# json output (for CI / editor integration)
luauperf src/ --format json

# see all available rules
luauperf --list-rules

# generate config
luauperf --init
```

## config

drop a `luauperf.toml` in your project root:

```toml
[rules]
# override severity per rule: "error", "warn", or "allow"
roblox::missing_strict = "allow"
cache::magnitude_over_squared = "error"
roblox::pcall_in_loop = "warn"

# exclude paths (substring match)
exclude = ["Packages/", "Generated/", "node_modules/"]
```

## rules

rules are grouped into categories:

- **complexity** — O(n²) patterns, unnecessary work in hot loops
- **cache** — things you should compute once and reuse
- **memory** — connection leaks, untracked threads, missing cleanup
- **roblox** — deprecated APIs, engine-specific footguns
- **alloc** — heap allocations in hot paths (strings, closures, tables)
- **native** — things that break `--!native` codegen
- **math** — deprecated math APIs, manual implementations of builtins
- **string** — string allocations, deprecated patterns
- **table** — deprecated table APIs, O(n) shifts
- **network** — remote events/functions fired in loops
- **physics** — expensive spatial queries in loops
- **render** — GUI/particle/beam creation in loops
- **instance** — Instance API misuse
- **style** — code quality signals with perf implications

full list: [RULES.md](RULES.md). or just run `luauperf --list-rules`.

**severity levels:**
- `error` — almost certainly a bug or major perf issue
- `warn` — probably a problem, worth looking at
- `allow` — off by default, turn on in config if you want

## how it works

parses luau with [full_moon](https://github.com/Kampfkarren/full-moon), walks the AST with a visitor that tracks loop/function depth. some rules use source text matching for patterns easier to detect outside the AST. parallel file processing via rayon.

no LSP, no daemon, no background process. runs, prints, exits.

## ci

```yaml
- name: perf lint
  run: luauperf src/ --format json > perf-lint.json
```

exits 1 if any `error` severity issues found.

## building from source

```bash
git clone https://github.com/bopmite/luauperf
cd luauperf
cargo build --release
# binary at target/release/luauperf
```

## license

MIT — see [LICENSE](LICENSE)
