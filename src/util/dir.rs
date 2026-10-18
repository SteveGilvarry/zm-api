use std::path::PathBuf;

pub fn get_project_root() -> std::io::Result<PathBuf> {
    if let Some(root) = get_cargo_project_root()? {
        Ok(root)
    } else {
        Ok(std::env::current_dir()?)
    }
}

pub fn get_cargo_project_root() -> std::io::Result<Option<PathBuf>> {
    let current_path = std::env::current_dir()?;

    for ancestor in current_path.ancestors() {
        for dir in std::fs::read_dir(ancestor)? {
            let dir = dir?;
            if dir.file_name() == *"Cargo.lock" {
                return Ok(Some(ancestor.to_path_buf()));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::get_cargo_project_root;

    #[test]
    fn test_get_cargo_project_root() {
        // cargo runs tests with the package root as the working directory, so
        // the nearest Cargo.lock is this crate's. Compare paths, not the
        // directory name: a worktree or a renamed clone is still the root.
        let root = get_cargo_project_root().unwrap().unwrap();
        let expected = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(
            root.canonicalize().unwrap(),
            expected.canonicalize().unwrap()
        );
    }
}
