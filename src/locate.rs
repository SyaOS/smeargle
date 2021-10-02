use async_std::io;
use async_std::path::{Path, PathBuf};
use tide::{prelude::*, StatusCode};

pub(crate) async fn locate(path: &Path) -> tide::Result<PathBuf> {
    match path.metadata().await {
        Ok(metadata) if metadata.is_file() => Ok(path.to_path_buf()),
        Ok(metadata) if metadata.is_dir() => {
            let path_buf = path.join("index.html");
            if path_buf.is_file().await {
                return Ok(path_buf);
            }
            let path_buf = path.join("index.hbs");
            if path_buf.is_file().await {
                return Ok(path_buf);
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
