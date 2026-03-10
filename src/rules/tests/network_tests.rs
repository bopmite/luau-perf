use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn fire_in_loop_detected() {
    let src = "for _, player in players do\n  remote:FireServer(data)\nend";
    let ast = parse(src);
    let hits = FireInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn fire_client_in_loop_ok() {
    let src = "for _, player in players do\n  remote:FireClient(player, data)\nend";
    let ast = parse(src);
    let hits = FireInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
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

#[test]
fn fire_client_per_player_detected() {
    let src = "for _, player in pairs(Players:GetPlayers()) do\n  remote:FireClient(player, data)\nend";
    let ast = parse(src);
    let hits = FireClientPerPlayer.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn fire_client_single_ok() {
    let src = "remote:FireClient(player, data)";
    let ast = parse(src);
    let hits = FireClientPerPlayer.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn datastore_in_loop_detected() {
    let src = "for _, key in keys do\n  local data = dataStore:GetAsync(key)\nend";
    let ast = parse(src);
    let hits = DataStoreInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn datastore_outside_loop_ok() {
    let src = "local data = dataStore:GetAsync(key)";
    let ast = parse(src);
    let hits = DataStoreInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn invoke_client_detected() {
    let src = "remote:InvokeClient(player, data)";
    let ast = parse(src);
    let hits = InvokeClientDangerous.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn fire_client_ok() {
    let src = "remote:FireClient(player, data)";
    let ast = parse(src);
    let hits = InvokeClientDangerous.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn http_service_in_loop_detected() {
    let src = "for _, url in urls do\n  local res = http:GetAsync(url)\nend";
    let ast = parse(src);
    let hits = HttpServiceInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn http_service_outside_loop_ok() {
    let src = "local res = http:GetAsync(url)";
    let ast = parse(src);
    let hits = HttpServiceInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn marketplace_in_loop_detected() {
    let src = "for _, id in ids do\n  local info = marketplace:GetProductInfo(id)\nend";
    let ast = parse(src);
    let hits = MarketplaceInfoInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn marketplace_outside_loop_ok() {
    let src = "local info = marketplace:GetProductInfo(id)";
    let ast = parse(src);
    let hits = MarketplaceInfoInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn json_decode_in_for_in_ok() {
    let src = "for _, raw in responses do\n  local data = http:JSONDecode(raw)\nend";
    let ast = parse(src);
    let hits = HttpServiceInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn json_encode_in_hot_loop_detected() {
    let src = "while true do\n  local s = http:JSONEncode(data)\nend";
    let ast = parse(src);
    let hits = HttpServiceInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn json_deep_clone_detected() {
    let src = "local copy = HttpService:JSONDecode(HttpService:JSONEncode(data))";
    let ast = parse(src);
    let hits = JsonDeepClone.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn json_deep_clone_short_var_detected() {
    let src = "local copy = http:JSONDecode(http:JSONEncode(data))";
    let ast = parse(src);
    let hits = JsonDeepClone.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn json_decode_alone_ok() {
    let src = "local data = HttpService:JSONDecode(jsonStr)";
    let ast = parse(src);
    let hits = JsonDeepClone.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn json_encode_alone_ok() {
    let src = "local str = HttpService:JSONEncode(data)";
    let ast = parse(src);
    let hits = JsonDeepClone.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
