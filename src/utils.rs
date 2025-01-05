use std::path::Path;

pub(crate) fn validate_path(path: &Path) -> color_eyre::eyre::Result<()> {
    if !path.exists() {
        return Err(eyre::eyre!(format!("Path {:?} does not exist", path)));
    }
    if !path.is_dir() {
        return Err(eyre::eyre!(format!(
            "Path {:?} expected to be a directory, but was {}",
            path,
            if path.is_file() {
                "a file"
            } else {
                "something else"
            }
        )));
    }
    Ok(())
}
