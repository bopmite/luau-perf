mod alloc;
mod cache;
mod complexity;
mod memory;
mod network;
mod roblox;

use crate::lint::{Rule, Severity};

pub fn all() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(complexity::TableFindInLoop),
        Box::new(complexity::GetDescendantsInLoop),
        Box::new(complexity::TableRemoveShift),
        Box::new(cache::MagnitudeOverSquared),
        Box::new(cache::UncachedGetService),
        Box::new(cache::TweenInfoInFunction),
        Box::new(cache::RaycastParamsInFunction),
        Box::new(cache::InstanceNewInLoop),
        Box::new(memory::UntrackedConnection),
        Box::new(memory::UntrackedTaskSpawn),
        Box::new(roblox::DeprecatedWait),
        Box::new(roblox::DeprecatedSpawn),
        Box::new(roblox::DebrisAddItem),
        Box::new(roblox::MissingNative),
        Box::new(alloc::StringConcatInLoop),
        Box::new(network::FireInLoop),
    ]
}

pub fn print_all() {
    for rule in all() {
        let sev = match rule.severity() {
            Severity::Deny => "\x1b[31mdeny\x1b[0m",
            Severity::Warn => "\x1b[33mwarn\x1b[0m",
            Severity::Allow => "allow",
        };
        println!("  {:<45} [{sev}]", rule.id());
    }
}
