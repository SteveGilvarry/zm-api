use super::hash;
use crate::error::{invalid_input_error, AppResult};
use bcrypt::verify as bcrypt_verify_bool;
use once_cell::sync::Lazy;
use tracing::debug;

pub async fn hash(password: String) -> AppResult<String> {
    let jh = tokio::task::spawn_blocking(move || hash::bcrypt_hash(password));
    let password = jh.await??;
    Ok(password)
}

/// Cached bcrypt hash of an arbitrary throwaway string, computed once on
/// first access. Used by [`verify_existing_or_dummy`] so a failed user
/// lookup still pays bcrypt's cost — closes the timing oracle that would
/// otherwise let an attacker enumerate valid usernames by measuring login
/// response latency.
static DUMMY_HASH: Lazy<String> =
    Lazy::new(|| hash::bcrypt_hash("not-a-real-password").expect("dummy bcrypt hash"));

/// Verify `password` against the user's hash if one was supplied;
/// otherwise verify against a fixed dummy hash so the call still spends
/// the same wall-clock cost. Returns `true` only when a real hash was
/// supplied AND it matched.
pub async fn verify_existing_or_dummy(password: String, user_hash: Option<String>) -> bool {
    let had_real_hash = user_hash.is_some();
    let hash_to_check = user_hash.unwrap_or_else(|| DUMMY_HASH.clone());
    let jh = tokio::task::spawn_blocking(move || bcrypt_verify_bool(password, &hash_to_check));
    matches!(jh.await, Ok(Ok(true))) && had_real_hash
}

pub async fn verify(password: String, hashed_pass: String) -> AppResult {
    let jh = tokio::task::spawn_blocking(move || bcrypt_verify_bool(password, &hashed_pass));
    match jh.await? {
        Ok(true) => Ok(()),
        Ok(false) => {
            debug!("The password is not correct");
            Err(invalid_input_error(
                "password",
                "The password is not correct.",
            ))
        }
        Err(e) => {
            debug!("Password verification error: {e}");
            Err(invalid_input_error(
                "password",
                "The password is not correct.",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

    #[tokio::test]
    pub async fn test_password_hash() {
        let password: String = Faker.fake();
        let hash_pass = hash(password).await.unwrap();
        assert!(!hash_pass.is_empty());
    }

    #[tokio::test]
    pub async fn test_password_hash_and_then_verify_it() {
        let password: String = Faker.fake();
        let hash_pass = hash(password.clone()).await.unwrap();
        verify(password, hash_pass).await.unwrap();
    }
}
