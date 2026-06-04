use std::path::{Component, Path, PathBuf};

/// Returns `true` if any segment of the path is `..` (a parent-directory
/// component). Use as a defence-in-depth check before joining a
/// DB-supplied path onto a filesystem operation: even values written by
/// administrators shouldn't contain traversal, and rejecting them stops a
/// compromised admin from reading files outside the intended root.
///
/// Note that this does NOT defend against absolute paths embedded in the
/// middle of a join (e.g. `PathBuf::from("/safe").join("/etc")` resolves
/// to `/etc`). For that, validate the caller-controlled segment too.
pub fn contains_traversal<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .components()
        .any(|c| matches!(c, Component::ParentDir))
}

/// Normalize a path by collapsing `..` and `.` components AND stripping
/// any leading `/` so the result is always relative.
///
/// **This is the wrong primitive for security boundaries.** An absolute
/// input like `/etc/passwd` silently becomes the relative `etc/passwd`,
/// which then resolves under whatever the caller joins it onto. If you
/// need to reject absolute paths or traversal at a trust boundary, use
/// `contains_traversal` + an explicit absolute-path check on the
/// caller-supplied segment.
pub fn normalize<P: AsRef<Path>>(path: &P) -> PathBuf {
    let mut components = path.as_ref().components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::CurDir | Component::RootDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::util::path::normalize;

    use super::contains_traversal;

    #[test]
    fn traversal_detects_parent_dir_components() {
        assert!(contains_traversal("/foo/../bar"));
        assert!(contains_traversal("foo/../bar"));
        assert!(contains_traversal(".."));
        assert!(contains_traversal("a/b/.."));
    }

    #[test]
    fn traversal_passes_clean_paths() {
        assert!(!contains_traversal("/var/cache/zm/events"));
        assert!(!contains_traversal("foo/bar"));
        assert!(!contains_traversal(""));
        // CurDir (`.`) is not traversal — only `..` is.
        assert!(!contains_traversal("./foo"));
        // A literal directory named `..something` (a single component
        // starting with two dots but with more after) is a `Normal`
        // component, not `ParentDir`.
        assert!(!contains_traversal("foo/..bar"));
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize(&Path::new("./test")), Path::new("test"));
        assert_eq!(normalize(&Path::new(".//test")), Path::new("test"));
        assert_eq!(normalize(&Path::new("test")), Path::new("test"));
        assert_eq!(
            normalize(&Path::new("./a/b/c/../test.txt")),
            Path::new("a/b/test.txt")
        );
        assert_eq!(
            normalize(&Path::new("../a/b/c/test.txt")),
            Path::new("a/b/c/test.txt")
        );
        assert_eq!(normalize(&Path::new("/a/b/c")), Path::new("a/b/c"));
    }
}
