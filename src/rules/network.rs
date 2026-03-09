use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct FireInLoop;
pub struct InvokeServerInLoop;
pub struct LargeRemoteData;
pub struct FireClientPerPlayer;
pub struct RemoteEventStringData;
pub struct DataStoreInLoop;
pub struct DictKeysInRemoteData;
pub struct UnreliableRemotePreferred;
pub struct InvokeClientDangerous;
pub struct HttpServiceInLoop;
pub struct MarketplaceInfoInLoop;

impl Rule for FireInLoop {
    fn id(&self) -> &'static str { "network::fire_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop_direct {
                return;
            }
            let is_remote_fire = visit::is_method_call(call, "FireServer")
                || visit::is_method_call(call, "FireAllClients");
            if is_remote_fire {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "remote event fired in loop - batch into a single call".into(),
                });
            }
        });
        hits
    }
}

impl Rule for InvokeServerInLoop {
    fn id(&self) -> &'static str { "network::invoke_server_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop_direct {
                return;
            }
            if visit::is_method_call(call, "InvokeServer") || visit::is_method_call(call, "InvokeClient") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "remote function invoked in loop - yields per iteration, batch into single call".into(),
                });
            }
        });
        hits
    }
}

impl Rule for LargeRemoteData {
    fn id(&self) -> &'static str { "network::large_remote_data" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let fire_methods = [":FireServer(", ":FireClient(", ":FireAllClients(", ":InvokeServer("];
        let mut hits = Vec::new();

        for method in &fire_methods {
            for pos in visit::find_pattern_positions(source, method) {
                let after_start = pos + method.len();
                let after_end = visit::ceil_char(source, (after_start + 500).min(source.len()));
                let args = &source[after_start..after_end];

                let open_braces = args.chars().take_while(|&c| c != ')').filter(|&c| c == '{').count();
                if open_braces >= 3 {
                    hits.push(Hit {
                        pos,
                        msg: "deeply nested table in remote call - large payloads cause network lag, flatten or compress data".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for FireClientPerPlayer {
    fn id(&self) -> &'static str { "network::fire_client_per_player" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let fire_positions = visit::find_pattern_positions(source, ":FireClient(");
        for &pos in &fire_positions {
            let context_start = visit::floor_char(source, pos.saturating_sub(200));
            let context = &source[context_start..pos];
            if context.contains("GetPlayers()") || context.contains("in pairs(") || context.contains("in ipairs(") {
                let has_loop = context.contains("\nfor ") || context.trim_start().starts_with("for ");
                if has_loop {
                    hits.push(Hit {
                        pos,
                        msg: ":FireClient() in loop over players - use :FireAllClients() to send a single message".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for RemoteEventStringData {
    fn id(&self) -> &'static str { "network::remote_event_string_data" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let fire_methods = [":FireServer(", ":FireClient(", ":FireAllClients("];
        let mut hits = Vec::new();
        for method in &fire_methods {
            for pos in visit::find_pattern_positions(source, method) {
                let after_start = pos + method.len();
                let after_end = visit::ceil_char(source, (after_start + 200).min(source.len()));
                let args = &source[after_start..after_end];
                let close = args.find(')').unwrap_or(args.len());
                let arg_str = &args[..close];
                if arg_str.contains("tostring(") || arg_str.contains("string.format(") {
                    hits.push(Hit {
                        pos,
                        msg: "string conversion in remote fire args - consider sending raw values and formatting on the receiving end".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for DataStoreInLoop {
    fn id(&self) -> &'static str { "network::datastore_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop_direct {
                return;
            }
            let is_ds = visit::is_method_call(call, "GetAsync")
                || visit::is_method_call(call, "SetAsync")
                || visit::is_method_call(call, "UpdateAsync")
                || visit::is_method_call(call, "RemoveAsync")
                || visit::is_method_call(call, "IncrementAsync");
            if is_ds {
                let src = format!("{call}");
                if src.contains("DataStore") || src.contains("dataStore") || src.contains("data_store") || src.contains("store") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "DataStore operation in loop - yields per call with rate limits (60 + numPlayers*10/min), batch operations".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for DictKeysInRemoteData {
    fn id(&self) -> &'static str { "network::dict_keys_in_remote_data" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = [":FireServer({", ":FireClient(", ":FireAllClients({"];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let open = if pat.ends_with('{') { pos + pat.len() - 1 } else {
                    let rest = &source[pos + pat.len()..];
                    if let Some(p) = rest.find('{') { pos + pat.len() + p } else { continue }
                };
                let after = &source[open..visit::ceil_char_boundary(source, open + 500)];
                let has_dict_key = after.lines().next().map(|l| l.contains(" = ")).unwrap_or(false) || after[1..after.len().min(200)].contains(" = ");
                if has_dict_key {
                    let callback_check = &source[visit::floor_char_boundary(source, pos.saturating_sub(200))..pos];
                    if callback_check.contains("Heartbeat:Connect") || callback_check.contains("RenderStepped:Connect") || callback_check.contains("Stepped:Connect") {
                        hits.push(Hit {
                            pos,
                            msg: "dictionary keys in high-frequency remote data - string keys add bytes per packet, use array-indexed tables for bandwidth savings".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for UnreliableRemotePreferred {
    fn id(&self) -> &'static str { "network::unreliable_remote_preferred" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = [":FireAllClients(", ":FireClient("];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let before = &source[visit::floor_char_boundary(source, pos.saturating_sub(300))..pos];
                let is_in_heartbeat = before.contains("Heartbeat:Connect") || before.contains("RenderStepped:Connect") || before.contains("Stepped:Connect");
                if is_in_heartbeat {
                    let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let line = &source[line_start..pos];
                    if !line.contains("unreliable") && !line.contains("Unreliable") {
                        hits.push(Hit {
                            pos,
                            msg: "reliable RemoteEvent in per-frame callback - use UnreliableRemoteEvent for high-frequency updates to avoid bandwidth throttling".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for InvokeClientDangerous {
    fn id(&self) -> &'static str { "network::invoke_client_dangerous" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "InvokeClient") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":InvokeClient() yields the server thread until client responds - a malicious/lagging client can stall the server indefinitely, use FireClient instead".into(),
                });
            }
        });
        hits
    }
}

impl Rule for HttpServiceInLoop {
    fn id(&self) -> &'static str { "network::http_service_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop_direct { return; }
            let methods = ["GetAsync", "PostAsync", "RequestAsync", "JSONEncode", "JSONDecode"];
            for m in &methods {
                if visit::is_method_call(call, m) {
                    if *m == "JSONEncode" || *m == "JSONDecode" {
                        hits.push(Hit {
                            pos: visit::call_pos(call),
                            msg: format!(":{m}() in loop serializes/deserializes per iteration - cache results outside if data doesn't change"),
                        });
                    } else {
                        hits.push(Hit {
                            pos: visit::call_pos(call),
                            msg: format!(":{m}() in loop makes an HTTP request per iteration - batch requests or process asynchronously"),
                        });
                    }
                    return;
                }
            }
        });
        hits
    }
}

impl Rule for MarketplaceInfoInLoop {
    fn id(&self) -> &'static str { "network::marketplace_info_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop_direct && visit::is_method_call(call, "GetProductInfo") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":GetProductInfo() in loop makes an HTTP request per iteration - cache results in a table".into(),
                });
            }
        });
        hits
    }
}

#[cfg(test)]
#[path = "tests/network_tests.rs"]
mod tests;
