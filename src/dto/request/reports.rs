use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// `Reports.Name` is `varchar(30)` in ZoneMinder's schema. Validating against
/// the column width turns an over-long name into a 400 naming the field,
/// instead of the 500 the truncation error used to surface as (GH #52).
///
/// Counted in `chars`, not garde's default of bytes: `varchar(30)` is 30
/// *characters*, so byte-counting would spuriously reject a legitimate
/// 30-character name containing any multi-byte character.
const NAME_MAX: usize = 30;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateReportRequest {
    #[garde(inner(length(chars, max = NAME_MAX)))]
    #[schema(example = "Weekly Security Report", max_length = 30)]
    pub name: Option<String>,
    #[garde(skip)]
    #[schema(example = 1)]
    pub filter_id: Option<u32>,
    #[garde(skip)]
    #[schema(value_type = String, example = "2025-01-01T00:00:00Z")]
    pub start_date_time: Option<String>,
    #[garde(skip)]
    #[schema(value_type = String, example = "2025-01-08T00:00:00Z")]
    pub end_date_time: Option<String>,
    /// Report interval in seconds (e.g. 604800 = 7 days).
    #[garde(skip)]
    #[schema(example = 604800)]
    pub interval: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateReportRequest {
    #[garde(inner(length(chars, max = NAME_MAX)))]
    #[schema(example = "Weekly Security Report", max_length = 30)]
    pub name: Option<String>,
    #[garde(skip)]
    #[schema(example = 1)]
    pub filter_id: Option<u32>,
    #[garde(skip)]
    #[schema(value_type = String, example = "2025-01-01T00:00:00Z")]
    pub start_date_time: Option<String>,
    #[garde(skip)]
    #[schema(value_type = String, example = "2025-01-08T00:00:00Z")]
    pub end_date_time: Option<String>,
    /// Report interval in seconds (e.g. 604800 = 7 days).
    #[garde(skip)]
    #[schema(example = 604800)]
    pub interval: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_with_name(name: &str) -> CreateReportRequest {
        CreateReportRequest {
            name: Some(name.to_string()),
            filter_id: None,
            start_date_time: None,
            end_date_time: None,
            interval: None,
        }
    }

    #[test]
    fn name_at_the_column_width_is_accepted() {
        // 30 characters is exactly what the column holds; it must not be
        // rejected. The reported boundary was 30 ok / 31 a 500.
        let req = create_with_name(&"a".repeat(NAME_MAX));
        assert!(req.validate().is_ok());
    }

    #[test]
    fn name_over_the_column_width_is_a_validation_error_naming_the_field() {
        let req = create_with_name(&"a".repeat(NAME_MAX + 1));
        let err = req.validate().expect_err("31 chars must not validate");
        assert!(
            err.to_string().contains("name"),
            "the error must name the offending field, got: {err}"
        );
    }

    #[test]
    fn the_bound_counts_characters_not_bytes() {
        // garde's default length mode is bytes, but varchar(30) is 30
        // characters. 30 multi-byte characters (90 bytes here) fit the column
        // and must be accepted; byte-counting would reject them.
        let multibyte = "é".repeat(NAME_MAX);
        assert_eq!(multibyte.chars().count(), NAME_MAX);
        assert!(multibyte.len() > NAME_MAX, "must actually be multi-byte");

        let req = create_with_name(&multibyte);
        assert!(
            req.validate().is_ok(),
            "30 multi-byte characters fit varchar(30) and must validate"
        );
    }

    #[test]
    fn an_absent_name_is_still_allowed() {
        // Reports.Name is nullable; validation must not make it required.
        let req = CreateReportRequest {
            name: None,
            filter_id: None,
            start_date_time: None,
            end_date_time: None,
            interval: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn update_enforces_the_same_bound() {
        let too_long = UpdateReportRequest {
            name: Some("a".repeat(NAME_MAX + 1)),
            filter_id: None,
            start_date_time: None,
            end_date_time: None,
            interval: None,
        };
        assert!(too_long.validate().is_err());
    }
}
