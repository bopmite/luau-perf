use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct FireInLoop;
pub struct InvokeServerInLoop;
pub struct LargeRemoteData;

impl Rule for FireInLoop {
    fn id(&self) -> &'static str { "network::fire_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_remote_fire = visit::is_method_call(call, "FireServer")
                || visit::is_method_call(call, "FireClient")
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
            if !ctx.in_loop {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::Rule;

    fn parse(src: &str) -> full_moon::ast::Ast {
        full_moon::parse(src).unwrap()
    }

    #[test]
    fn fire_in_loop_detected() {
        let src = "for _, player in players do\n  remote:FireClient(player, data)\nend";
        let ast = parse(src);
        let hits = FireInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn fire_outside_loop_ok() {
        let src = "remote:FireServer(data)";
        let ast = parse(src);
        let hits = FireInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn invoke_in_loop_detected() {
        let src = "for i = 1, 10 do\n  remote:InvokeServer(i)\nend";
        let ast = parse(src);
        let hits = InvokeServerInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn invoke_outside_loop_ok() {
        let src = "local result = remote:InvokeServer(data)";
        let ast = parse(src);
        let hits = InvokeServerInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn large_remote_data_detected() {
        let src = "remote:FireServer({a = {b = {c = 1}}})";
        let ast = parse(src);
        let hits = LargeRemoteData.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn simple_remote_data_ok() {
        let src = "remote:FireServer(\"hello\")";
        let ast = parse(src);
        let hits = LargeRemoteData.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
