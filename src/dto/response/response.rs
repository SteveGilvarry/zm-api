use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::AppResponseError;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub page_num: i64,
    pub page_size: i64,
    pub total: i64,
}

impl<T> PageResponse<T> {
    pub fn new(data: Vec<T>, page_num: i64, page_size: i64, total: i64) -> PageResponse<T> {
        PageResponse {
            data,
            page_num,
            page_size,
            total,
        }
    }

    pub fn map<F, B>(&self, f: F) -> PageResponse<B>
    where
        F: FnMut(&T) -> B,
    {
        let data: Vec<B> = self.data.iter().map(f).collect();
        PageResponse {
            data,
            page_num: self.page_num,
            page_size: self.page_size,
            total: self.total,
        }
    }
}

/// Generic paginated response wrapper for list endpoints
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// The items in the current page
    pub data: Vec<T>,
    /// Total number of items across all pages
    pub total: u64,
    /// Number of items per page
    pub per_page: u64,
    /// Current page number (1-indexed)
    pub current_page: u64,
    /// Total number of pages
    pub last_page: u64,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        let last_page = if total == 0 {
            1
        } else {
            total.div_ceil(page_size)
        };

        Self {
            data,
            total,
            per_page: page_size,
            current_page: page,
            last_page,
        }
    }

    /// Map the data items to a different type
    pub fn map_data<F, U>(self, f: F) -> PaginatedResponse<U>
    where
        F: FnMut(T) -> U,
    {
        PaginatedResponse {
            data: self.data.into_iter().map(f).collect(),
            total: self.total,
            per_page: self.per_page,
            current_page: self.current_page,
            last_page: self.last_page,
        }
    }

    /// Check if there's a next page
    #[allow(dead_code)]
    pub fn has_next_page(&self) -> bool {
        self.current_page < self.last_page
    }

    /// Check if there's a previous page
    #[allow(dead_code)]
    pub fn has_prev_page(&self) -> bool {
        self.current_page > 1
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum AppResultResponse<R> {
    Err(AppResponseError),
    Ok(R),
}

impl<R> AppResultResponse<R> {
    #[allow(dead_code)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, AppResultResponse::Ok(_))
    }
    #[allow(dead_code)]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
