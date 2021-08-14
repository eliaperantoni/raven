use std::error::Error;
use std::fs;
use std::path::Path;

use ron;
use serde::{Deserialize, Serialize};

fn import(path: &Path) {
    match path.extension() {
        Some("png") => (),
        _ => panic!()
    }
}

#[derive(Serialize, Deserialize)]
struct Import {
    main: Path,
    rest: Vec<Path>,
}

impl Import {
    fn import(src: Path) {

    }


}
