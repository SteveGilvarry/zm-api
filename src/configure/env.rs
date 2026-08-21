use std::str::FromStr;

use super::Profile;
use config::ConfigError;

pub fn get_env_source(prefix: &str) -> config::Environment {
    // Prefix joins with a single underscore (`APP_`), nested keys with a double
    // underscore (`__`): `APP_DB__HOST` → `db.host`, matching the documented
    // form in CLAUDE.md / docs. The prefix separator was previously `__`, so the
    // documented `APP_DB__HOST` was silently ignored and only `APP__DB__HOST`
    // worked (GH #38).
    config::Environment::with_prefix(prefix)
        .prefix_separator("_")
        .separator("__")
}

pub fn get_profile() -> Result<Profile, config::ConfigError> {
    std::env::var("APP_PROFILE")
        .map(|env| Profile::from_str(&env).map_err(|e| ConfigError::Message(e.to_string())))
        .unwrap_or_else(|_e| Ok(Profile::Dev))
}
