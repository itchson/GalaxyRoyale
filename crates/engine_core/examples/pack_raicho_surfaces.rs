//! Offline authoring conversion through the real portable Surface encoder.
use engine_core::Surface;
use serde_json::{Value, json};
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: pack_raicho_surfaces <semantic-json> <packed-json>".into());
    }
    let source: Value = serde_json::from_slice(&fs::read(&args[0])?)?;
    if source["version"] != 1 || source["asset"] != "pan_hikari" {
        return Err("expected version-one PAN semantic field".into());
    }
    let mut groups = Vec::new();
    for (name, size) in [("coarse48", 48), ("fine96", 96)] {
        let group = &source[name];
        let cells = group["cells"].as_array().ok_or("missing surface cells")?;
        if group["width"] != size || group["height"] != size || cells.len() != size * size {
            return Err("surface dimensions or field length mismatch".into());
        }
        let mut packed = Vec::new();
        for (index, cell) in cells.iter().enumerate() {
            if cell.is_null() {
                continue;
            }
            let values = cell.as_array().ok_or("surface cell must be a tuple")?;
            if values.len() != 4 {
                return Err("surface tuple requires normal and mask".into());
            }
            let mut normal = [0.0; 3];
            for axis in 0..3 {
                normal[axis] = values[axis]
                    .as_f64()
                    .ok_or("normal component must be numeric")?
                    as f32;
            }
            let mask = u8::try_from(values[3].as_u64().ok_or("mask must be an integer")?)?;
            let surface = Surface::new(normal, mask)?;
            packed.push([index as u32, surface.packed()]);
        }
        groups.push(packed);
    }
    let output = json!({
        "version":1,"asset":"pan_hikari","source_atlas_sha256":source["source_atlas_sha256"],
        "source_atlas_fnv64":format!("{:016x}", include_bytes!("../../../assets/pixels/raicho-fine-detail/atlas.json")
            .iter().fold(0xcbf29ce484222325_u64, |hash, &byte| (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3))),
        "encoding":"oct8xy/linear-mask8/enabled; seven transform bits remain zero",
        "coarse":groups[0],"fine":groups[1]
    });
    let path = PathBuf::from(&args[1]);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_vec(&output)?)?;
    println!(
        "native packed: {} coarse / {} fine; output {} bytes",
        groups[0].len(),
        groups[1].len(),
        fs::metadata(path)?.len()
    );
    Ok(())
}
