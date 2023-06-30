use duplicate::duplicate_item;
use paste::paste;

use std::{io, path::PathBuf};

#[duplicate_item(
    name     folder_path;
    [data]   ["./.data"];
    [guild]  ["./.data/guilds"];
)]
paste! { pub fn [<name _dir>] () -> Result<PathBuf, io::Error> {
    const PATH_STR: &str = folder_path;
    let path = PathBuf::from(PATH_STR);
    if !path.is_dir() {
        std::fs::create_dir_all(&path)?;
    }
    Ok(path)
}}

