//! Offline opt-in metadata adapter proving actual engine cell extraction.
//! This utility is not part of the shipped game or its automatic asset loader.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Vec2};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::HashMap, path::Path};
#[derive(Deserialize)]
struct Pack {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
    asset_metadata: Metadata,
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct Metadata {
    groups: Vec<Group>,
    animations: Vec<Animation>,
    budgets: Budgets,
}
#[derive(Deserialize)]
struct Budgets {
    group_cells: usize,
    group_transforms: usize,
    extent: f32,
}
#[derive(Deserialize)]
struct Group {
    name: String,
    parts: Vec<Part>,
}
#[derive(Deserialize)]
struct Part {
    name: String,
    sprite: String,
    parent: Option<String>,
    #[serde(default)]
    transform: Placement,
}
#[derive(Deserialize)]
#[serde(default)]
struct Placement {
    position: [f32; 2],
    basis_x: [f32; 2],
    basis_y: [f32; 2],
}
impl Default for Placement {
    fn default() -> Self {
        Self {
            position: [0., 0.],
            basis_x: [1., 0.],
            basis_y: [0., 1.],
        }
    }
}
#[derive(Deserialize)]
struct Animation {
    name: String,
    frames: Vec<Step>,
    #[serde(rename = "loop")]
    looping: bool,
}
#[derive(Deserialize)]
struct Step {
    kind: String,
    name: String,
    duration_ms: u32,
}

fn frame_at(animation: &Animation, elapsed_ms: u64) -> Result<usize, String> {
    if animation.frames.is_empty()
        || animation.frames.len() > 256
        || animation
            .frames
            .iter()
            .any(|f| !(1..=60000).contains(&f.duration_ms))
    {
        return Err("Invalid animation timeline".into());
    }
    let total: u64 = animation.frames.iter().map(|f| f.duration_ms as u64).sum();
    let mut time = if animation.looping {
        elapsed_ms % total
    } else {
        elapsed_ms.min(total - 1)
    };
    for (i, frame) in animation.frames.iter().enumerate() {
        if time < frame.duration_ms as u64 {
            return Ok(i);
        }
        time -= frame.duration_ms as u64;
    }
    Err("Invalid frame selection".into())
}

fn body(s: &Sprite, palette: &[u32]) -> Result<PixelBody, String> {
    if !(1..=256).contains(&s.width)
        || !(1..=256).contains(&s.height)
        || s.indices.len() != s.width * s.height
        || s.indices.iter().any(|&i| i as usize >= palette.len())
    {
        return Err("Invalid packed sprite cells".into());
    }
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let colors: Vec<_> = palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (key(i), c))
        .collect();
    let rows: Vec<String> = s
        .indices
        .chunks_exact(s.width)
        .map(|row| {
            row.iter()
                .map(|&i| if i == 0 { ' ' } else { key(i as usize) })
                .collect()
        })
        .collect();
    let rows: Vec<_> = rows.iter().map(String::as_str).collect();
    let body = PixelBody::from_ascii(&rows, &colors);
    if body.occupied_count() == 0 {
        return Err("Empty sprite".into());
    }
    Ok(body)
}
fn extract(pack: &Pack, group: &Group) -> Result<Value, String> {
    if group.parts.is_empty() {
        return Err("Empty group".into());
    }
    let mut prepared = Vec::new();
    let mut known = HashMap::new();
    let mut count = 0;
    for p in &group.parts {
        if known.contains_key(&p.name) {
            return Err("Duplicate part".into());
        }
        let b = body(
            pack.sprites.get(&p.sprite).ok_or("Unknown sprite")?,
            &pack.palette,
        )?;
        let local = AffineTransform {
            position: Vec2::from_array(p.transform.position),
            basis_x: Vec2::from_array(p.transform.basis_x),
            basis_y: Vec2::from_array(p.transform.basis_y),
        };
        let transform = if let Some(parent) = &p.parent {
            let t: AffineTransform = *known.get(parent).ok_or("Parent must be earlier")?;
            t.compose(local)
        } else {
            local
        };
        if !transform.position.is_finite()
            || !transform.basis_x.is_finite()
            || !transform.basis_y.is_finite()
        {
            return Err("Nonfinite transform".into());
        }
        count += b.occupied_count();
        if count > pack.asset_metadata.budgets.group_cells
            || prepared.len() + 1 > pack.asset_metadata.budgets.group_transforms
        {
            return Err("Atomic group budget exceeded".into());
        }
        for (cell, _) in b.cells() {
            for offset in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ] {
                let point = transform.point(cell + offset);
                if !point.is_finite()
                    || point.abs().max_element() > pack.asset_metadata.budgets.extent
                {
                    return Err("Occupied corners exceed extent".into());
                }
            }
        }
        known.insert(p.name.clone(), transform);
        prepared.push((b, transform));
    }
    // No append happens until every child passes the complete group preflight.
    let mut frame = FrameBuilder::new();
    for (body, t) in &prepared {
        frame.push_body_affine_colored(body, *t, |_, c| c);
    }
    let frame = frame.finish();
    if frame.pixel_count != count {
        return Err("Engine extraction occupancy mismatch".into());
    }
    let cells: Vec<_> = frame
        .batches
        .iter()
        .flat_map(|batch| {
            batch.pixels.iter().map(|p| {
                let t = batch.transforms[p.transform as usize];
                let c = Vec2::from_array(p.local);
                let corners = [
                    Vec2::splat(-0.5),
                    Vec2::new(0.5, -0.5),
                    Vec2::splat(0.5),
                    Vec2::new(-0.5, 0.5),
                ]
                .map(|d| t.point(c + d).to_array());
                json!({"corners":corners,"color":p.color})
            })
        })
        .collect();
    let corners: Vec<_> = cells
        .iter()
        .flat_map(|c| c["corners"].as_array().unwrap().iter())
        .collect();
    let axis = |i: usize, max: bool| {
        corners.iter().map(|p| p[i].as_f64().unwrap()).fold(
            if max {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            },
            if max { f64::max } else { f64::min },
        )
    };
    Ok(
        json!({"group":group.name,"occupied_cells":count,"transforms":prepared.len(),"batches":frame.batches.len(),"bounds":[axis(0,false),axis(1,false),axis(0,true),axis(1,true)],"cells":cells}),
    )
}
fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 4 {
        return Err(
            "Usage: asset-native-preview <pack.json> <animation-name> <output.json>".into(),
        );
    }
    let path = Path::new(&args[1]);
    if std::fs::canonicalize(path).ok() == std::fs::canonicalize(&args[3]).ok() && path.exists() {
        return Err("Input and output must differ".into());
    }
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 32_000_000 {
        return Err("Input exceeds work bound".into());
    }
    let pack: Pack =
        serde_json::from_str(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if pack.version != 1
        || !(2..=256).contains(&pack.palette.len())
        || pack.palette[0] != 0
        || pack.palette[1..].iter().any(|c| c >> 24 != 255)
    {
        return Err("Invalid version/palette".into());
    }
    if !(1..=60_000).contains(&pack.asset_metadata.budgets.group_cells)
        || !(1..=256).contains(&pack.asset_metadata.budgets.group_transforms)
        || !pack.asset_metadata.budgets.extent.is_finite()
        || !(1.0..=1_000_000.0).contains(&pack.asset_metadata.budgets.extent)
    {
        return Err("Invalid native budgets".into());
    }
    let animation = pack
        .asset_metadata
        .animations
        .iter()
        .find(|a| a.name == args[2])
        .ok_or("Unknown animation")?;
    if animation.frames.is_empty() || animation.frames.len() > 256 {
        return Err("Animation frame bound".into());
    }
    let mut frames = Vec::new();
    let mut extracted_cells = 0;
    let mut sample_time_ms = 0;
    for _ in 0..animation.frames.len() {
        let step = &animation.frames[frame_at(animation, sample_time_ms)?];
        if step.kind != "group" || !(1..=60000).contains(&step.duration_ms) {
            return Err("Native preview supports timed group frames only".into());
        }
        let group = pack
            .asset_metadata
            .groups
            .iter()
            .find(|g| g.name == step.name)
            .ok_or("Unknown frame group")?;
        let count: usize = group
            .parts
            .iter()
            .map(|p| {
                pack.sprites
                    .get(&p.sprite)
                    .map(|s| s.indices.iter().filter(|&&i| i != 0).count())
                    .unwrap_or(0)
            })
            .sum();
        extracted_cells += count;
        if extracted_cells > 240_000 {
            return Err("Total native preview cell-work bound exceeded".into());
        }
        let mut frame = extract(&pack, group)?;
        frame["duration_ms"] = step.duration_ms.into();
        frame["sample_time_ms"] = sample_time_ms.into();
        sample_time_ms += step.duration_ms as u64;
        frames.push(frame);
    }
    let output = json!({"scope":"actual native PixelBody/FrameBuilder extraction; offline animation adapter only","animation":animation.name,"frames":frames});
    if let Some(parent) = Path::new(&args[3])
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        &args[3],
        serde_json::to_string_pretty(&output).unwrap() + "\n",
    )
    .map_err(|e| e.to_string())?;
    println!("Extracted {} actual native frames", animation.frames.len());
    Ok(())
}
fn main() {
    if let Err(e) = run(&std::env::args().collect::<Vec<_>>()) {
        eprintln!("{e}");
        std::process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pack() -> Pack {
        Pack {
            version: 1,
            palette: vec![0, 0xffffffff],
            sprites: HashMap::from([(
                "cell".into(),
                Sprite {
                    width: 1,
                    height: 1,
                    indices: vec![1],
                },
            )]),
            asset_metadata: Metadata {
                groups: vec![],
                animations: vec![],
                budgets: Budgets {
                    group_cells: 1000,
                    group_transforms: 256,
                    extent: 1_000_000.,
                },
            },
        }
    }
    #[test]
    fn actual_group_rolls_over_128_transform_slots_without_losing_cells() {
        let p = pack();
        let g = Group {
            name: "rollover".into(),
            parts: (0..129)
                .map(|i| Part {
                    name: format!("cell{i}"),
                    sprite: "cell".into(),
                    parent: None,
                    transform: Placement {
                        position: [i as f32, 0.],
                        ..Default::default()
                    },
                })
                .collect(),
        };
        let f = extract(&p, &g).unwrap();
        assert_eq!(f["occupied_cells"], 129);
        assert_eq!(f["batches"], 2);
        assert_eq!(f["bounds"], json!([-0.5, -0.5, 128.5, 0.5]));
    }
    #[test]
    fn host_owned_animation_selects_exact_boundaries_without_game_state() {
        let mut a = Animation {
            name: "pulse".into(),
            frames: vec![
                Step {
                    kind: "group".into(),
                    name: "a".into(),
                    duration_ms: 600,
                },
                Step {
                    kind: "group".into(),
                    name: "b".into(),
                    duration_ms: 400,
                },
            ],
            looping: true,
        };
        for (time, index) in [(0, 0), (599, 0), (600, 1), (999, 1), (1000, 0), (1600, 1)] {
            assert_eq!(frame_at(&a, time).unwrap(), index);
        }
        a.looping = false;
        assert_eq!(frame_at(&a, 1000).unwrap(), 1);
        assert_eq!(frame_at(&a, u64::MAX).unwrap(), 1);
        a.frames[0].duration_ms = 0;
        assert!(frame_at(&a, 0).is_err());
    }
    #[test]
    fn transparent_body_cannot_produce_nonfinite_native_bounds() {
        assert!(
            body(
                &Sprite {
                    width: 1,
                    height: 1,
                    indices: vec![0]
                },
                &[0, 0xffffffff]
            )
            .is_err()
        );
    }
    #[test]
    fn last_child_budget_failure_rejects_group_before_emission() {
        let mut p = pack();
        p.asset_metadata.budgets.group_cells = 1;
        let g = Group {
            name: "rejected".into(),
            parts: (0..2)
                .map(|i| Part {
                    name: i.to_string(),
                    sprite: "cell".into(),
                    parent: None,
                    transform: Default::default(),
                })
                .collect(),
        };
        assert_eq!(extract(&p, &g).unwrap_err(), "Atomic group budget exceeded");
    }
    #[test]
    fn actual_affine_corners_preserve_nested_reflection_and_shear() {
        let p = pack();
        let g = Group {
            name: "nested".into(),
            parts: vec![
                Part {
                    name: "a".into(),
                    sprite: "cell".into(),
                    parent: None,
                    transform: Placement {
                        position: [10., 20.],
                        basis_x: [-2., 0.],
                        basis_y: [1., 3.],
                    },
                },
                Part {
                    name: "b".into(),
                    sprite: "cell".into(),
                    parent: Some("a".into()),
                    transform: Placement {
                        position: [4., 1.],
                        basis_x: [1., 1.],
                        basis_y: [0., 0.5],
                    },
                },
            ],
        };
        let f = extract(&p, &g).unwrap();
        assert_eq!(f["occupied_cells"], 2);
        assert_eq!(f["bounds"], json!([2.25, 18.5, 11.5, 25.25]));
    }
}
