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
