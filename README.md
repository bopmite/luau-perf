# luperf

Static performance analyzer for Luau. Catches perf anti-patterns before they hit production.

## Install

```
cargo install --path .
```

## Usage

```
luperf src/
luperf src/Server/Services/
luperf path/to/file.luau
luperf src/ --format json
luperf --list-rules
luperf --init
```

## Rules

| Rule | Default | What it catches |
|------|---------|-----------------|
| `complexity::table_find_in_loop` | deny | `table.find()` in loops — O(n) per iteration |
| `complexity::get_descendants_in_loop` | warn | `GetDescendants`/`GetChildren`/`FindFirstChild` in loops |
| `complexity::table_remove_shift` | warn | `table.remove(t, 1)` — O(n) shift |
| `cache::magnitude_over_squared` | warn | `.Magnitude` — uses sqrt, compare squared instead |
| `cache::uncached_get_service` | warn | `game:GetService()` inside function body |
| `cache::tween_info_in_function` | warn | `TweenInfo.new()` inside function — cache at module level |
| `cache::raycast_params_in_function` | warn | `RaycastParams.new()` inside function — cache and reuse |
| `cache::instance_new_in_loop` | warn | `Instance.new()` in loop — pre-allocate or Clone |
| `memory::untracked_connection` | deny | `:Connect()` result not stored |
| `memory::untracked_task_spawn` | warn | `task.spawn`/`task.delay` not tracked |
| `roblox::deprecated_wait` | deny | `wait()` — use `task.wait()` |
| `roblox::deprecated_spawn` | deny | `spawn()`/`delay()` — use `task.*` |
| `roblox::debris_add_item` | warn | `Debris:AddItem()` — use `task.delay` + `Destroy()` |
| `roblox::missing_native` | warn | Missing `--!native` header |
| `alloc::string_concat_in_loop` | warn | `..` in loops — use `table.concat` |
| `network::fire_in_loop` | deny | Remote events fired in loops — batch them |

## Config

Create `luperf.toml` in your project root (`luperf --init`):

```toml
[rules]
memory::untracked_task_spawn = "allow"
cache::magnitude_over_squared = "deny"

exclude = ["Packages/", "Generated/"]
```

## License

MIT
