use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};

use crate::error::AppResult;

pub async fn store_file<P: AsRef<Path>>(file_path: &P, content: &[u8]) -> AppResult<()> {
  if let Some(parent_dir) = file_path.as_ref().parent() {
    fs::create_dir_all(&parent_dir).await?;
  }
  let mut file = fs::File::create(&file_path).await?;
  file.write_all(content).await?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::store_file;
  use test_context::{test_context, AsyncTestContext};
  use tokio::fs;
  use uuid::Uuid;
  
  use std::env;

  #[allow(dead_code)]
  struct FileTestContext {
    content: Vec<u8>,
    path: PathBuf,
  }

  impl AsyncTestContext for FileTestContext {
    async fn setup() -> Self {
      // Use simple in-memory content and a temp file path to avoid external dependencies
      let content = b"test-bytes".to_vec();
      let path = env::temp_dir().join(format!("zm_api_test_{}.bin", Uuid::new_v4()));
      Self { content, path }
    }

    async fn teardown(self) {
      // Best-effort cleanup
      let _ = fs::remove_file(self.path).await;
    }
  }

  #[test_context(FileTestContext)]
  #[tokio::test]
  pub async fn test_store_file(ctx: &mut FileTestContext) {
    store_file(&ctx.path, &ctx.content).await.unwrap();
    let result = fs::read(&ctx.path).await;
    assert!(result.is_ok())
  }
}
