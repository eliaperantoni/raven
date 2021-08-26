use std::path::{Path, PathBuf};

pub const PROJECT_ROOT_RUNE: &'static str = "$/";

pub fn strip_rune<P: AsRef<Path> + ?Sized>(path: &P) -> &Path {
    path.as_ref()
        .strip_prefix(PROJECT_ROOT_RUNE)
        .expect("expected to find project root rune to strip it")
}

/// Given the absolute path to the project root and the absolute path to an asset, returns the filesystem absolute path.
///
/// For instance:
/// `$/ferris/ferris.fbx` becomes `/project/ferris/ferris.fbx` given that `project_root` is `/project`
pub fn as_fs_abs<R: AsRef<Path>, P: AsRef<Path>>(project_root: R, path: P) -> PathBuf {
    assert!(path.as_ref().starts_with(PROJECT_ROOT_RUNE));

    let mut abs_path = PathBuf::default();
    abs_path.push(project_root.as_ref());
    abs_path.push(strip_rune(path.as_ref()));

    abs_path
}
