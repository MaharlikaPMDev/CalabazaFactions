use crate::{
    config::{EconomyConfig, EconomyMode},
    domain::FactionState,
};
use pumpkin_plugin_api::ipc;
use serde_json::{Value, json};

pub const CALABAZA_BANK_CONTRACT_VERSION: u32 = 1;

pub fn balance(config: &EconomyConfig, state: &FactionState, player: &str) -> Result<i64, String> {
    match config.mode {
        EconomyMode::Standalone => Ok(state.wallets.get(player).copied().unwrap_or(0)),
        EconomyMode::External => {
            let response = call(
                config,
                json!({
                    "schema": "calabazabank.ipc",
                    "version": CALABAZA_BANK_CONTRACT_VERSION,
                    "action": "balance",
                    "account_id": player,
                }),
            )?;
            response
                .get("balance")
                .and_then(Value::as_i64)
                .ok_or_else(|| "CalabazaBank returned no integer balance".into())
        }
    }
}

pub fn debit(
    config: &EconomyConfig,
    state: &mut FactionState,
    player: &str,
    amount: i64,
    transaction_id: &str,
    reason: &str,
) -> Result<(), String> {
    if amount <= 0 {
        return Err("amount must be positive".into());
    }
    match config.mode {
        EconomyMode::Standalone => {
            let wallet = state.wallets.entry(player.into()).or_default();
            if *wallet < amount {
                return Err("insufficient wallet balance".into());
            }
            *wallet -= amount;
            Ok(())
        }
        EconomyMode::External => {
            call(
                config,
                json!({
                    "schema": "calabazabank.ipc",
                    "version": CALABAZA_BANK_CONTRACT_VERSION,
                    "action": "debit",
                    "account_id": player,
                    "amount": amount,
                    "transaction_id": transaction_id,
                    "reason": reason,
                }),
            )?;
            Ok(())
        }
    }
}

pub fn credit(
    config: &EconomyConfig,
    state: &mut FactionState,
    player: &str,
    amount: i64,
    transaction_id: &str,
    reason: &str,
) -> Result<(), String> {
    if amount <= 0 {
        return Err("amount must be positive".into());
    }
    match config.mode {
        EconomyMode::Standalone => {
            *state.wallets.entry(player.into()).or_default() += amount;
            Ok(())
        }
        EconomyMode::External => {
            call(
                config,
                json!({
                    "schema": "calabazabank.ipc",
                    "version": CALABAZA_BANK_CONTRACT_VERSION,
                    "action": "credit",
                    "account_id": player,
                    "amount": amount,
                    "transaction_id": transaction_id,
                    "reason": reason,
                }),
            )?;
            Ok(())
        }
    }
}

pub fn health(config: &EconomyConfig) -> Result<(), String> {
    if config.mode == EconomyMode::Standalone {
        return Ok(());
    }
    call(
        config,
        json!({
            "schema": "calabazabank.ipc",
            "version": CALABAZA_BANK_CONTRACT_VERSION,
            "action": "health",
            "consumer": "CalabazaFactions",
        }),
    )?;
    Ok(())
}

fn call(config: &EconomyConfig, request: Value) -> Result<Value, String> {
    let payload = serde_json::to_vec(&request).map_err(|error| error.to_string())?;
    let response = ipc::send_ipc_message(&config.external_plugin, &payload)
        .map_err(|()| {
            format!(
                "CalabazaBank plugin '{}' is unavailable",
                config.external_plugin
            )
        })?
        .map_err(|error| format!("CalabazaBank rejected the request: {error}"))?;
    let response: Value = serde_json::from_slice(&response)
        .map_err(|error| format!("invalid CalabazaBank response: {error}"))?;
    if response.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(response
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("CalabazaBank operation failed")
            .to_string());
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_debit_and_credit_are_local() {
        let config = EconomyConfig::default();
        let mut state = FactionState::default();
        state.wallets.insert("player".into(), 100);
        debit(&config, &mut state, "player", 30, "tx-1", "test").unwrap();
        credit(&config, &mut state, "player", 10, "tx-2", "test").unwrap();
        assert_eq!(balance(&config, &state, "player").unwrap(), 80);
    }
}
