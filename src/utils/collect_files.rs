use std::{fs, io, path::PathBuf};

pub fn collect_files(dir: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let path = PathBuf::from(dir);

    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory '{}' not found", dir),
        ));
    }

    if path.is_file() {
        files.push(path);
        return Ok(files);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry.expect("The file entry does not exist");
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "gz" {
                    continue;
                }
            }
            files.push(path);
        }
    }

    Ok(files)
}
