use raven_core::resource::*;
use std::path::{Path, PathBuf};
use std::error::Error;
use raven_core::io::Serializable;

const PROJECT_ROOT: &'static str = "/home/elia/code/raven_proj";
const IMPORT_DIR: &'static str = ".import";

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    import("cuddlyferris.png")?;
    Ok(())
}

fn import<T: AsRef<Path>>(p: T) -> Result<()> {
    let ext = match p.as_ref().extension() {
        Some(os_ext) => os_ext.to_str(),
        _ => return Err(Box::<dyn Error>::from("no extension")),
    };

    match ext {
        Some("png" | "jpg" | "jpeg") => import_tex(p.as_ref()),
        _ => return Err(Box::<dyn Error>::from("unknown extension")),
    }?;

    Ok(())
}

fn make_path<T: AsRef<Path>>(p: T) -> Result<PathBuf> {
    let mut abs_p = PathBuf::default();
    abs_p.push(PROJECT_ROOT);
    abs_p.push(IMPORT_DIR);
    abs_p.push(p.as_ref());

    std::fs::create_dir_all(&abs_p).map_err(|e| Box::<dyn Error>::from(e))?;
    Ok(abs_p)
}

fn as_abs<T: AsRef<Path>>(p: T) -> PathBuf {
    let mut abs_p = PathBuf::default();
    abs_p.push(PROJECT_ROOT);
    abs_p.push(p);
    abs_p
}

fn import_tex<T: AsRef<Path>>(p: T) -> Result<()> {
    let import_root = make_path(p.as_ref())?;

    let tex = image::open(as_abs(p.as_ref()))?;
    let tex = tex.flipv();
    let tex = tex.into_rgba8();

    let tex = Texture {
        raw: tex.into_raw()
    };

    let dst_path = import_root.join("main.tex");
    tex.save(dst_path)?;

    Ok(())
}
