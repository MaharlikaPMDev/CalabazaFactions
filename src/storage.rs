use crate::{config::Config, domain::FactionState};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn load(data_dir: &Path) -> Result<(Config, FactionState), String> {
    fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    let config_path = seed(
        data_dir,
        "config.toml",
        include_str!("../config/factions.toml"),
    )?;
    let config = toml::from_str(&fs::read_to_string(config_path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("invalid config.toml: {e}"))?;
    let state_path = data_dir.join("state.json");
    let state = if state_path.exists() {
        serde_json::from_str(&fs::read_to_string(&state_path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("invalid state.json: {e}"))?
    } else {
        FactionState::default()
    };
    Ok((config, state))
}

fn seed(dir: &Path, name: &str, contents: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    if !path.exists() {
        fs::write(&path, contents).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

pub fn save(data_dir: &Path, state: &FactionState) -> Result<(), String> {
    let target = data_dir.join("state.json");
    let temp = data_dir.join("state.json.tmp");
    let backup = data_dir.join("state.json.bak");
    let bytes = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    fs::write(&temp, bytes).map_err(|e| e.to_string())?;
    if target.exists() {
        let _ = fs::copy(&target, &backup);
    }
    fs::rename(&temp, &target).map_err(|e| e.to_string())?;
    let public = serde_json::json!({"schema_version":state.schema_version,"player_faction":state.player_faction,"factions":state.factions.iter().map(|(id,f)|(id.clone(),serde_json::json!({"name":f.name,"members":f.members,"power":f.power,"bank":f.bank}))).collect::<serde_json::Map<_,_>>()});
    fs::write(
        data_dir.join("api.json"),
        serde_json::to_vec_pretty(&public).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
