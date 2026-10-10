use engine_core::{FrameBuilder, Vec2};
use game_core::{Game, InputFrame, Profile};
use serde_json::{Value, json};
use std::{collections::{BTreeMap, BTreeSet}, env, fs, path::Path, time::Instant};

const DT: f32 = 1.0 / 60.0;
const OBSERVE_STEPS: usize = 6 * 60;
const FIGHT_STEPS: usize = 120 * 60;

fn qa_profile() -> Profile {
    // A declared high-level fixture, not a reward-granting gameplay command.
    let mut p = Profile::default();
    p.currency = [13, 17, 19];
    p.ranks = [5; 9];
    p.unlocked = [true; 3];
    p.campaign.patches = [true; 8];
    p.campaign.blueprints = [true; 8];
    p.campaign.crafted = [true; 8];
    p.campaign.royale_results = [1; 8];
    p.campaign.victory_claimed = true;
    p.campaign.credits_seen = true;
    p.campaign.selected_ship = 9;
    p.campaign.ship_xp = [1320; 10];
    p
}

fn rewards(p: &Profile) -> Value {
    json!({"currency":p.currency,"ranks":p.ranks,"unlocked":p.unlocked,
        "kills":p.kills,"boss_kills":p.boss_kills,"boss_generations":p.boss_generations,
        "pending":p.pending,"pity":p.pity,"quests":p.quests,"relics":p.relics,
        "ship_xp":p.campaign.ship_xp,"patches":p.campaign.patches,
        "blueprints":p.campaign.blueprints,"crafted":p.campaign.crafted,
        "royale_results":p.campaign.royale_results,"victory_claimed":p.campaign.victory_claimed,
        "cargo_materials":p.campaign.cargo_materials,"stored_materials":p.campaign.stored_materials})
}

fn pair(value: &Value, field: &str) -> Option<Vec2> {
    let coords=value.get(field)?.as_array()?;
    if coords.len()!=2 { return None; }
    let point=Vec2::new(coords[0].as_f64()? as f32,coords[1].as_f64()? as f32);
    point.is_finite().then_some(point)
}

fn render_sample(game: &mut Game, frame: &mut FrameBuilder, counts: &mut BTreeMap<String,(usize,usize)>) {
    for zoom in [0.6_f32,1.6] {
        // This probes extraction at equivalent zoomed world extents. It does not run a GPU.
        game.set_viewport(640.0/zoom,320.0/zoom);
        game.render_into(0.5,frame);
        let rendered=frame.frame();
        assert!(rendered.pixel_count>0,"actual extraction must contain world pixels");
        assert_eq!(rendered.pixel_count,rendered.batches.iter().map(|b|b.pixels.len()).sum::<usize>());
        for batch in &rendered.batches {
            assert!(batch.transforms.len()<=128);
            for transform in &batch.transforms {
                assert!(transform.a.iter().chain(&transform.b).all(|v|v.is_finite()));
                assert!((0.0..=1.0).contains(&transform.b[2]));
                assert!((0.0..=1.0).contains(&transform.b[3]));
            }
            for pixel in &batch.pixels {
                assert!(pixel.local.iter().all(|v|v.is_finite()));
                assert!((pixel.transform as usize)<batch.transforms.len());
            }
        }
        let range=counts.entry(format!("{zoom:.1}")).or_insert((usize::MAX,0));
        range.0=range.0.min(rendered.pixel_count);range.1=range.1.max(rendered.pixel_count);
    }
    game.set_viewport(640.0,320.0);
}

fn scenario(family: &str, generation: u8) -> Value {
    let command=format!("boss-{family}-{generation}");
    let mut game=Game::new(qa_profile());
    game.set_viewport(640.0,320.0);
    let profile_before=serde_json::to_value(game.profile()).unwrap();
    let reward_before=rewards(game.profile());
    game.diagnostic(&command);
    let initial=serde_json::to_value(game.hud()).unwrap();
    let initial_boss=initial.get("boss").filter(|v|!v.is_null()).expect("boss diagnostic must create a live boss");
    assert_eq!(initial_boss["generation"],generation);
    assert!(pair(initial_boss,"position").is_some(),"read-only actual boss position is required for disclosed aiming");
    assert_eq!(rewards(game.profile()),reward_before,"diagnostic setup must not grant or reserve progression");
    let initial_health=initial_boss["health"].as_f64().unwrap();
    let mut min_health=initial_health;
    let mut phases=BTreeSet::new();
    let mut telegraph_steps=0usize;
    let mut events=BTreeMap::<String,usize>::new();
    let mut render_counts=BTreeMap::new();
    let mut frame=FrameBuilder::new();
    let mut defeat=false;
    let mut died=false;
    let mut steps=0usize;
    let mut missing_after=0usize;
    game.take_events();
    for step in 0..OBSERVE_STEPS+FIGHT_STEPS {
        let hud=serde_json::to_value(game.hud()).unwrap();
        let Some(boss)=hud.get("boss").filter(|v|!v.is_null()) else {break;};
        if let Some(phase)=boss["phase"].as_str() {phases.insert(phase.to_owned());}
        let telegraph=boss["telegraph"].as_bool().unwrap_or(false);
        if telegraph {telegraph_steps+=1;}
        min_health=min_health.min(boss["health"].as_f64().unwrap());
        let exposed=boss["phase"].as_str()==Some("Core exposed")&&!telegraph;
        let target=pair(boss,if exposed {"core_position"} else {"position"}).expect("boss aiming telemetry must be finite");
        let player=pair(&hud,"position").unwrap();
        let delta=target-player;
        let direction=delta.normalize_or_zero();
        // 0.081 exceeds the real joystick deadzone only slightly: turn toward the
        // disclosed moving target while minimizing thrust. All shots use the
        // actual primary/signature weapon timers and swept occupied-cell hits.
        // Follow retreating siege patterns at normal thrust if they exceed the
        // disclosed 420-unit range, then return to minimal turning thrust.
        let stick_power=if delta.length()>420.0 {0.35}else{0.081};
        let input=InputFrame {joystick:Some((direction*stick_power).to_array()),fire:step>=OBSERVE_STEPS,..Default::default()};
        game.tick(&input,DT);steps+=1;
        let mut boss_explosion_this_step=false;
        for event in game.take_events() {
            *events.entry(event.kind.clone()).or_default()+=1;
            if event.kind=="death" {died=true;}
            if event.kind=="explosion"&&event.intensity>=1.0&&Vec2::new(event.x,event.y).distance(target)<100.0 {
                boss_explosion_this_step=true;
            }
        }
        let after=serde_json::to_value(game.hud()).unwrap();
        if after["boss"].is_null() {
            missing_after+=1;
            defeat=!died&&boss_explosion_this_step&&min_health<initial_health;
            render_sample(&mut game,&mut frame,&mut render_counts);
            break;
        }
        if step%60==0 {render_sample(&mut game,&mut frame,&mut render_counts);}
    }
    let reward_after=rewards(game.profile());
    let reward_isolated=reward_after==reward_before;
    let profile_after=serde_json::to_value(game.profile()).unwrap();
    let profile_isolated=profile_before==profile_after;
    game.reset_to_dock();
    let counters_at_retreat=rewards(game.profile());
    let dock_hull=game.hud().hull;
    for _ in 0..600 {game.tick(&InputFrame{fire:true,..Default::default()},DT);}
    let sanctuary_safe=game.hud().safe&&game.hud().boss.is_none()&&game.hud().hull>=dock_hull;
    assert!(sanctuary_safe,"sanctuary must stay safe after boss cleanup");
    assert_eq!(rewards(game.profile()),counters_at_retreat,"ten seconds at Haven must not mutate boss rewards");
    let complete=defeat&&reward_isolated&&profile_isolated&&sanctuary_safe;
    println!("{}: {} after {:.2}s; phases {:?}; telegraph {} steps; rewards {}; sanctuary {}",command,
        if defeat{"actual projectile combat defeat"}else if died{"player died"}else{"finite budget exhausted or disengaged"},
        steps as f32*DT,phases,telegraph_steps,if reward_isolated{"unchanged"}else{"CHANGED"},sanctuary_safe);
    json!({"command":command,"complete":complete,"actual_defeat":defeat,"player_died":died,
        "simulation_seconds":steps as f32*DT,"initial_hull":initial_health,"minimum_observed_hull":min_health,
        "phases":phases,"telegraph_steps":telegraph_steps,"events":events,"missing_boss_observations":missing_after,
        "quad_ranges_by_world_zoom_extent":render_counts,"reward_isolated":reward_isolated,"full_profile_isolated":profile_isolated,
        "sanctuary_safe":sanctuary_safe,"profile_before":profile_before,"profile_after":profile_after,
        "reward_before":reward_before,"reward_after":reward_after})
}

fn seek_queued_boss(game:&mut Game)->Option<Value> {
    game.take_events();
    for _ in 0..2400 {
        game.tick(&InputFrame {destination:Some([1800.0,0.0]),..Default::default()},DT);
        let hud=serde_json::to_value(game.hud()).unwrap();
        if !hud["boss"].is_null() {return Some(hud["boss"].clone());}
    }
    None
}

fn queued_retry_probe()->Value {
    let mut p=Profile::default();
    p.kills[1]=408;p.pending[1]=true;p.boss_generations[1]=[16,8,4,2,0];p.boss_kills[1]=30;
    let mut game=Game::new(p);
    game.set_viewport(640.0,320.0);
    let before=game.profile().clone();
    let first=seek_queued_boss(&mut game).expect("saved promoted queue must create a real fight after flying out");
    assert_eq!(first["generation"],5);assert_eq!(first["family"],1);
    assert_eq!(rewards(game.profile()),rewards(&before),"spawning must not consume promotion credit");
    game.reset_to_dock();
    let retreat=game.profile().clone();
    assert_eq!(rewards(&retreat),rewards(&before),"returning home must retain reserved and promoted queues");
    let mut reloaded=Game::new(retreat.clone());reloaded.set_viewport(640.0,320.0);
    assert_eq!(rewards(reloaded.profile()),rewards(&before),"reload must retain the source counters");
    let retry=seek_queued_boss(&mut reloaded).expect("promoted fight must retry after reload");
    assert_eq!(retry["generation"],5);
    // Reset destination via a centered joystick, then let actual hostile AI,
    // projectiles and collisions defeat the unupgraded Scout. No forced die call.
    let mut actual_death=false;let mut death_seconds=0.0;
    reloaded.take_events();
    for step in 0..180*60 {
        reloaded.tick(&InputFrame {joystick:Some([0.0,0.0]),..Default::default()},DT);
        if reloaded.take_events().iter().any(|event|event.kind=="death") {
            actual_death=true;death_seconds=(step+1) as f32*DT;break;
        }
    }
    assert!(actual_death,"finite hostile-combat probe must observe an actual player death");
    assert_eq!(rewards(reloaded.profile()),rewards(&before),"actual loss must not consume or reward a boss defeat");
    let after_loss=serde_json::to_value(reloaded.profile()).unwrap();
    let next=seek_queued_boss(&mut reloaded).expect("actual death must leave promoted fight retryable");
    assert_eq!(next["generation"],5);
    println!("Native queue proof: G5 spawn -> retreat -> reload -> actual hostile death at {death_seconds:.2}s -> G5 retry; counters unchanged.");
    json!({"family":"bastion","generation":5,"source_defeats":[16,8,4,2,0],"aggregate":30,
        "ordinary_kills":408,"reserved_generation_one":true,"generation_five_queue":1,
        "actual_hostile_death":actual_death,"death_seconds":death_seconds,
        "profile_before":before,"profile_after_retreat":retreat,"profile_after_actual_loss":after_loss,
        "retry_hud":next,"passed":true})
}

fn main() {
    let args:Vec<String>=env::args().collect();
    let output=args.get(1).map(String::as_str).unwrap_or("work/boss-live-combat.json");
    let started=Instant::now();
    let mut cases=Vec::new();
    for family in ["dart","bastion","coil"] {for generation in 1..=5 {cases.push(scenario(family,generation));}}
    let queue_retry=queued_retry_probe();
    let passed=cases.iter().filter(|case|case["complete"]==true).count();
    let report=json!({"scope":"actual CPU gameplay/render extraction; no GPU/browser/device validation",
        "fixture":"Declared Royale Sovereign, all hardware/signatures rank5, level12 XP; starting currency13/17/19, no ordinary boss defeats",
        "input":"Read-only actual BossHud position/core_position; joystick0.081 aiming or0.35 pursuit beyond420units; actual held fire; no damage/kill/grant helpers",
        "fixed_step_seconds":DT,"observation_seconds":OBSERVE_STEPS as f32*DT,
        "combat_budget_seconds":FIGHT_STEPS as f32*DT,"base_world_viewport":[640,320],
        "render_zoom_extents":[0.6,1.6],"physical_device":"none","passed":passed,
        "total":cases.len(),"wall_seconds":started.elapsed().as_secs_f64(),"cases":cases,
        "queue_retry_probe":queue_retry});
    if let Some(parent)=Path::new(output).parent() {fs::create_dir_all(parent).unwrap();}
    fs::write(output,serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("Actual native diagnostic-combat results: {passed}/{} completed with unchanged rewards; report {output}",cases.len());
    if passed!=cases.len() {std::process::exit(1);}
}
