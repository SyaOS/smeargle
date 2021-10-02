use async_std::path::{Component, Path, PathBuf};

pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::new();
    for component in path.components() {
        if let Component::Normal(name) = component {
            path_buf.push(name);
        } else if component == Component::ParentDir {
            path_buf.pop();
        }
    }
    path_buf
}
