mod alloc;
mod cache;
mod complexity;
mod instance;
mod math;
mod memory;
mod native;
mod network;
mod physics;
mod render;
mod roblox;
mod string;
mod style;
mod table;

use crate::lint::{Rule, Severity};

pub fn all() -> Vec<Box<dyn Rule>> {
    vec![
        // complexity (10)
        Box::new(complexity::TableFindInLoop),
        Box::new(complexity::GetDescendantsInLoop),
        Box::new(complexity::TableRemoveShift),
        Box::new(complexity::TableSortInLoop),
        Box::new(complexity::GetTaggedInLoop),
        Box::new(complexity::GetPlayersInLoop),
        Box::new(complexity::CloneInLoop),
        Box::new(complexity::WaitForChildInLoop),
        Box::new(complexity::FindFirstChildRecursive),
        Box::new(complexity::RequireInFunction),
        // cache (15)
        Box::new(cache::MagnitudeOverSquared),
        Box::new(cache::UncachedGetService),
        Box::new(cache::TweenInfoInFunction),
        Box::new(cache::RaycastParamsInFunction),
        Box::new(cache::InstanceNewInLoop),
        Box::new(cache::CFrameNewInLoop),
        Box::new(cache::Vector3NewInLoop),
        Box::new(cache::OverlapParamsInFunction),
        Box::new(cache::NumberRangeInFunction),
        Box::new(cache::NumberSequenceInFunction),
        Box::new(cache::ColorSequenceInFunction),
        Box::new(cache::TweenCreateInLoop),
        Box::new(cache::GetAttributeInLoop),
        Box::new(cache::Color3NewInLoop),
        Box::new(cache::UDim2NewInLoop),
        // memory (7)
        Box::new(memory::UntrackedConnection),
        Box::new(memory::UntrackedTaskSpawn),
        Box::new(memory::ConnectInLoop),
        Box::new(memory::MissingPlayerRemoving),
        Box::new(memory::WhileTrueNoYield),
        Box::new(memory::ConnectInConnect),
        Box::new(memory::CharacterAddedNoCleanup),
        // roblox (16)
        Box::new(roblox::DeprecatedWait),
        Box::new(roblox::DeprecatedSpawn),
        Box::new(roblox::DebrisAddItem),
        Box::new(roblox::MissingNative),
        Box::new(roblox::DeprecatedBodyMovers),
        Box::new(roblox::PcallInLoop),
        Box::new(roblox::MissingStrict),
        Box::new(roblox::WaitForChildNoTimeout),
        Box::new(roblox::ModelSetPrimaryPartCFrame),
        Box::new(roblox::GetRankInGroupUncached),
        Box::new(roblox::InsertServiceLoadAsset),
        Box::new(roblox::DeprecatedPhysicsService),
        Box::new(roblox::SetAttributeInLoop),
        Box::new(roblox::StringValueOverAttribute),
        Box::new(roblox::TouchedEventUnfiltered),
        Box::new(roblox::DestroyChildrenManual),
        // alloc (7)
        Box::new(alloc::StringConcatInLoop),
        Box::new(alloc::StringFormatInLoop),
        Box::new(alloc::ClosureInLoop),
        Box::new(alloc::RepeatedGsub),
        Box::new(alloc::TostringInLoop),
        Box::new(alloc::TableCreatePreferred),
        Box::new(alloc::ExcessiveStringSplit),
        // network (2)
        Box::new(network::FireInLoop),
        Box::new(network::InvokeServerInLoop),
        // math (5)
        Box::new(math::RandomDeprecated),
        Box::new(math::RandomNewInLoop),
        Box::new(math::ClampManual),
        Box::new(math::SqrtOverSquared),
        Box::new(math::FloorDivision),
        // string (6)
        Box::new(string::LenOverHash),
        Box::new(string::RepInLoop),
        Box::new(string::GsubForFind),
        Box::new(string::LowerUpperInLoop),
        Box::new(string::ByteComparison),
        Box::new(string::SubForSingleChar),
        // table (6)
        Box::new(table::ForeachDeprecated),
        Box::new(table::GetnDeprecated),
        Box::new(table::MaxnDeprecated),
        Box::new(table::FreezeInLoop),
        Box::new(table::InsertWithPosition),
        Box::new(table::RemoveInIpairs),
        // native (6)
        Box::new(native::GetfenvSetfenv),
        Box::new(native::DynamicRequire),
        Box::new(native::CoroutineInNative),
        Box::new(native::MathHugeComparison),
        Box::new(native::VarargInNative),
        Box::new(native::StringPatternInNative),
        // physics (2)
        Box::new(physics::SpatialQueryInLoop),
        Box::new(physics::MoveToInLoop),
        // render (5)
        Box::new(render::GuiCreationInLoop),
        Box::new(render::BeamTrailInLoop),
        Box::new(render::ParticleEmitterInLoop),
        Box::new(render::BillboardGuiInLoop),
        Box::new(render::TransparencyChangeInLoop),
        // instance (4)
        Box::new(instance::TwoArgInstanceNew),
        Box::new(instance::PropertyChangeSignalWrong),
        Box::new(instance::ClearAllChildrenLoop),
        Box::new(instance::SetParentInLoop),
        // style (5)
        Box::new(style::ServiceLocatorAntiPattern),
        Box::new(style::EmptyFunctionBody),
        Box::new(style::DeprecatedGlobalCall),
        Box::new(style::TypeCheckInLoop),
        Box::new(style::DeepNesting),
    ]
}

pub fn print_all() {
    let rules = all();
    let mut current_cat = "";

    for rule in &rules {
        let id = rule.id();
        let cat = id.split("::").next().unwrap_or(id);
        let name = id.split("::").nth(1).unwrap_or(id);

        if cat != current_cat {
            if !current_cat.is_empty() {
                println!();
            }
            println!(" \x1b[1m{}\x1b[0m", cat);
            current_cat = cat;
        }

        let sev = match rule.severity() {
            Severity::Error => "\x1b[31merror\x1b[0m",
            Severity::Warn => "\x1b[33m warn\x1b[0m",
            Severity::Allow => " allow",
        };
        println!("   {:<42} {sev}", name);
    }

    println!();
    println!(
        " \x1b[90m{} rules across {} categories\x1b[0m",
        rules.len(),
        {
            let mut cats: Vec<&str> = rules
                .iter()
                .map(|r| r.id().split("::").next().unwrap_or(""))
                .collect();
            cats.dedup();
            cats.len()
        }
    );
}
