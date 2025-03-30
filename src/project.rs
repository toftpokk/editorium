use std::path::PathBuf;

pub enum Node {
    File { name: String, path: PathBuf },
    Directory { name: String, path: PathBuf },
}

impl Node {
    pub fn new(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        return if path.is_dir() {
            Self::Directory {
                name: name,
                path: path,
            }
        } else {
            Self::File {
                name: name,
                path: path,
            }
        };
    }
}
