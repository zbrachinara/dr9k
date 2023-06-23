use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
};

mod data_file {
    use std::{io::ErrorKind, path::PathBuf, sync::OnceLock};

    const DATA_DIR_PATH: &str = "./.data";
    static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

    pub fn get_data_dir() -> PathBuf {
        DATA_DIR
            .get_or_init(|| {
                if let Err(e) = std::fs::create_dir(DATA_DIR_PATH) {
                    if e.kind() != ErrorKind::AlreadyExists {
                        panic!("Could not create the data directory");
                    }
                }
                DATA_DIR_PATH.into()
            })
            .clone()
    }
}

pub use data_file::get_data_dir;
use tap::Tap;

pub fn data_file(path: impl AsRef<Path>) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(get_data_dir().tap_mut(|d| d.push(path)))
}
