//! Serde helpers for request DTOs.

use serde::{Deserialize, Deserializer};

/// For `Option<Option<T>>` update fields: a missing key is `None` (leave the
/// column alone), `null` is `Some(None)` (clear it), a value is
/// `Some(Some(v))`. Plain serde reads `null` as `None`, so a nullable column
/// could never be cleared. Use with `#[serde(default, deserialize_with = ...)]`.
pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(de).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Patch {
        #[serde(default, deserialize_with = "double_option")]
        v: Option<Option<u32>>,
    }

    /// Why the helper exists: without it `null` is indistinguishable from a
    /// missing key.
    #[test]
    fn plain_serde_reads_null_as_missing() {
        #[derive(Deserialize)]
        struct Plain {
            #[serde(default)]
            v: Option<Option<u32>>,
        }
        let v = serde_json::from_str::<Plain>(r#"{"v":null}"#).unwrap().v;
        assert_eq!(v, None, "null was meant to clear, but reads as not sent");
    }

    #[test]
    fn missing_null_and_value_are_distinct() {
        let p = |s: &str| serde_json::from_str::<Patch>(s).unwrap().v;
        assert_eq!(p("{}"), None);
        assert_eq!(p(r#"{"v":null}"#), Some(None));
        assert_eq!(p(r#"{"v":7}"#), Some(Some(7)));
    }
}
