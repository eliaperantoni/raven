use crate::assets::*;
use std::path::Path;
use std::error::Error;
use std::ffi::OsStr;
use std::convert::TryFrom;

const IMPORT_DIR: &'static str = ".import";

struct ResPath {
    filename: String,
}

fn import(path: &Path) -> Result<(), Box<dyn Error>> {
    let ext = path.extension().ok_or(err!("no extension"))?;
    let ext = ext.to_str().ok_or(err!("non UTF8 path"))?;

    match ext {
        "png" | "jpg" | "jpeg" => todo!("import texture"),
        "obj" | "fbx" | "gltf" | "glb" => todo!("import scene"),
        _ => return Err(err!("unknown format {}", ext)),
    }

    todo!()
}


