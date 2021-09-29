use async_std::path::{Component, Path, PathBuf};
use std::{env, io};
use tide::{Body, Request, Response, StatusCode};

fn normalize(path: &str) -> PathBuf {
    let mut path_buf = PathBuf::new();
    for component in Path::new(path).components() {
        if let Component::Normal(name) = component {
            path_buf.push(name);
        } else if component == Component::ParentDir {
            path_buf.pop();
        }
    }
    return path_buf;
}

enum PathType<'a> {
    File(&'a Path),
    Directory(&'a Path),
    NotFound(&'a Path),
}

impl<'a> PathType<'a> {
    async fn from_path(path: &'a Path) -> io::Result<PathType<'a>> {
        match path.metadata().await {
            Ok(metadata) => {
                if metadata.is_file() {
                    Ok(PathType::File(path))
                } else if metadata.is_dir() {
                    Ok(PathType::Directory(path))
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "Unknown path type"))
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    Ok(PathType::NotFound(path))
                } else {
                    Err(e)
                }
            }
        }
    }
}

async fn endpoint(request: Request<()>) -> tide::Result {
    async fn serve(path: &Path) -> tide::Result {
        let body = Body::from_file(path).await?;
        return Ok(Response::from(body));
    }
    match PathType::from_path(normalize(request.url().path()).as_path()).await? {
        PathType::File(path) => serve(path).await,
        PathType::Directory(path) => {
            match PathType::from_path(path.join("index.html").as_path()).await? {
                PathType::File(path) => serve(path).await,
                PathType::NotFound(_) => {
                    match PathType::from_path(path.join("index.hbs").as_path()).await? {
                        PathType::File(path) => serve(path).await,
                        _ => Ok(Response::new(StatusCode::NotFound)),
                    }
                }
                _ => Ok(Response::new(StatusCode::NotFound)),
            }
        }
        _ => Ok(Response::new(StatusCode::NotFound)),
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    tide::log::start();

    let mut app = tide::Server::new();
    app.with(tide::log::LogMiddleware::new());

    app.at("").get(endpoint);
    app.at("*").get(endpoint);

    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").map_or(0u16, |port| port.parse().unwrap());

    app.listen((host, port)).await
}
