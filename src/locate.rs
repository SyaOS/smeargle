use async_std::{
    io,
    path::{Component, Path, PathBuf},
};
use tide::{prelude::*, Request, StatusCode};

fn normalize(path: &Path) -> PathBuf {
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

pub(crate) async fn locate<T>(request: &Request<T>) -> tide::Result<PathBuf> {
    let mut buf = normalize(Path::new(request.url().path()));
    match buf.metadata().await {
        Ok(metadata) if metadata.is_file() => Ok(buf),
        Ok(metadata) if metadata.is_dir() => {
            buf.push("index.html");
            if matches!(buf.metadata().await, Ok(metadata) if metadata.is_file()) {
                return Ok(buf);
            }
            buf.set_file_name("index.hbs");
            if matches!(buf.metadata().await, Ok(metadata) if metadata.is_file()) {
                return Ok(buf);
            }
            Err(tide::Error::from_str(
                StatusCode::Forbidden,
                "Cannot load index file",
            ))
        }
        Ok(_metadata) => Err(tide::Error::from_str(
            StatusCode::NotFound,
            "Unsupported type",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Err(error).status(StatusCode::NotFound)
        }
        Err(error) => Err(error)?,
    }
}
