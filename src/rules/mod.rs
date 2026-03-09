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
        // complexity
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
        Box::new(complexity::DeepMetatableChain),
        Box::new(complexity::PairsInPairs),
        Box::new(complexity::GmatchInLoop),
        Box::new(complexity::DataStoreNoPcall),
        Box::new(complexity::AccumulatingRebuild),
        Box::new(complexity::OneIterationLoop),
        Box::new(complexity::ElseifChainOverTable),
        Box::new(complexity::FilterThenFirst),
        Box::new(complexity::NestedTableFind),
        Box::new(complexity::StringMatchInLoop),
        Box::new(complexity::PromiseChainInLoop),
        Box::new(complexity::RepeatedTypeof),
        // cache
        Box::new(cache::MagnitudeOverSquared),
        Box::new(cache::UncachedGetService),
        Box::new(cache::TweenInfoInFunction),
        Box::new(cache::RaycastParamsInFunction),
        Box::new(cache::InstanceNewInLoop),
        Box::new(cache::CFrameNewInLoop),
        Box::new(cache::Vector3NewInLoop),
        Box::new(cache::Vector2NewInLoop),
        Box::new(cache::OverlapParamsInFunction),
        Box::new(cache::NumberRangeInFunction),
        Box::new(cache::NumberSequenceInFunction),
        Box::new(cache::ColorSequenceInFunction),
        Box::new(cache::TweenCreateInLoop),
        Box::new(cache::GetAttributeInLoop),
        Box::new(cache::Color3NewInLoop),
        Box::new(cache::UDim2NewInLoop),
        Box::new(cache::RepeatedMethodCall),
        Box::new(cache::CurrentCameraUncached),
        Box::new(cache::LocalPlayerUncached),
        Box::new(cache::WorkspaceLookupInLoop),
        Box::new(cache::RepeatedColor3),
        Box::new(cache::EnumLookupInLoop),
        Box::new(cache::BrickColorNewInLoop),
        Box::new(cache::RegionNewInLoop),
        Box::new(cache::RepeatedPropertyChain),
        // memory
        Box::new(memory::UntrackedConnection),
        Box::new(memory::UntrackedTaskSpawn),
        Box::new(memory::ConnectInLoop),
        Box::new(memory::MissingPlayerRemoving),
        Box::new(memory::WhileTrueNoYield),
        Box::new(memory::ConnectInConnect),
        Box::new(memory::CharacterAddedNoCleanup),
        Box::new(memory::HeartbeatAllocation),
        Box::new(memory::CircularConnectionRef),
        Box::new(memory::WeakTableNoShrink),
        Box::new(memory::RunServiceNoDisconnect),
        Box::new(memory::TaskDelayLongDuration),
        Box::new(memory::TweenCompletedConnect),
        Box::new(memory::SetAttributeInHeartbeat),
        Box::new(memory::SoundNotDestroyed),
        Box::new(memory::UnboundedTableGrowth),
        Box::new(memory::DebrisNegativeDuration),
        Box::new(memory::CollectionTagNoCleanup),
        Box::new(memory::AttributeChangedInLoop),
        // roblox
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
        Box::new(roblox::MissingOptimize),
        Box::new(roblox::DeprecatedRegion3),
        Box::new(roblox::BindableSameScript),
        Box::new(roblox::ServerPropertyInHeartbeat),
        Box::new(roblox::GameLoadedRace),
        Box::new(roblox::HumanoidStatePolling),
        Box::new(roblox::ServerSideTween),
        Box::new(roblox::RequireInConnect),
        Box::new(roblox::FindFirstChildChain),
        Box::new(roblox::OnceOverConnect),
        Box::new(roblox::HealthPolling),
        Box::new(roblox::ChangedEventUnfiltered),
        Box::new(roblox::DescendantEventWorkspace),
        Box::new(roblox::GetAttributeInHeartbeat),
        Box::new(roblox::PivotToInLoop),
        Box::new(roblox::DeprecatedTick),
        Box::new(roblox::DeprecatedFindPartOnRay),
        Box::new(roblox::WhileWaitDo),
        Box::new(roblox::GetPropertyChangedInLoop),
        Box::new(roblox::RenderSteppedOnServer),
        Box::new(roblox::TaskWaitNoArg),
        Box::new(roblox::DeprecatedDelay),
        Box::new(roblox::CloneSetParent),
        Box::new(roblox::YieldInConnectCallback),
        Box::new(roblox::DeprecatedUdim),
        Box::new(roblox::TeleportServiceRace),
        Box::new(roblox::Color3NewMisuse),
        Box::new(roblox::RaycastFilterDeprecated),
        Box::new(roblox::PlayerAddedRace),
        Box::new(roblox::GameWorkspace),
        Box::new(roblox::CoroutineResumeCreate),
        Box::new(roblox::CharacterAddedNoWait),
        Box::new(roblox::GetServiceWorkspace),
        Box::new(roblox::FindFirstChildNoCheck),
        Box::new(roblox::GetFullNameInLoop),
        Box::new(roblox::BindToRenderStepNoCleanup),
        Box::new(roblox::CFrameOldConstructor),
        // alloc
        Box::new(alloc::StringConcatInLoop),
        Box::new(alloc::StringFormatInLoop),
        Box::new(alloc::ClosureInLoop),
        Box::new(alloc::RepeatedGsub),
        Box::new(alloc::TostringInLoop),
        Box::new(alloc::TableCreatePreferred),
        Box::new(alloc::ExcessiveStringSplit),
        Box::new(alloc::CoroutineWrapInLoop),
        Box::new(alloc::TableCreateForDict),
        Box::new(alloc::MutableUpvalueClosure),
        Box::new(alloc::UnpackInLoop),
        Box::new(alloc::RepeatedStringByte),
        Box::new(alloc::StringInterpInLoop),
        Box::new(alloc::SelectInLoop),
        Box::new(alloc::TableInsertKnownSize),
        Box::new(alloc::BufferOverStringPack),
        Box::new(alloc::TaskSpawnInLoop),
        Box::new(alloc::GsubFunctionInLoop),
        Box::new(alloc::TypeofInLoop),
        Box::new(alloc::SetmetatableInLoop),
        // network
        Box::new(network::FireInLoop),
        Box::new(network::InvokeServerInLoop),
        Box::new(network::LargeRemoteData),
        Box::new(network::FireClientPerPlayer),
        Box::new(network::RemoteEventStringData),
        Box::new(network::DataStoreInLoop),
        Box::new(network::DictKeysInRemoteData),
        Box::new(network::UnreliableRemotePreferred),
        Box::new(network::InvokeClientDangerous),
        Box::new(network::HttpServiceInLoop),
        Box::new(network::MarketplaceInfoInLoop),
        // math
        Box::new(math::RandomDeprecated),
        Box::new(math::RandomNewInLoop),
        Box::new(math::ClampManual),
        Box::new(math::SqrtOverSquared),
        Box::new(math::FloorDivision),
        Box::new(math::FmodOverModulo),
        Box::new(math::PowTwo),
        Box::new(math::VectorNormalizeManual),
        Box::new(math::UnnecessaryTonumber),
        Box::new(math::LerpManual),
        Box::new(math::AbsForSignCheck),
        Box::new(math::Vector3ZeroConstant),
        Box::new(math::Vector2ZeroConstant),
        Box::new(math::CFrameIdentityConstant),
        Box::new(math::HugeComparison),
        Box::new(math::ExpOverPow),
        Box::new(math::FloorRoundManual),
        Box::new(math::MaxMinSingleArg),
        Box::new(math::PowSlowExponent),
        // string
        Box::new(string::LenOverHash),
        Box::new(string::RepInLoop),
        Box::new(string::GsubForFind),
        Box::new(string::LowerUpperInLoop),
        Box::new(string::ByteComparison),
        Box::new(string::SubForSingleChar),
        Box::new(string::TostringOnString),
        Box::new(string::FindMissingPlainFlag),
        Box::new(string::LowerForComparison),
        Box::new(string::MatchForBoolean),
        Box::new(string::ConcatChain),
        Box::new(string::SubForPrefixCheck),
        Box::new(string::PatternBacktracking),
        Box::new(string::ReverseInLoop),
        Box::new(string::FormatKnownTypes),
        Box::new(string::FormatNoArgs),
        Box::new(string::FormatRedundantTostring),
        Box::new(string::FormatSimpleConcat),
        // table
        Box::new(table::ForeachDeprecated),
        Box::new(table::GetnDeprecated),
        Box::new(table::MaxnDeprecated),
        Box::new(table::FreezeInLoop),
        Box::new(table::InsertWithPosition),
        Box::new(table::RemoveInIpairs),
        Box::new(table::PackOverLiteral),
        Box::new(table::ManualCopyLoop),
        Box::new(table::DeferredFieldAssignment),
        Box::new(table::IpairsOverNumericFor),
        Box::new(table::PolymorphicConstructor),
        Box::new(table::SortComparisonAllocation),
        Box::new(table::ClearVsNew),
        Box::new(table::TableMoveOverLoop),
        Box::new(table::ConcatWithSeparatorLoop),
        Box::new(table::PairsOverGeneralized),
        Box::new(table::NilFieldInConstructor),
        Box::new(table::RawsetInLoop),
        Box::new(table::NextTNilOverPairs),
        // native
        Box::new(native::GetfenvSetfenv),
        Box::new(native::DynamicRequire),
        Box::new(native::CoroutineInNative),
        Box::new(native::MathHugeComparison),
        Box::new(native::VarargInNative),
        Box::new(native::StringPatternInNative),
        Box::new(native::LoadstringDeopt),
        Box::new(native::UntypedParams),
        Box::new(native::HeavyApiScript),
        Box::new(native::LargeTableLiteral),
        Box::new(native::MixedComputationApi),
        Box::new(native::GlobalWrite),
        Box::new(native::ShadowedBuiltin),
        Box::new(native::TableZeroIndex),
        Box::new(native::MethodCallDefeatsFastcall),
        Box::new(native::SharedGlobalMutation),
        Box::new(native::ImportChainTooDeep),
        Box::new(native::PcallInNative),
        Box::new(native::DynamicTableKeyInNative),
        Box::new(native::NonFastcallInHotLoop),
        // physics
        Box::new(physics::SpatialQueryInLoop),
        Box::new(physics::MoveToInLoop),
        Box::new(physics::TouchedWithoutDebounce),
        Box::new(physics::SetNetworkOwnerInLoop),
        Box::new(physics::PreciseCollisionFidelity),
        Box::new(physics::CollisionGroupStringInLoop),
        Box::new(physics::AnchoredWithVelocity),
        Box::new(physics::RaycastParamsInLoop),
        Box::new(physics::CFrameAssignInLoop),
        Box::new(physics::CanTouchQueryNotDisabled),
        Box::new(physics::WeldConstraintInLoop),
        Box::new(physics::MasslessNotSet),
        Box::new(physics::AssemblyVelocityInLoop),
        Box::new(physics::SpatialQueryPerFrame),
        // render
        Box::new(render::GuiCreationInLoop),
        Box::new(render::BeamTrailInLoop),
        Box::new(render::ParticleEmitterInLoop),
        Box::new(render::BillboardGuiInLoop),
        Box::new(render::TransparencyChangeInLoop),
        Box::new(render::RichTextInLoop),
        Box::new(render::NeonGlassMaterialInLoop),
        Box::new(render::SurfaceGuiInLoop),
        Box::new(render::ImageLabelInLoop),
        Box::new(render::ScrollingFrameInLoop),
        // instance
        Box::new(instance::TwoArgInstanceNew),
        Box::new(instance::PropertyChangeSignalWrong),
        Box::new(instance::ClearAllChildrenLoop),
        Box::new(instance::SetParentInLoop),
        Box::new(instance::PropertyBeforeParent),
        Box::new(instance::RepeatedFindFirstChild),
        Box::new(instance::ChangedOnMovingPart),
        Box::new(instance::BulkPropertySet),
        Box::new(instance::CollectionServiceInLoop),
        Box::new(instance::NameIndexingInLoop),
        Box::new(instance::DestroyInLoop),
        Box::new(instance::GetChildrenInLoop),
        // style
        Box::new(style::ServiceLocatorAntiPattern),
        Box::new(style::EmptyFunctionBody),
        Box::new(style::DeprecatedGlobalCall),
        Box::new(style::TypeCheckInLoop),
        Box::new(style::DeepNesting),
        Box::new(style::DotMethodCall),
        Box::new(style::PrintInHotPath),
        Box::new(style::DebugInHotPath),
        Box::new(style::IndexFunctionMetatable),
        Box::new(style::ConditionalFieldInConstructor),
        Box::new(style::GlobalFunctionNotLocal),
        Box::new(style::AssertInHotPath),
        Box::new(style::RedundantCondition),
        Box::new(style::LongFunctionBody),
        Box::new(style::DuplicateStringLiteral),
        Box::new(style::TypeOverTypeof),
        Box::new(style::NestedTernary),
        Box::new(style::UnusedVariable),
        Box::new(style::MultipleReturns),
        Box::new(style::UDim2PreferFromOffset),
        Box::new(style::UDim2PreferFromScale),
        Box::new(style::TostringMathFloor),
        Box::new(style::DeepParentChain),
        Box::new(style::ErrorNoLevel),
        Box::new(style::MatchForExistence),
        Box::new(style::NestedStringFormat),
    ]
}

pub fn print_all() {
    let rules = all();
    let mut current_cat = "";
    let mut cat_count = 0u32;

    let mut cat_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for rule in &rules {
        let cat = rule.id().split("::").next().unwrap_or("");
        *cat_counts.entry(cat).or_insert(0) += 1;
    }

    for rule in &rules {
        let id = rule.id();
        let cat = id.split("::").next().unwrap_or(id);
        let name = id.split("::").nth(1).unwrap_or(id);

        if cat != current_cat {
            if !current_cat.is_empty() {
                println!();
            }
            let count = cat_counts.get(cat).copied().unwrap_or(0);
            println!(" \x1b[1m{}\x1b[0m \x1b[90m({})\x1b[0m", cat, count);
            current_cat = cat;
            cat_count += 1;
        }

        let sev = match rule.severity() {
            Severity::Error => "\x1b[31merror\x1b[0m",
            Severity::Warn => "\x1b[33m warn\x1b[0m",
            Severity::Allow => " allow",
        };
        let lvl = match rule_level(id) {
            crate::lint::Level::Default => "\x1b[32mdefault\x1b[0m ",
            crate::lint::Level::Strict => "\x1b[36m strict\x1b[0m ",
            crate::lint::Level::Pedantic => "\x1b[90mpedantic\x1b[0m",
        };
        println!("   {:<42} {sev}  {lvl}", name);
    }

    println!();
    println!(
        " \x1b[90m{} rules across {} categories\x1b[0m",
        rules.len(),
        cat_count,
    );
}

pub fn explain(rule_id: &str) {
    let rules = all();
    let rule = rules.iter().find(|r| r.id() == rule_id);
    match rule {
        Some(r) => {
            let sev = match r.severity() {
                Severity::Error => "error",
                Severity::Warn => "warn",
                Severity::Allow => "allow",
            };
            println!("\n \x1b[1m{}\x1b[0m", r.id());
            let lvl = match rule_level(r.id()) {
                crate::lint::Level::Default => "default",
                crate::lint::Level::Strict => "strict",
                crate::lint::Level::Pedantic => "pedantic",
            };
            println!(" severity: {sev}");
            println!(" level:    {lvl}");
            println!(" fixable:  {}", if is_fixable(r.id()) { "yes (--fix)" } else { "no" });
            println!();
            println!(" {}", explain_text(r.id()));
            println!();
        }
        None => {
            eprintln!("\x1b[31merror\x1b[0m: unknown rule '{rule_id}'");
            eprintln!("Run --list-rules to see all available rules.");
        }
    }
}

pub fn rule_level(id: &str) -> crate::lint::Level {
    use crate::lint::Level;
    match id {
        // === DEFAULT: Bugs, deprecated APIs, patterns that are always wrong ===

        // memory bugs
        "memory::untracked_connection"
        | "memory::connect_in_loop"
        | "memory::missing_player_removing"
        | "memory::while_true_no_yield"
        | "memory::runservice_no_disconnect"
        | "memory::connect_in_connect"
        | "memory::character_added_no_cleanup"

        // deprecated APIs
        | "roblox::deprecated_wait"
        | "roblox::deprecated_spawn"
        | "roblox::deprecated_body_movers"
        | "roblox::model_set_primary_part_cframe"
        | "roblox::deprecated_physics_service"
        | "roblox::deprecated_region3"
        | "math::random_deprecated"
        | "table::foreach_deprecated"
        | "table::getn_deprecated"
        | "table::maxn_deprecated"

        // correctness / race conditions
        | "roblox::game_loaded_race"
        | "roblox::wait_for_child_no_timeout"
        | "table::remove_in_ipairs"
        | "complexity::datastore_no_pcall"

        // critical perf (always wrong)
        | "instance::two_arg_instance_new"
        | "network::fire_in_loop"
        | "network::invoke_server_in_loop"
        | "network::datastore_in_loop"
        | "complexity::table_find_in_loop"

        // deopt killers (entire script breaks)
        | "native::getfenv_setfenv"
        | "native::loadstring_deopt"
        | "native::global_write"
        | "native::shared_global_mutation"

        // more deprecated APIs
        | "roblox::deprecated_tick"
        | "roblox::deprecated_find_part_on_ray"

        // dangerous patterns
        | "network::invoke_client_dangerous"

        // more deprecated
        | "roblox::deprecated_delay"

        // correctness
        | "roblox::render_stepped_on_server"
        | "memory::debris_negative_duration"
        | "roblox::color3_new_misuse"
        | "roblox::raycast_filter_deprecated"
        | "roblox::coroutine_resume_create"
        | "math::max_min_single_arg"
        => Level::Default,

        // === STRICT: Optimization suggestions worth fixing ===

        // allocation in loops
        "alloc::string_concat_in_loop"
        | "alloc::string_format_in_loop"
        | "alloc::closure_in_loop"
        | "alloc::coroutine_wrap_in_loop"
        | "alloc::excessive_string_split"
        | "alloc::string_interp_in_loop"
        | "alloc::unpack_in_loop"
        | "alloc::tostring_in_loop"
        | "alloc::repeated_gsub"
        | "alloc::table_create_for_dict"
        | "alloc::task_spawn_in_loop"
        | "alloc::gsub_function_in_loop"
        | "alloc::setmetatable_in_loop"
        | "string::reverse_in_loop"

        // caching suggestions
        | "cache::uncached_get_service"
        | "cache::instance_new_in_loop"
        | "cache::tween_create_in_loop"
        | "cache::get_attribute_in_loop"
        | "cache::workspace_lookup_in_loop"
        | "cache::region_new_in_loop"
        | "cache::raycast_params_in_function"
        | "cache::overlap_params_in_function"
        | "cache::tween_info_in_function"
        | "cache::magnitude_over_squared"
        | "cache::current_camera_uncached"
        | "cache::local_player_uncached"

        // complexity
        | "complexity::filter_then_first"
        | "complexity::nested_table_find"
        | "complexity::string_match_in_loop"
        | "complexity::promise_chain_in_loop"
        | "complexity::accumulating_rebuild"
        | "complexity::one_iteration_loop"
        | "complexity::get_descendants_in_loop"
        | "complexity::table_remove_shift"
        | "complexity::table_sort_in_loop"
        | "complexity::get_tagged_in_loop"
        | "complexity::get_players_in_loop"
        | "complexity::clone_in_loop"
        | "complexity::wait_for_child_in_loop"
        | "complexity::find_first_child_recursive"
        | "complexity::pairs_in_pairs"

        // memory (important but not always bugs)
        | "memory::untracked_task_spawn"
        | "memory::heartbeat_allocation"
        | "memory::circular_connection_ref"
        | "memory::tween_completed_connect"
        | "memory::set_attribute_in_heartbeat"
        | "memory::sound_not_destroyed"
        | "memory::unbounded_table_growth"
        | "memory::collection_tag_no_cleanup"
        | "memory::attribute_changed_in_loop"

        // network
        // network (important)
        | "network::http_service_in_loop"
        | "network::marketplace_info_in_loop"
        | "network::fire_client_per_player"

        // instance
        | "instance::property_change_signal_wrong"
        | "instance::clear_all_children_loop"
        | "instance::set_parent_in_loop"
        | "instance::property_before_parent"
        | "instance::repeated_find_first_child"
        | "instance::changed_on_moving_part"
        | "instance::collection_service_in_loop"
        | "instance::destroy_in_loop"
        | "instance::get_children_in_loop"

        // physics
        | "physics::spatial_query_in_loop"
        | "physics::move_to_in_loop"
        | "physics::touched_without_debounce"
        | "physics::set_network_owner_in_loop"
        | "physics::raycast_params_in_loop"
        | "physics::cframe_assign_in_loop"
        | "physics::weld_constraint_in_loop"
        | "physics::assembly_velocity_in_loop"
        | "physics::spatial_query_per_frame"

        // render
        | "render::gui_creation_in_loop"
        | "render::beam_trail_in_loop"
        | "render::particle_emitter_in_loop"
        | "render::billboard_gui_in_loop"
        | "render::surface_gui_in_loop"
        | "render::image_label_in_loop"
        | "render::scrolling_frame_in_loop"

        // roblox
        | "roblox::debris_add_item"
        | "roblox::set_attribute_in_loop"
        | "roblox::string_value_over_attribute"
        | "roblox::touched_event_unfiltered"
        | "roblox::destroy_children_manual"
        | "roblox::server_property_in_heartbeat"
        | "roblox::humanoid_state_polling"
        | "roblox::require_in_connect"
        | "roblox::find_first_child_chain"
        | "roblox::health_polling"
        | "roblox::changed_event_unfiltered"
        | "roblox::descendant_event_workspace"
        | "roblox::get_attribute_in_heartbeat"
        | "roblox::pivot_to_in_loop"
        | "roblox::while_wait_do"
        | "roblox::get_property_changed_in_loop"
        | "roblox::clone_set_parent"
        | "roblox::yield_in_connect_callback"
        | "roblox::teleport_service_race"
        | "roblox::player_added_race"
        | "roblox::character_added_no_wait"
        | "roblox::find_first_child_no_check"
        | "roblox::bind_to_render_step_no_cleanup"
        | "string::format_redundant_tostring"

        // native
        | "native::dynamic_require"
        | "native::shadowed_builtin"
        | "native::table_zero_index"
        | "native::pcall_in_native"

        // style with real perf impact
        | "style::duplicate_get_service"
        | "style::dot_method_call"
        | "style::print_in_hot_path"
        | "style::debug_in_hot_path"
        | "style::index_function_metatable"
        | "style::redundant_condition"

        // string
        | "string::len_over_hash"
        | "string::rep_in_loop"
        | "string::gsub_for_find"
        | "string::lower_upper_in_loop"
        | "string::tostring_on_string"
        | "string::pattern_backtracking"

        // table
        | "table::freeze_in_loop"
        | "table::insert_with_position"
        | "table::pack_over_literal"
        | "table::manual_copy_loop"
        | "table::concat_with_separator_loop"

        // math
        | "math::random_new_in_loop"
        | "math::clamp_manual"
        | "math::sqrt_over_squared"
        | "math::floor_division"
        | "math::fmod_over_modulo"
        | "math::vector_normalize_manual"
        | "math::unnecessary_tonumber"
        => Level::Strict,

        // === PEDANTIC: Everything else (micro-opts, style, situational) ===
        _ => Level::Pedantic,
    }
}

fn is_fixable(id: &str) -> bool {
    matches!(id,
        "roblox::deprecated_wait" | "roblox::deprecated_spawn" |
        "roblox::missing_native" | "roblox::missing_strict" |
        "math::floor_division" | "string::len_over_hash" |
        "table::getn_deprecated" | "math::fmod_over_modulo" |
        "roblox::missing_optimize" | "table::foreach_deprecated" |
        "table::maxn_deprecated" |
        "style::udim2_prefer_from_offset" | "style::udim2_prefer_from_scale" |
        "math::vector3_zero_constant" | "math::vector2_zero_constant" |
        "math::cframe_identity_constant" |
        "roblox::color3_new_misuse" | "roblox::raycast_filter_deprecated" |
        "roblox::getservice_workspace" | "math::floor_round_manual"
    )
}

fn explain_text(id: &str) -> &'static str {
    match id {
        // alloc
        "alloc::closure_in_loop" => "Creating a function() inside a loop allocates a new closure object each iteration via NEWCLOSURE. Extract the function outside the loop. If the closure captures no mutable upvalues, Luau can use DUPCLOSURE (free) instead.",
        "alloc::string_concat_in_loop" => "String concatenation (..) in a loop allocates a new string each iteration. Luau strings are immutable, so each .. creates a copy. Use table.insert + table.concat() or the buffer library instead.",
        "alloc::string_format_in_loop" => "string.format() allocates a new string each call. In a loop, this creates N temporary strings. If the format string is constant, consider building results with table.concat or buffer.",
        "alloc::repeated_gsub" => "Each :gsub() call allocates a new string. Chaining N gsub calls creates N intermediate strings. Consider combining patterns with alternation (pat1|pat2) or using the buffer library.",
        "alloc::tostring_in_loop" => "tostring() allocates a new string each call. In a loop, this creates N temporary strings. Cache the result outside the loop if the value doesn't change.",
        "alloc::table_create_preferred" => "Using {} in a loop allocates a new table each iteration without size hints. If you know the array size, table.create(n) pre-allocates the array part, avoiding incremental resizing.",
        "alloc::excessive_string_split" => "string.split() allocates a new table of substrings each call. In a loop, this creates N tables. Split once outside the loop and reuse the result.",
        "alloc::coroutine_wrap_in_loop" => "coroutine.wrap() allocates a new coroutine each call (~200x slower than creating a closure). In a loop, this creates N coroutines. Use a regular function or extract the coroutine outside.",
        "alloc::table_create_for_dict" => "table.create(n) only pre-allocates the array part of a table. If you then assign string keys (t.x = ...), the hash part grows dynamically anyway. Use a table literal {x = ..., y = ...} for dictionaries.",
        "alloc::mutable_upvalue_closure" => "When a closure captures a local that is later reassigned, Luau must use NEWCLOSURE (heap allocation) instead of DUPCLOSURE (free reuse of a prototype). Make captured locals immutable or restructure code.",
        "alloc::unpack_in_loop" => "table.unpack()/unpack() pushes all elements onto the stack each call. In a loop, this repeats the work N times. Cache unpacked values outside the loop.",
        "alloc::repeated_string_byte" => "Multiple string.byte(s, i) calls on the same string in a loop each do bounds checking and extraction. Use a single string.byte(s, 1, -1) call to get all bytes at once.",
        "alloc::string_interp_in_loop" => "String interpolation (`...{expr}...`) allocates a new string each iteration, just like concatenation (..). Use table.concat or buffer for loop string building.",
        "alloc::select_in_loop" => "select(i, ...) walks the vararg list from the start each call, making it O(n). In a loop over varargs, this is O(n^2) total. Cache results: local args = {...}; for i, v in ipairs(args).",
        "alloc::table_insert_known_size" => "table.insert() in a numeric for with known bounds causes incremental table resizing. Use table.create(n) to pre-allocate the array part, then assign by index: t[i] = value.",
        "alloc::buffer_over_string_pack" => "string.pack/unpack in a loop allocates a new string per call. The buffer library (buffer.writeu32/readu32) provides zero-allocation binary I/O using FASTCALL builtins.",
        "alloc::task_spawn_in_loop" => "task.spawn/defer in a loop creates a new coroutine per iteration (~247x overhead vs direct call). If the function doesn't need to yield, call it directly instead.",
        "alloc::gsub_function_in_loop" => "gsub with a function replacement in a loop invokes the function per match and allocates a closure. Cache the replacement function outside or use buffer-based string building.",

        // batch 6 additions
        "table::nil_field_in_constructor" => "Setting a field to nil in a table constructor defeats Luau's table template optimization. The compiler pre-allocates exact shapes, but nil fields waste hash slots. Omit them - nil is the default.",
        "table::rawset_in_loop" => "rawset() bypasses __newindex but is not a FASTCALL builtin. If no metatable is set, regular t[k] = v is faster because it uses SETTABLEKS/SETTABLE opcodes directly.",
        "table::next_t_nil_over_pairs" => "next(t, nil) is equivalent to next(t). The nil second argument is unnecessary and adds visual noise.",
        "complexity::filter_then_first" => "Iterating over GetDescendants/GetChildren just to find the first match is O(n). FindFirstChild or FindFirstChildOfClass is O(1) lookup with early return.",
        "complexity::nested_table_find" => "table.find() in a nested loop creates O(n*m*k) complexity. Convert the inner collection to a hashset: local set = {}; for _,v in t do set[v] = true end.",
        "memory::debris_negative_duration" => "Debris:AddItem with zero or negative duration destroys the instance on the same frame - likely a bug. Use a positive duration for timed cleanup.",
        "memory::collection_tag_no_cleanup" => "GetInstanceAddedSignal without GetInstanceRemovedSignal means tagged instances that are destroyed or reparented leave behind stale connections and data.",
        "memory::attribute_changed_in_loop" => "GetAttributeChangedSignal() inside a loop creates a new signal connection per iteration. Each connection lives until the instance is destroyed, leading to N duplicate handlers. Connect outside the loop or use a single Changed event.",
        "roblox::render_stepped_on_server" => "RenderStepped only fires on the client (it's tied to the rendering pipeline). On the server, use Heartbeat or Stepped instead.",
        "roblox::task_wait_no_arg" => "task.wait() with no argument waits exactly one frame (~16ms at 60fps). If you need a specific delay, pass a duration. If one frame is intentional, consider adding a comment.",
        "roblox::deprecated_delay" => "delay() is a legacy global with inconsistent timing behavior. task.delay() uses the modern task scheduler with better error handling and deterministic timing.",
        "roblox::clone_set_parent" => "Setting .Parent immediately after :Clone() before setting other properties triggers a replication packet per subsequent property change. Set all properties first, then .Parent last.",
        "native::pcall_in_native" => "pcall/xpcall in --!native scripts forces interpreter fallback for the protected call. The native compiler can't generate code across pcall boundaries. Restructure to minimize pcall usage in hot loops.",
        "native::dynamic_table_key_in_native" => "Dynamic table access t[variable] in --!native uses GETTABLE which can't be inline-cached. GETTABLEKS (constant string key, t.field) uses inline caching for fast property access.",
        "native::non_fastcall_in_hot_loop" => "These functions are NOT fastcall builtins in the Luau VM. In --!native code they fall back to the interpreter. Fastcall builtins: math.* (except noise/random), string.byte/char/len/sub, table.insert(2-arg)/unpack, buffer.*, bit32.*, vector.*, type/typeof, select.",
        "string::reverse_in_loop" => "string.reverse() allocates a new reversed string each call. In a loop, cache the result outside if the input string doesn't change between iterations.",
        "string::format_known_types" => "string.format(\"%s\", x) is just tostring(x) with extra format-string parsing overhead. Use tostring() directly for simple type conversion.",
        "physics::massless_not_set" => "The Massless property only has effect on parts that are welded to an assembly with a non-massless root part. On unanchored, unwelded parts, Massless does nothing.",
        "physics::assembly_velocity_in_loop" => "Setting AssemblyLinearVelocity/AssemblyAngularVelocity in a loop crosses the Lua-C++ bridge per call and fights the physics solver. Use constraint-based movers (LinearVelocity, AngularVelocity) instead.",
        "style::unused_variable_in_loop" => "Allocating an instance (Instance.new, :Clone) in a loop body but never using the variable wastes creation and GC cost per iteration.",
        "style::multiple_returns_hot_path" => "Returning many values from a hot-path function requires stack management overhead per frame. Consider returning a table or reducing return count.",
        "cache::brick_color_new_in_loop" => "BrickColor.new() in a loop allocates a BrickColor userdata each iteration. Cache outside if the color doesn't change between iterations.",
        "cache::region_new_in_loop" => "Region3.new() in a loop allocates a Region3 userdata each iteration. Cache outside if the bounds are loop-invariant.",
        "network::http_service_in_loop" => "HTTP requests (GetAsync/PostAsync/RequestAsync) in a loop send N network requests. Each one yields the thread. Batch requests or process asynchronously.",
        "network::marketplace_info_in_loop" => "GetProductInfo() in a loop makes an HTTP request per iteration. Cache results in a table keyed by product ID.",
        "render::image_label_in_loop" => "Creating ImageLabel/ImageButton in a loop loads an image asset per instance. Pre-create a template and use :Clone() for better performance.",
        "render::scrolling_frame_in_loop" => "ScrollingFrame creation in a loop triggers expensive layout computation per instance. Pre-create a template and :Clone().",
        "instance::destroy_in_loop" => ":Destroy() in a loop fires ancestry-changed events, Destroying events, and processes connections per call. For clearing children, use :ClearAllChildren() instead.",
        "instance::get_children_in_loop" => ":GetChildren/:GetDescendants allocates a new table of all children each call. In a loop, cache outside: local children = obj:GetChildren().",
        "math::huge_comparison" => "math.huge in a loop requires a GETIMPORT lookup each access. Cache in a local: local INF = math.huge before the loop.",
        "math::exp_over_pow" => "math.exp() in a loop with constant exponent recomputes the same value each iteration. Cache outside: local e = math.exp(k).",
        "alloc::typeof_in_loop" => "typeof() in a loop crosses the Lua-C++ bridge each call to determine the type. Cache outside if checking the same value repeatedly.",
        "alloc::setmetatable_in_loop" => "setmetatable() in a loop creates a new metatable-linked table per iteration. Consider object pooling or a constructor pattern to reuse metatables.",
        "roblox::yield_in_connect_callback" => "Yielding (task.wait, WaitForChild) inside :Connect callbacks blocks the signal handler. Use task.spawn to run async work from within a connection callback.",
        "roblox::deprecated_udim" => "UDim2.new(0, px, 0, py) can be UDim2.fromOffset(px, py). UDim2.new(sx, 0, sy, 0) can be UDim2.fromScale(sx, sy). Cleaner and more readable.",
        "roblox::teleport_service_race" => "TeleportAsync can fail from rate limits, network errors, or invalid place IDs. Without pcall, the error kills the script. Always wrap in pcall with retry logic.",
        "roblox::color3_new_misuse" => "Color3.new() takes values in the 0-1 range. Passing values like 255 means you probably intended Color3.fromRGB() which takes 0-255. Color3.new(255, 0, 0) produces white, not red.",
        "roblox::raycast_filter_deprecated" => "Enum.RaycastFilterType.Blacklist and Whitelist are deprecated. Use Exclude and Include respectively. The old names will be removed in a future engine update.",
        "roblox::player_added_race" => "Players.PlayerAdded only fires for players who join AFTER the event is connected. If a player joins before the script runs (common with deferred loading), they are silently missed. Always loop through Players:GetPlayers() after connecting PlayerAdded.",
        "roblox::game_workspace" => "game.Workspace crosses the Lua-C++ bridge to look up the Workspace service. The global `workspace` is a direct reference that avoids this overhead.",
        "roblox::coroutine_resume_create" => "coroutine.resume(coroutine.create(f)) is the Lua 5.1 pattern for spawning threads. In Luau, task.spawn(f) is simpler, handles errors properly (prints traceback instead of silently failing), and integrates with the task scheduler.",
        "roblox::character_added_no_wait" => "CharacterAdded only fires when a NEW character spawns. If the character already exists when you connect (e.g., late-loading scripts), the handler won't fire for it. Check player.Character first and handle the existing character.",
        "roblox::getservice_workspace" => "game:GetService(\"Workspace\") returns the same thing as the global `workspace`. The global is a direct reference that doesn't cross the Lua-C++ bridge, making it simpler and marginally faster.",
        "math::floor_round_manual" => "math.floor(x + 0.5) is a manual rounding idiom from Lua 5.1. Luau provides math.round(x) which is clearer and handles edge cases (negative numbers, .5 rounding) correctly.",
        "math::max_min_single_arg" => "math.max(x) and math.min(x) with a single argument just return that argument unchanged. This is likely a bug - you probably meant to compare against another value like math.max(x, 0) or math.min(x, limit).",
        "string::format_no_args" => "string.format(\"literal\") with no format arguments returns the string unchanged. Just use the string directly instead of wrapping it in string.format().",
        "string::format_redundant_tostring" => "string.format's %s specifier already calls tostring() internally. Wrapping the argument in tostring() is redundant and adds unnecessary overhead.",
        "string::format_simple_concat" => "string.format() with only %s specifiers is doing simple concatenation with function call overhead. string.format is NOT a VM fastcall builtin. Use the .. operator instead — it compiles to a single CONCAT opcode that batches all operands efficiently.",
        "roblox::find_first_child_no_check" => "FindFirstChild returns nil if the child doesn't exist. Accessing a property on the result without checking for nil will throw 'attempt to index nil' at runtime. Store in a local and check before accessing.",
        "roblox::get_full_name_in_loop" => "GetFullName() builds the full ancestry path string each call. In a loop, this allocates N strings. Cache the result outside the loop if the instance doesn't change.",
        "roblox::bind_to_render_step_no_cleanup" => "BindToRenderStep registers a named callback to run every frame. Without a matching UnbindFromRenderStep, the binding persists indefinitely, leaking if the script is reused or the feature is toggled off.",
        "roblox::cframe_old_constructor" => "CFrame.new() with 12 positional number args (position + rotation matrix) is deprecated and harder to read. Use CFrame.fromMatrix(pos, rightVector, upVector, lookVector) or CFrame.new(pos) * CFrame.fromEulerAngles() instead.",
        "complexity::string_match_in_loop" => "string.match() compiles the pattern each call. In a loop, the same pattern is compiled N times. Use gmatch for iteration or cache results outside the loop.",
        "complexity::promise_chain_in_loop" => "Promise chaining (:andThen, :catch) in a loop creates N promise objects per iteration. Collect items and use Promise.all() for batch processing.",
        "complexity::repeated_typeof" => "typeof() or type() called 3+ times on the same expression. Cache the result in a local: `local xType = typeof(x)` then compare against the local.",

        // cache
        "cache::magnitude_over_squared" => ".Magnitude computes sqrt internally. When comparing distances (if a.Magnitude < b), compare squared values instead: a.Magnitude * a.Magnitude < b * b, avoiding the sqrt cost.",
        "cache::uncached_get_service" => ":GetService() does a lookup each call. Cache the result at module level: local Players = game:GetService('Players'). This also enables GETIMPORT optimization.",
        "cache::tween_info_in_function" => "TweenInfo.new() allocates a new userdata each call. If the parameters are constant, cache it as a module-level local to avoid repeated allocation.",
        "cache::raycast_params_in_function" => "RaycastParams.new() allocates a new userdata each call. Create once at module level and reuse by updating FilterDescendantsInstances as needed.",
        "cache::instance_new_in_loop" => "Instance.new() in a loop creates N instances sequentially. Consider :Clone() from a template (faster for complex instances) or pre-allocating outside the loop.",
        "cache::cframe_new_in_loop" => "CFrame constructors in a loop allocate a new CFrame each iteration. If the arguments are loop-invariant, cache the CFrame outside the loop.",
        "cache::vector3_new_in_loop" => "Vector3.new() in a loop allocates a new Vector3 each iteration. If arguments are loop-invariant, cache outside the loop. In --!native, Vector3 uses SIMD when typed.",
        "cache::vector2_new_in_loop" => "Vector2.new() in a loop allocates a new Vector2 each iteration. If arguments are loop-invariant, cache outside the loop.",
        "cache::overlap_params_in_function" => "OverlapParams.new() allocates a new userdata each call. Create once at module level and reuse by updating properties as needed.",
        "cache::number_range_in_function" => "NumberRange.new() allocates a new userdata. If the range is constant, cache as a module-level local.",
        "cache::number_sequence_in_function" => "NumberSequence.new() allocates a new userdata. If the sequence is constant, cache as a module-level local.",
        "cache::color_sequence_in_function" => "ColorSequence.new() allocates a new userdata. If the sequence is constant, cache as a module-level local.",
        "cache::tween_create_in_loop" => "TweenService:Create() allocates a new Tween object each call. In a loop, this creates N tweens. Pre-create tweens or use a tween pool.",
        "cache::get_attribute_in_loop" => ":GetAttribute() crosses the Lua-C++ bridge (~247ns per call). In a loop, cache the value outside: local val = obj:GetAttribute('key').",
        "cache::color3_new_in_loop" => "Color3 constructors in a loop allocate a new Color3 each iteration. If arguments are loop-invariant, cache outside the loop.",
        "cache::udim2_new_in_loop" => "UDim2 constructors in a loop allocate a new UDim2 each iteration. If arguments are loop-invariant, cache outside the loop.",
        "cache::repeated_method_call" => "Methods like :GetChildren(), :GetDescendants() allocate a new table each call. Calling the same method 2+ times wastes allocations. Cache: local children = obj:GetChildren().",

        // complexity
        "complexity::table_find_in_loop" => "table.find() is O(n) linear search. In a loop, this becomes O(n*m). Convert the lookup table to a hashmap: local set = {}; for _,v in t do set[v] = true end.",
        "complexity::get_descendants_in_loop" => "GetDescendants()/GetChildren() allocates a new table of all descendants each call. In a loop, this creates N tables. Cache outside: local desc = obj:GetDescendants().",
        "complexity::table_remove_shift" => "table.remove(t, 1) shifts all remaining elements left - O(n) per call. For queue patterns, use a read index or swap the element with the last and remove from end.",
        "complexity::table_sort_in_loop" => "table.sort() is O(n log n). Sorting inside a loop multiplies this cost. Sort once outside the loop, or maintain a sorted data structure.",
        "complexity::get_tagged_in_loop" => "CollectionService:GetTagged() allocates a new table each call. In a loop, cache outside: local tagged = CollectionService:GetTagged('Tag').",
        "complexity::get_players_in_loop" => ":GetPlayers() allocates a new table each call. Cache outside the loop: local players = Players:GetPlayers().",
        "complexity::clone_in_loop" => ":Clone() deep-copies the entire instance tree. In a loop, this is expensive. Consider object pooling or pre-creating clones.",
        "complexity::wait_for_child_in_loop" => ":WaitForChild() yields the thread until the child exists. In a loop, each iteration may yield. Cache results: local child = parent:WaitForChild('Name').",
        "complexity::find_first_child_recursive" => "FindFirstChild(name, true) does a recursive O(n) search through all descendants. Cache the result or use CollectionService tags for indexed lookup.",
        "complexity::require_in_function" => "require() inside a function body runs on every call. While Luau caches module results, the lookup still has overhead. Move require to module level for clarity and GETIMPORT.",
        "complexity::deep_metatable_chain" => "Each __index lookup walks the metatable chain linearly. With >3 levels, this defeats Luau's inline caching. Flatten the hierarchy or use explicit method tables.",
        "complexity::pairs_in_pairs" => "Nested pairs/ipairs loops create O(n*m) iteration. Consider using a lookup table for the inner loop to reduce to O(n+m).",
        "complexity::gmatch_in_loop" => "string.gmatch() creates a new iterator and compiles the pattern each call. In a loop, this repeats per iteration. Move outside if the pattern is constant.",
        "complexity::datastore_no_pcall" => "DataStore operations can fail from throttling, network issues, or Roblox outages. Without pcall, the error propagates and kills the script. Always wrap DataStore calls in pcall for resilience.",

        // instance
        "instance::two_arg_instance_new" => "Instance.new(class, parent) sets Parent immediately, triggering replication before properties are set. Each subsequent property change sends another packet. Set .Parent last: local p = Instance.new('Part'); p.Size = ...; p.Parent = workspace.",
        "instance::property_change_signal_wrong" => ".Changed fires for ANY property change on the instance. Use GetPropertyChangedSignal('PropertyName') to only fire for the specific property you care about.",
        "instance::clear_all_children_loop" => "Calling :Destroy() in a loop over :GetChildren() iterates and destroys one at a time. :ClearAllChildren() is a single C++ call that handles the batch internally.",
        "instance::set_parent_in_loop" => "Setting .Parent in a loop triggers ancestry-changed events and replication for each iteration. Batch: create instances unparented, set all properties, then parent them.",
        "instance::property_before_parent" => "Setting .Parent before other properties triggers a replication packet per subsequent property change. Set properties first, parent last to batch into a single replication packet.",
        "instance::repeated_find_first_child" => "Calling FindFirstChild() with the same argument multiple times wastes CPU on repeated tree searches. Cache the result: local child = parent:FindFirstChild('Name').",
        "instance::changed_on_moving_part" => ".Changed on Parts/Models fires for EVERY property change, including Position/CFrame updates from physics simulation (~240Hz). Use GetPropertyChangedSignal for specific properties.",
        "instance::bulk_property_set" => "Setting 5+ properties on a parented instance triggers replication for each one. For CFrame/Position changes, use workspace:BulkMoveTo(). For other properties, set Parent last.",

        // math
        "math::random_deprecated" => "math.random() uses a global RNG state shared across all scripts. Random.new() creates an independent RNG with better distribution and thread safety.",
        "math::random_new_in_loop" => "Random.new() allocates a new RNG each call. Create once outside the loop: local rng = Random.new(); for ... do rng:NextNumber() end.",
        "math::clamp_manual" => "math.min(math.max(x, min), max) is two function calls. math.clamp(x, min, max) is a single FASTCALL builtin - fewer instructions, clearer intent.",
        "math::sqrt_over_squared" => "math.sqrt() computes a square root. When comparing distances, compare squared values instead: (a-b).Magnitude^2 < threshold^2. Avoids the sqrt entirely.",
        "math::floor_division" => "math.floor(a/b) requires a function call. The // operator compiles to a single IDIV opcode, avoiding function overhead.",
        "math::fmod_over_modulo" => "math.fmod(a, b) is a function call. The % operator compiles to MOD/MODK bytecode (single opcode).",

        // memory
        "memory::untracked_connection" => ":Connect() returns an RBXScriptConnection. Not storing it means you can never :Disconnect(), causing the callback and everything it captures to stay in memory forever.",
        "memory::untracked_task_spawn" => "task.spawn/task.delay create threads that can't be cancelled if you don't store the return value. Track threads for cleanup in module destroy/PlayerRemoving handlers.",
        "memory::connect_in_loop" => ":Connect() in a loop creates N separate connections. Each one fires independently and can never be disconnected. This is almost always a bug.",
        "memory::missing_player_removing" => "PlayerAdded without a corresponding PlayerRemoving handler means per-player data (tables, connections) is never cleaned up when players leave, causing memory growth over time.",
        "memory::while_true_no_yield" => "while true do without any yielding call (wait, task.wait, coroutine.yield) runs forever without giving other threads time. Luau will kill the script after a timeout.",
        "memory::connect_in_connect" => ":Connect() inside another :Connect() callback creates a new inner connection every time the outer event fires. The inner connections are never disconnected, leaking memory.",
        "memory::character_added_no_cleanup" => "CharacterAdded fires on each respawn. Connections made to character descendants leak if not disconnected when the character is destroyed. Use CharacterRemoving or Destroying for cleanup.",
        "memory::heartbeat_allocation" => "Table/object allocation inside Heartbeat/RenderStepped callbacks runs at 60Hz, creating ~60 garbage tables per second. Pre-allocate outside the callback and reuse.",
        "memory::circular_connection_ref" => "When a :Connect() callback captures a reference to the object whose event it listens to, it creates a cycle: Instance → Connection → Closure → Instance. Luau's GC can't collect cycles through C++ connections.",
        "memory::weak_table_no_shrink" => "Weak tables (__mode = 'v' or 'k') don't shrink their internal array when entries are collected. Add 's' to the mode string (__mode = 'vs') to enable shrinking.",
        "memory::runservice_no_disconnect" => "RunService.Heartbeat/RenderStepped/Stepped:Connect() fires every frame. Without storing the connection, you can never :Disconnect() it, causing the callback to run forever.",

        // native
        "native::getfenv_setfenv" => "getfenv/setfenv disables ALL Luau optimizations for the entire script: GETIMPORT (cached globals), FASTCALL (builtin fast-paths), DUPCLOSURE (free closure reuse), and native codegen.",
        "native::dynamic_require" => "require(table[key]) prevents Luau from statically resolving the module path. This disables GETIMPORT optimization for the required module's exports.",
        "native::coroutine_in_native" => "Coroutines in --!native scripts force interpreter fallback for coroutine-related functions. The native compiler can't generate code for yield points.",
        "native::math_huge_comparison" => "Comparing to math.huge requires a global lookup. Use x ~= x to check for NaN (only NaN is not equal to itself), or x == 1/0 for positive infinity.",
        "native::vararg_in_native" => "Vararg (...) access in --!native hot loops prevents some native code optimizations. Consider passing explicit parameters instead of varargs in performance-critical functions.",
        "native::string_pattern_in_native" => "String pattern functions (match, find, gmatch, gsub) run in the interpreter even in --!native scripts. They can't be compiled to native code. Move pattern matching out of hot loops.",
        "native::loadstring_deopt" => "loadstring() disables ALL Luau optimizations for the entire script, same as getfenv/setfenv. The compiler can't reason about dynamically compiled code.",
        "native::untyped_params" => "Functions in --!native without type annotations on parameters miss specialization opportunities. Typed Vector3 params enable SIMD, typed numbers enable unboxing.",
        "native::heavy_api_script" => "--!native benefits computation (math, loops, table ops), not Roblox API bridge calls. Scripts that mostly call APIs see no native codegen benefit - the time is spent in C++, not Lua.",
        "native::large_table_literal" => "Large table literals in --!native scripts waste native compilation memory on table-creation code. The native compiler generates code for each entry. Move large data tables to non-native modules.",
        "native::mixed_computation_api" => "Functions mixing computation and API calls in --!native compile everything to native, but only the computation benefits. Split into a native computation function and a non-native API function.",
        "native::global_write" => "Writing to _G (e.g. _G.foo = bar) disables the safeenv flag for the entire script. This turns off GETIMPORT (cached globals), FASTCALL (builtin fast-paths), and native codegen.",
        "native::shadowed_builtin" => "Shadowing a builtin like 'local math = require(...)' prevents FASTCALL and GETIMPORT optimizations for that builtin in the current scope. The VM can't prove the local is the real builtin.",
        "native::table_zero_index" => "Luau arrays are 1-based. Index 0 goes into the hash part of the table (slower than array part) and is skipped by ipairs() and the # operator.",

        // network
        "network::fire_in_loop" => "Firing a RemoteEvent in a loop sends N network packets. Each one has header overhead and may be throttled. Batch data into a single table and fire once.",
        "network::invoke_server_in_loop" => "InvokeServer() yields until the server responds. In a loop, this serializes N round-trips. Batch into a single invoke with all data.",
        "network::large_remote_data" => "Large/deeply nested tables in Remote calls are serialized and sent over the network. Flatten nested structures and remove redundant data to reduce payload size.",
        "network::fire_client_per_player" => ":FireClient() in a loop over Players:GetPlayers() sends N individual network packets. Use :FireAllClients() to send a single message to all players.",
        "network::remote_event_string_data" => "tostring()/string.format() in Remote fire arguments converts data to strings before sending. Send raw values and format on the receiving end to reduce serialization overhead.",

        // physics
        "physics::spatial_query_in_loop" => "Physics queries (Raycast, GetPartBoundsInBox, GetPartsInPart, etc.) are expensive C++ operations. In a loop, consider spatial indexing or batching queries.",
        "physics::move_to_in_loop" => ":MoveTo() sets CFrame and fires events for each call. workspace:BulkMoveTo() batches multiple moves into a single operation with less overhead.",
        "physics::spatial_query_per_frame" => "Spatial queries (Raycast, GetPartBoundsInBox, etc.) inside RunService callbacks run every frame at 60Hz. Each call traverses the physics spatial hash. Throttle with a counter, cache results across frames, or use CollectionService tags for entity tracking.",

        // render
        "render::gui_creation_in_loop" => "Creating GUI instances (Frame, TextLabel, etc.) in a loop is expensive. Pre-create templates and use :Clone(), or pool GUI elements for reuse.",
        "render::beam_trail_in_loop" => "Beam/Trail creation in a loop allocates rendering resources per iteration. Pre-create and reuse by toggling Enabled or re-attaching Attachments.",
        "render::particle_emitter_in_loop" => "ParticleEmitter creation in a loop is expensive. Create once, reuse via :Emit(count) to trigger particles without re-creating the emitter.",
        "render::billboard_gui_in_loop" => "BillboardGui creation in a loop allocates a 3D-to-2D rendering context per iteration. Pre-create a template and use :Clone().",
        "render::transparency_change_in_loop" => "Setting Transparency in a loop causes per-frame rendering updates. Use TweenService or NumberSequence for smooth transitions handled by the engine.",

        // roblox
        "roblox::deprecated_wait" => "wait() is a legacy global that throttles to 1/30s resolution. task.wait() uses the modern task scheduler with frame-accurate timing.",
        "roblox::deprecated_spawn" => "spawn() and delay() are legacy globals with inconsistent timing. task.spawn() and task.delay() use the modern task scheduler with better error handling and deterministic behavior.",
        "roblox::debris_add_item" => "Debris:AddItem() uses a legacy internal timer. task.delay(time, function() obj:Destroy() end) is more precise and follows modern task scheduler semantics.",
        "roblox::missing_native" => "The --!native directive enables native code generation (JIT-like compilation). Scripts with computation-heavy code (math, loops) see significant speedups.",
        "roblox::deprecated_body_movers" => "BodyVelocity, BodyForce, BodyGyro etc. are deprecated. Use modern constraint-based movers: LinearVelocity, VectorForce, AlignOrientation for better physics simulation.",
        "roblox::pcall_in_loop" => "pcall/xpcall are not FASTCALL builtins - each call has significant overhead compared to builtins. In tight loops, this overhead accumulates. Guard with a flag or restructure.",
        "roblox::missing_strict" => "The --!strict directive enables strict type checking, catching errors at analysis time and enabling the compiler to generate better bytecode and native code.",
        "roblox::wait_for_child_no_timeout" => "WaitForChild() without a timeout yields the thread forever if the child never appears. Always provide a timeout: WaitForChild('Name', 5).",
        "roblox::model_set_primary_part_cframe" => "SetPrimaryPartCFrame() is deprecated and slower than PivotTo(). Model:PivotTo(cframe) uses the model's pivot for positioning.",
        "roblox::get_rank_in_group_uncached" => "GetRankInGroup() makes an HTTP request each call. Cache the result per player at join time: local rank = player:GetRankInGroup(groupId).",
        "roblox::insert_service_load_asset" => "InsertService:LoadAsset() makes an HTTP request and deserializes the asset. Cache the result to avoid repeated network calls.",
        "roblox::deprecated_physics_service" => "PhysicsService collision group methods are deprecated. Use BasePart.CollisionGroup string property instead, which is simpler and more performant.",
        "roblox::set_attribute_in_loop" => "Each SetAttribute() call triggers attribute replication. In a loop, this sends N packets. Batch attribute changes or consider alternative data storage.",
        "roblox::string_value_over_attribute" => "Instance.new('StringValue') etc. creates a full Instance for storing a single value. Attributes (:SetAttribute/:GetAttribute) are lighter - no instance overhead.",
        "roblox::touched_event_unfiltered" => ".Touched fires at physics rate (~240Hz per contact pair). Without debounce/filtering in the handler, your callback runs hundreds of times per second.",
        "roblox::destroy_children_manual" => "Calling :Destroy() in a loop over :GetChildren() iterates one at a time. :ClearAllChildren() is a single C++ call that handles the batch.",
        "roblox::missing_optimize" => "The --!optimize 2 directive enables function inlining and loop unrolling at the bytecode level. Should be paired with --!native for maximum performance.",
        "roblox::deprecated_region3" => "FindPartsInRegion3 and variants are deprecated. Use workspace:GetPartBoundsInBox() with OverlapParams for better control and performance.",
        "roblox::bindable_same_script" => "BindableEvent:Fire() and .Event:Connect() in the same script adds unnecessary serialization overhead. Call the handler function directly.",
        "roblox::server_property_in_heartbeat" => "Property assignments (.Position, .CFrame) inside Heartbeat/Stepped trigger replication every frame. Use UnreliableRemoteEvent for client-driven updates or batch changes.",

        // string
        "string::len_over_hash" => "string.len(s) or s:len() is a function call. #s compiles to the LEN opcode directly - no function call overhead.",
        "string::rep_in_loop" => "string.rep() allocates a new string each call. In a loop, this creates N strings. Cache the result if the input doesn't change.",
        "string::gsub_for_find" => ":gsub(pattern, '') is used to remove characters, but if you only need to check existence, string.find() is cheaper - no allocation.",
        "string::lower_upper_in_loop" => "string.lower/upper allocates a new string each call. If the input is constant across iterations, cache the result outside the loop.",
        "string::byte_comparison" => "string.sub(s, i, i) allocates a 1-char string for comparison. string.byte(s, i) returns a number - no allocation, faster comparison.",
        "string::sub_for_single_char" => "string.sub for single character extraction allocates a new string. string.byte returns a number directly - use it when comparing characters.",

        // style
        "style::duplicate_get_service" => "Multiple GetService() calls for the same service repeat the lookup. Cache in a module-level local: local Players = game:GetService('Players').",
        "style::empty_function_body" => "Empty function bodies (function() end) still allocate a closure. If used as a no-op callback, consider a shared constant: local NOOP = function() end.",
        "style::deprecated_global" => "rawget/rawset/rawequal bypass metatables. Verify this is intentional - it may indicate a workaround for incorrect metatable usage.",
        "style::type_check_in_loop" => "typeof() in a loop rechecks the type each iteration. If checking the same value, cache outside: local t = typeof(obj).",
        "style::deep_nesting" => "Deeply nested code (>8 levels) is hard to read and may indicate complex control flow. Extract helper functions to flatten the structure.",
        "style::dot_method_call" => "obj.Method(obj, ...) bypasses NAMECALL optimization. obj:Method(...) compiles to NAMECALL - a single opcode that combines table lookup and method call.",
        "style::print_in_hot_path" => "print/warn involve I/O and string formatting. In loops or RunService callbacks (60Hz), this creates significant overhead. Remove or guard with a debug flag for production.",
        "style::debug_in_hot_path" => "debug.traceback/info perform expensive stack introspection, walking the call stack each time. In loops, this overhead accumulates. Move outside or guard with a condition.",
        "style::index_function_metatable" => "__index = function(self, key) prevents Luau's inline caching. Using __index = methodTable allows the VM to cache lookups after the first access.",
        "style::conditional_field_in_constructor" => "Conditionally setting different fields creates objects with different 'shapes'. Luau's inline cache works best when objects have consistent key sets. Initialize all fields, even if nil.",
        "style::global_function_not_local" => "'function foo()' creates a global function, polluting the environment and preventing GETIMPORT optimization. 'local function foo()' enables inlining at --!optimize 2.",

        // table
        "table::foreach_deprecated" => "table.foreach/foreachi are deprecated Lua 5.0 functions. for k,v in pairs/ipairs loops are faster - they use FORGPREP_NEXT/FORGPREP_INEXT optimizations.",
        "table::getn_deprecated" => "table.getn(t) is deprecated. #t compiles to the LEN opcode directly and is the idiomatic way to get array length.",
        "table::maxn_deprecated" => "table.maxn(t) is deprecated. It scans the entire table for the largest numeric key. Use #t for contiguous arrays or track the max index manually.",
        "table::freeze_in_loop" => "table.freeze() makes a table read-only. Freezing inside a loop is wasteful - freeze once after the table is fully constructed.",
        "table::insert_with_position" => "table.insert(t, pos, v) with a position shifts all elements after pos - O(n). It also can't use FASTCALL. Use 2-arg table.insert(t, v) for appending.",
        "table::remove_in_ipairs" => "table.remove() during ipairs/pairs iteration shifts elements and corrupts the iteration order, causing skipped elements. Iterate backwards or collect indices to remove after.",
        "table::pack_over_literal" => "table.pack(...) is a function call that creates a table. {...} is a table constructor - directly compiled to NEWTABLE + SETLIST, significantly faster.",
        "table::manual_copy_loop" => "Manually copying a table with for k,v in pairs(src) do dst[k] = v end is slow. table.clone() is a single C call that copies the entire table at once.",
        "table::deferred_field_assignment" => "local t = {} followed by t.x = ... misses Luau's table template optimization. Using {x = ..., y = ...} lets the compiler pre-allocate the exact shape.",
        "table::ipairs_over_numeric_for" => "for i = 1, #t do ... t[i] ... uses index-based access. for i, v in ipairs(t) uses FORGPREP_INEXT - a specialized fast-path that's ~2x faster for packed arrays.",
        "table::polymorphic_constructor" => "Table constructors with different key sets in the same scope create differently-shaped objects. Luau's inline cache (IC) can only cache one shape per access site - misses cause ~27% overhead.",

        // batch 1 additions
        "math::pow_two" => "math.pow(x, 2) is a function call. x * x is a single MUL instruction - faster and avoids call overhead. The VM has special-cased x^2 in POWK, but x * x is still clearer.",
        "math::pow_slow_exponent" => "The Luau VM only fast-paths ^2 (x*x), ^0.5 (sqrt), and ^3 (x*x*x) in the POWK opcode. All other constant exponents call libc pow() which is ~50-100x slower. Common fixes: ^4 → local x2=x*x; x2*x2, ^(-1) → 1/x, ^0.25 → math.sqrt(math.sqrt(x)).",
        "math::vector_normalize_manual" => "v / v.Magnitude manually normalizes a vector. v.Unit is a built-in property that computes the unit vector natively - no Lua-side division needed.",
        "math::unnecessary_tonumber" => "tonumber() on a numeric literal is a no-op. The value is already a number - remove the unnecessary function call.",
        "math::lerp_manual" => "a + (b - a) * t is a manual linear interpolation. Use Vector3:Lerp(target, alpha), CFrame:Lerp(target, alpha), or a dedicated lerp utility for clarity and potential optimization.",
        "math::abs_for_sign_check" => "math.abs(x) > 0 is equivalent to x ~= 0. Comparing directly avoids the function call. math.abs(x) == 0 is equivalent to x == 0.",
        "physics::touched_without_debounce" => ".Touched fires at ~240Hz per contact pair. Without a debounce/cooldown check at the top of the handler, the callback runs hundreds of times per second.",
        "physics::set_network_owner_in_loop" => "SetNetworkOwner() changes physics ownership, which involves network negotiation. In a loop, this triggers N ownership changes. Set once outside the loop.",
        "physics::precise_collision_fidelity" => "PreciseConvexDecomposition is the most expensive collision fidelity mode. It decomposes meshes into many convex hulls. Use Box, Hull, or Default for most parts.",
        "physics::collision_group_string_in_loop" => "Setting .CollisionGroup to a string in a loop does a string comparison for the collision group name each time. Cache the assignment outside the loop.",
        "physics::anchored_with_velocity" => "Anchored parts ignore all physics forces and velocities. Setting Velocity/Force properties on an Anchored part is wasted work.",
        "cache::current_camera_uncached" => "workspace.CurrentCamera crosses the Lua-C++ bridge each access. Cache in a local: local camera = workspace.CurrentCamera.",
        "cache::local_player_uncached" => "Players.LocalPlayer crosses the Lua-C++ bridge each access. Cache in a module-level local: local localPlayer = Players.LocalPlayer.",
        "cache::workspace_lookup_in_loop" => "workspace:FindFirstChild/WaitForChild in a loop searches the workspace tree each iteration. Cache the result outside: local obj = workspace:FindFirstChild('Name').",
        "memory::task_delay_long_duration" => "task.delay() with very long durations (>5 minutes) keeps the callback and its captures alive in memory for the duration. Consider alternative approaches for long-lived timers.",
        "memory::tween_completed_connect" => ".Completed:Connect() creates a permanent connection. Use .Completed:Once() instead - it automatically disconnects after the first fire, preventing memory leaks.",
        "memory::set_attribute_in_heartbeat" => "SetAttribute() in a RunService callback triggers attribute replication at 60Hz. That's 60 replication packets per second per attribute per instance. Use plain Lua tables for per-frame mutable data instead.",
        "style::assert_in_hot_path" => "assert() has overhead even when the condition is true - it evaluates all arguments and checks the result. In hot loops, this adds up. Remove assertions or guard with a debug flag.",
        "style::redundant_condition" => "if true then / if false then are unconditional branches. Remove the condition (if true) or the dead code (if false).",
        "style::long_function_body" => "Functions with many statements are hard to maintain and optimize. The native code compiler has per-function limits. Split large functions into smaller, focused helpers.",
        "style::duplicate_string_literal" => "The same string literal appearing many times wastes memory and makes refactoring harder. Extract to a module-level constant.",
        "string::tostring_on_string" => "tostring() on a value that is already a string is a no-op function call. Remove it.",
        "string::find_missing_plain_flag" => "string.find(s, literal) without the plain flag compiles the pattern even for literal strings. Add nil, true as 3rd/4th args to skip pattern compilation.",
        "string::lower_for_comparison" => "Calling string.lower() twice for case-insensitive comparison allocates two new strings. Consider a helper function or use string.byte() for single-character checks.",
        "table::sort_comparison_allocation" => "table.sort(t, function(a, b) ... end) with an inline comparator in a loop allocates a new closure per iteration. Extract the comparison function outside.",
        "table::clear_vs_new" => "Reassigning a variable to {} in a loop allocates a new table each iteration. table.clear(t) reuses the existing table's memory, avoiding allocation and GC pressure.",
        "table::move_over_loop" => "Copying array elements one at a time in a loop is O(n) Lua operations. table.move(src, 1, #src, 1, dst) is a single C call that does the same copy faster.",
        "table::concat_with_separator_loop" => "result = result .. sep .. item in a loop creates O(n^2) intermediate strings. Use table.insert into an array, then table.concat(t, sep) for O(n) string building.",
        "roblox::game_loaded_race" => "game:IsLoaded() without game.Loaded:Wait() has a race condition: if the game hasn't loaded yet when this code runs, the check returns false and you miss the load event entirely.",
        "roblox::humanoid_state_polling" => "Humanoid:GetState() in a loop polls the state every iteration. Use Humanoid.StateChanged:Connect() instead - it fires only when the state actually changes.",
        "roblox::server_side_tween" => "TweenService:Create() on the server creates tweens that replicate every property change to all clients. Run visual tweens on the client instead.",
        "roblox::require_in_connect" => "require() inside a :Connect() callback runs on every event fire. Module require has lookup overhead even with caching. Hoist to module level.",
        "roblox::find_first_child_chain" => "Chaining :FindFirstChild() calls (a:FindFirstChild('B'):FindFirstChild('C'):FindFirstChild('D')) does a tree search at each step. Cache intermediate results in locals.",
        "roblox::once_over_connect" => ":Connect() followed by :Disconnect() in the handler is the manual version of :Once(). Use :Once() instead - it auto-disconnects after the first fire, cleaner and no leaked connection reference.",
        "render::rich_text_in_loop" => "Rich text tags (<font>, <b>, etc.) inside string building in a loop create complex formatted strings per iteration. Pre-build if content is static.",
        "string::match_for_boolean" => "string.match() in a boolean context (if/while) allocates capture tables even when you only care about truthiness. string.find() returns indices without allocation - use it when you don't need captures.",
        "string::concat_chain" => "Long concatenation chains (a .. b .. c .. d .. e .. f) create N-1 intermediate strings. Use string.format(), string interpolation, or table.concat() for cleaner code and fewer allocations.",
        "instance::collection_service_in_loop" => "AddTag/RemoveTag in a loop triggers CollectionService events per call, causing listeners to fire N times. HasTag in a loop crosses the Lua-C++ bridge each iteration. Batch tag operations or cache tag state.",
        "instance::name_indexing_in_loop" => "workspace.Name in a loop does a name-based instance lookup each iteration, crossing the Lua-C++ bridge. Cache the reference outside: local obj = workspace.Name.",
        "roblox::health_polling" => "Humanoid.Health in a loop polls the property each iteration. Use Humanoid.HealthChanged event or GetPropertyChangedSignal('Health') instead - fires only when health actually changes.",
        "roblox::changed_event_unfiltered" => ".Changed fires for ANY property change on the instance, including internal engine updates. Use GetPropertyChangedSignal('PropertyName') to listen for specific properties only.",
        "physics::raycast_params_in_loop" => "RaycastParams.new() allocates a new userdata each call. In a loop, this creates N params objects. Create once outside the loop and reuse by updating FilterDescendantsInstances as needed.",
        "physics::cframe_assign_in_loop" => ".CFrame assignment in a loop crosses the Lua-C++ bridge, triggers physics recalculation, and sends a replication packet per iteration. Use workspace:BulkMoveTo() to batch all moves into one engine call.",
        "math::vector3_zero_constant" => "Vector3.new(0,0,0) allocates a new Vector3. Vector3.zero is a pre-allocated constant - no allocation, no constructor call. Same for Vector3.one.",
        "math::vector2_zero_constant" => "Vector2.new(0,0) allocates a new Vector2. Vector2.zero is a pre-allocated constant - no allocation, no constructor call. Same for Vector2.one.",
        "math::cframe_identity_constant" => "CFrame.new() with no arguments allocates a new identity CFrame. CFrame.identity is a pre-allocated constant - no allocation.",
        "network::datastore_in_loop" => "DataStore operations yield and are rate-limited (60 + numPlayers*10/min). In a loop, you risk hitting throttle limits and each iteration yields the thread. Batch operations or use a queue.",
        "roblox::descendant_event_workspace" => "DescendantAdded/Removing on workspace fires for EVERY instance added or removed anywhere in the entire game. Use CollectionService tags for indexed lookup or scope the listener to a smaller subtree.",
        "roblox::get_attribute_in_heartbeat" => ":GetAttribute() in a RunService callback crosses the Lua-C++ bridge at 60Hz. Cache the value in a Lua variable and update via AttributeChanged events.",
        "roblox::pivot_to_in_loop" => ":PivotTo() in a loop crosses the Lua-C++ bridge per call and triggers replication. workspace:BulkMoveTo() batches all moves into a single engine call.",
        "table::pairs_over_generalized" => "pairs()/ipairs() are function calls that return an iterator. Luau's generalized iteration (for k, v in t do) emits the same FORGPREP bytecode without the function call overhead.",
        "style::type_over_typeof" => "type() returns Lua types only ('string', 'number', 'table', etc.). typeof() also handles Roblox types ('Vector3', 'CFrame', 'Instance', etc.). Use typeof() for correct Roblox type checking.",
        "style::nested_ternary" => "Deeply nested if/then/else expressions are hard to read and maintain. Extract to a helper function or use a lookup table.",

        // batch 5 additions
        "roblox::deprecated_tick" => "tick() is deprecated and returns Unix timestamp with limited precision. Use os.clock() for elapsed time measurement or workspace:GetServerTimeNow() for synchronized wall-clock time.",
        "roblox::deprecated_find_part_on_ray" => "FindPartOnRay/FindPartOnRayWithWhitelist/FindPartOnRayWithIgnoreList are deprecated. workspace:Raycast() with RaycastParams provides better control and performance.",
        "roblox::while_wait_do" => "while wait() do combines yielding and loop condition in a way that obscures control flow. Use while true do ... task.wait() end for explicit timing control with the modern task scheduler.",
        "roblox::get_property_changed_in_loop" => ":GetPropertyChangedSignal() creates a new signal object each call. In a loop, this creates N signal objects that are never garbage collected. Cache the signal outside the loop or use a single .Changed handler.",
        "complexity::accumulating_rebuild" => "{unpack(result), item} in a loop copies the entire growing table each iteration, creating O(n^2) total work. Use table.insert(result, item) for O(1) amortized append.",
        "complexity::one_iteration_loop" => "A loop that unconditionally returns or breaks on the first iteration executes at most once. Remove the loop wrapper or restructure the logic.",
        "complexity::elseif_chain_over_table" => "Long elseif chains with equality comparisons are O(n) linear scans. A lookup table provides O(1) dispatch: local handlers = {[1] = fn1, [2] = fn2}; handlers[x]()",
        "render::neon_glass_material_in_loop" => "Neon and Glass materials trigger special rendering passes (glow bloom / refraction). Setting these in a loop creates many expensive-to-render parts. Cache the material value outside.",
        "render::surface_gui_in_loop" => "SurfaceGui creation allocates a 3D-to-2D rendering context. In a loop, pre-create a template and use :Clone() for better performance.",
        "physics::can_touch_query_not_disabled" => "CanCollide = false only disables physical collision response. The engine still evaluates CanTouch (Touched events at ~240Hz) and CanQuery (raycast/spatial query hits). Disable both for decorative/non-interactive parts.",
        "physics::weld_constraint_in_loop" => "Each WeldConstraint adds a constraint to the physics solver. Creating many in a loop increases solver iteration time. Pre-create constraints or use WeldConstraint pooling.",
        "memory::sound_not_destroyed" => "Sound instances persist in memory after playback ends. Without cleanup (Ended:Once -> Destroy, or Debris:AddItem), accumulated Sounds cause memory growth and audio system overhead.",
        "memory::unbounded_table_growth" => "table.insert in a per-frame or per-event callback without cleanup creates unbounded memory growth. Add a size limit with table.remove or use a ring buffer pattern.",
        "network::dict_keys_in_remote_data" => "String dictionary keys in RemoteEvent data add bytes per key per packet. For high-frequency updates (Heartbeat), use array-indexed tables {value1, value2} instead of {Key1 = value1}.",
        "network::unreliable_remote_preferred" => "Reliable RemoteEvents in per-frame callbacks guarantee delivery and ordering, consuming bandwidth for data that's immediately superseded. UnreliableRemoteEvent drops stale packets automatically.",
        "network::invoke_client_dangerous" => ":InvokeClient() yields the server thread until the client responds. A malicious or disconnecting client can stall the server indefinitely. Use FireClient + client-to-server response pattern instead.",
        "cache::repeated_color3" => "The same Color3.fromRGB/new call repeated 4+ times wastes constructor calls. Extract to a module-level constant: local RED = Color3.fromRGB(255, 0, 0).",
        "cache::repeated_property_chain" => "Long property chains like player.Character.HumanoidRootPart accessed 3+ times cost multiple GETTABLEKS ops each time. Cache in a local: local rootPart = character.HumanoidRootPart.",
        "cache::enum_lookup_in_loop" => "Enum.Category.Value crosses the Lua-C++ bridge each access. In a loop, cache outside: local material = Enum.Material.SmoothPlastic.",
        "native::method_call_defeats_fastcall" => "Method syntax (:byte, :sub, :len, :char) generates NAMECALL instead of FASTCALL. In loops, use string.byte(s, i) instead of s:byte(i) for the fast builtin path.",
        "native::shared_global_mutation" => "Writing to shared.* (like _G.*) disables GETIMPORT, FASTCALL, and DUPCLOSURE optimizations for the ENTIRE script. Use a required module for cross-script state instead.",
        "native::import_chain_too_deep" => "GETIMPORT caches at most 3 levels of property access (global.a.b). Deeper chains fall back to individual GETTABLEKS instructions. Cache intermediate results in locals.",
        "string::sub_for_prefix_check" => "string.sub(s, 1, n) == prefix allocates a new substring for comparison. string.find(s, prefix, 1, true) == 1 returns a number, avoiding the allocation entirely.",
        "string::pattern_backtracking" => "Patterns with multiple greedy quantifiers (.*/.+) can cause exponential backtracking on non-matching inputs. Simplify patterns or use string.find with plain flag for literal searches.",

        "style::udim2_prefer_from_offset" => "UDim2.new(0, x, 0, y) is equivalent to UDim2.fromOffset(x, y). The fromOffset form is shorter, clearer, and communicates intent better.",
        "style::udim2_prefer_from_scale" => "UDim2.new(sx, 0, sy, 0) is equivalent to UDim2.fromScale(sx, sy). The fromScale form is shorter, clearer, and communicates intent better.",
        "style::tostring_math_floor" => "tostring(math.floor(x)) nests two function calls. Consider storing the floor result first, or using string.format(\"%d\", x) if you just need a truncated integer string.",
        "style::deep_parent_chain" => "script.Parent.Parent.Parent traverses 3+ levels up the instance hierarchy. This is fragile - any reparenting breaks the reference. Use :FindFirstAncestor(name) or store a reference to the root module at the top of your codebase.",
        "style::error_no_level" => "error('msg') without a second argument defaults to level 1, pointing to the error() call itself. Use error('msg', 2) to point to the caller, making the error message more useful for debugging.",
        "style::match_for_existence" => "string.match() allocates captures. When you only check if a pattern exists (in an if condition or ~= nil check), string.find() is faster because it returns indices without allocating.",
        "style::nested_string_format" => "Nested string.format() calls create an intermediate string that's immediately consumed by the outer format. Combine them into a single string.format() call to avoid the intermediate allocation.",

        _ => "No detailed explanation available for this rule. Run --list-rules to see all rules.",
    }
}
