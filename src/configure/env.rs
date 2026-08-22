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

#[cfg(test)]
mod tests {
    use super::*;

    /// Resolve one variable through the real source, using `Environment::source`
    /// rather than the process environment (which is global and races other
    /// tests).
    fn resolve(var: &str, key: &str) -> Option<String> {
        config::Config::builder()
            .add_source(
                get_env_source("APP").source(Some(
                    [(var.to_string(), "10.0.0.5".to_string())]
                        .into_iter()
                        .collect(),
                )),
            )
            .build()
            .expect("build config")
            .get::<String>(key)
            .ok()
    }

    #[test]
    fn documented_single_underscore_prefix_is_honoured() {
        // GH #38: the prefix separator was `__`, so the documented
        // `APP_DB__HOST` was silently ignored — the server connected with the
        // default credentials and gave no indication the override was dropped.
        assert_eq!(
            resolve("APP_DB__HOST", "db.host").as_deref(),
            Some("10.0.0.5")
        );
    }

    #[test]
    fn double_underscore_prefix_no_longer_matches() {
        // The accidental form must not keep working, or the two spellings
        // silently disagree about which wins.
        assert_eq!(resolve("APP__DB__HOST", "db.host"), None);
    }
}
