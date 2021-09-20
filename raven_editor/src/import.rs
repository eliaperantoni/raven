use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use image::GenericImageView;

use raven_core::io::Serializable;
use raven_core::path as path_pkg;
use raven_core::resource::Texture;

use crate::OpenProjectState;
use crate::Result;

const IMPORT_DIR: &'static str = ".import";

pub(super) fn import(path: &Path, state: &OpenProjectState) -> Result<()> {
    if !path_pkg::is_valid(path) {
        panic!("invalid path: {:?}", path);
    }

    let ext = match path.extension() {
        Some(os_ext) => os_ext.to_str(),
        _ => return Err(Box::<dyn Error>::from("no extension")),
    };

    match ext {
        Some("png" | "jpg" | "jpeg") => import_tex(path, state),
        Some("fbx" | "obj") => todo!(),
        _ => return Err(Box::<dyn Error>::from("unknown extension")),
    }?;

    Ok(())
}

/// Given the absolute path to an asset, returns the path to the root directory for the imported files.
///
/// For instance:
/// `$/ferris/ferris.fbx` becomes `$/.import/ferris/ferris.fbx`
fn as_import_root(path: &Path) -> PathBuf {
    assert!(path_pkg::is_valid(path));

    let mut import_root = PathBuf::default();
    import_root.push(path_pkg::PROJECT_ROOT_RUNE);
    import_root.push(IMPORT_DIR);
    import_root.push(path_pkg::strip_rune(path));

    import_root
}

fn wipe_dir(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            wipe_dir(&path)?;
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn prepare_import_root_for(path: &Path, state: &OpenProjectState) -> Result<PathBuf> {
    assert!(path_pkg::is_valid(path));

    let import_root = as_import_root(path);

    fs::create_dir_all(path_pkg::as_fs_abs(&state.root_path, &import_root))
        .map_err(|e| Box::<dyn Error>::from(e))?;

    // Make sure the import directory contains no file
    wipe_dir(&path_pkg::as_fs_abs(&state.root_path, &import_root))?;

    Ok(import_root)
}

fn import_tex(path: &Path, state: &OpenProjectState) -> Result<()> {
    let import_root = prepare_import_root_for(path, state)?;

    let tex = image::open(path_pkg::as_fs_abs(&state.root_path, path))?;

    let size = [tex.width(), tex.height()];

    let tex = tex.into_rgba8();

    let tex = Texture::new(tex.into_raw(), size);

    tex.save(path_pkg::as_fs_abs(
        &state.root_path,
        import_root.join("main.tex"),
    ))?;

    Ok(())
}
