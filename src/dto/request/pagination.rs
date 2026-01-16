use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Default page size for paginated listings
pub const DEFAULT_PAGE_SIZE: u64 = 20;
/// Maximum allowed page size
pub const MAX_PAGE_SIZE: u64 = 1000;

/// Common pagination query parameters for list endpoints
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams, Default)]
pub struct PaginationParams {
    /// Page number (1-indexed). Defaults to 1.
    #[schema(example = 1, minimum = 1)]
    #[serde(default = "default_page")]
    pub page: Option<u64>,

    /// Number of items per page. Defaults to 20, max 1000.
    #[schema(example = 20, minimum = 1, maximum = 1000)]
    #[serde(default)]
    pub page_size: Option<u64>,
}

fn default_page() -> Option<u64> {
    Some(1)
}

impl PaginationParams {
    /// Get the page number, defaulting to 1
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    /// Get the page size, defaulting to DEFAULT_PAGE_SIZE and capped at MAX_PAGE_SIZE
    pub fn page_size(&self) -> u64 {
        self.page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE)
    }

    /// Get the offset for database queries (0-indexed)
    pub fn offset(&self) -> u64 {
        (self.page().saturating_sub(1)) * self.page_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_defaults() {
        let params = PaginationParams::default();
        assert_eq!(params.page(), 1);
        assert_eq!(params.page_size(), DEFAULT_PAGE_SIZE);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn test_pagination_custom_values() {
        let params = PaginationParams {
            page: Some(3),
            page_size: Some(50),
        };
        assert_eq!(params.page(), 3);
        assert_eq!(params.page_size(), 50);
        assert_eq!(params.offset(), 100); // (3-1) * 50
    }

    #[test]
    fn test_pagination_page_size_capped() {
        let params = PaginationParams {
            page: Some(1),
            page_size: Some(5000), // Over max
        };
        assert_eq!(params.page_size(), MAX_PAGE_SIZE);
    }

    #[test]
    fn test_pagination_minimum_values() {
        let params = PaginationParams {
            page: Some(0),      // Below minimum
            page_size: Some(0), // Below minimum
        };
        assert_eq!(params.page(), 1);
        assert_eq!(params.page_size(), 1);
    }
}
