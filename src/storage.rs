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
    let config: Config =
        toml::from_str(&fs::read_to_string(config_path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("invalid config.toml: {e}"))?;
    if !config.storage.backend.eq_ignore_ascii_case("json") {
        return Err(format!(
            "storage backend '{}' is not available in the WASI build; use json",
            config.storage.backend
        ));
    }
    let state_path = data_dir.join("state.json");
    let backup_path = data_dir.join("state.json.bak");
    let mut state = if state_path.exists() {
        match read_state(&state_path) {
            Ok(state) => state,
            Err(primary_error) if backup_path.exists() => {
                let recovered = read_state(&backup_path).map_err(|e| {
                    format!("invalid state.json ({primary_error}) and backup ({e})")
                })?;
                let corrupt_path = data_dir.join("state.json.corrupt");
                if corrupt_path.exists() {
                    fs::remove_file(&corrupt_path).map_err(|e| e.to_string())?;
                }
                fs::rename(&state_path, &corrupt_path).map_err(|e| e.to_string())?;
                fs::copy(&backup_path, &state_path).map_err(|e| e.to_string())?;
                recovered
            }
            Err(error) => return Err(format!("invalid state.json: {error}")),
        }
    } else {
        FactionState::default()
    };
    state.migrate();
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
    replace_with_backup(&temp, &target, &backup)?;
    let public = serde_json::json!({
        "schema_version": state.schema_version,
        "player_faction": state.player_faction,
        "factions": state.factions.iter().map(|(id, faction)| (
            id.clone(),
            serde_json::json!({
                "name": faction.name,
                "members": faction.members,
                "power": faction.power,
                "bank": faction.bank,
                "core_lifecycle": faction.core_lifecycle,
                "physical_core": faction.physical_core,
                "claims": faction.claims,
            })
        )).collect::<serde_json::Map<_, _>>()
    });
    let api_target = data_dir.join("api.json");
    let api_temp = data_dir.join("api.json.tmp");
    let api_backup = data_dir.join("api.json.bak");
    fs::write(
        &api_temp,
        serde_json::to_vec_pretty(&public).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    replace_with_backup(&api_temp, &api_target, &api_backup)
}

fn read_state(path: &Path) -> Result<FactionState, String> {
    serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn replace_with_backup(temp: &Path, target: &Path, backup: &Path) -> Result<(), String> {
    if backup.exists() {
        fs::remove_file(backup).map_err(|e| e.to_string())?;
    }
    if target.exists() {
        fs::rename(target, backup).map_err(|e| e.to_string())?;
    }
    if let Err(error) = fs::rename(temp, target) {
        if backup.exists() {
            let _ = fs::rename(backup, target);
        }
        return Err(error.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("calabaza-{name}-{nonce}"))
    }

    #[test]
    fn save_keeps_previous_state_as_valid_backup() {
        let dir = test_dir("backup");
        fs::create_dir_all(&dir).unwrap();
        let first = FactionState {
            next_mail_id: 7,
            ..Default::default()
        };
        save(&dir, &first).unwrap();
        let mut second = first.clone();
        second.next_mail_id = 8;
        save(&dir, &second).unwrap();

        let backup: FactionState =
            serde_json::from_str(&fs::read_to_string(dir.join("state.json.bak")).unwrap()).unwrap();
        assert_eq!(backup.next_mail_id, 7);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_recovers_from_corrupt_primary_using_backup() {
        let dir = test_dir("recovery");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("config.toml"),
            include_str!("../config/factions.toml"),
        )
        .unwrap();
        let state = FactionState {
            next_mail_id: 42,
            ..Default::default()
        };
        fs::write(
            dir.join("state.json.bak"),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        fs::write(dir.join("state.json"), b"{interrupted").unwrap();

        let (_, recovered) = load(&dir).unwrap();
        assert_eq!(recovered.next_mail_id, 42);
        assert!(dir.join("state.json.corrupt").exists());
        let _ = fs::remove_dir_all(dir);
    }
}
