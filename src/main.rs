mod locate;
mod normalize;
mod serve;
mod state;

use std::convert::TryFrom;
use std::env;

use async_std::path::PathBuf;

use crate::locate::locate;
use crate::normalize::normalize;
use crate::serve::serve;
use crate::state::State;

async fn endpoint(request: tide::Request<State>) -> tide::Result {
    let mut path_buf = PathBuf::from(request.url().path());
    path_buf = normalize(&path_buf);
    path_buf = locate(&path_buf).await?;
    serve(&request, &path_buf).await
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").map_or(Ok(0u16), |port| port.parse())?;
    let base_url = env::var("BASE_URL").map_or_else(
        |_| surf::Url::parse("https://httpbin.org/"),
        |url| url.parse(),
    )?;

    let handlebars = handlebars::Handlebars::new();
    let client = surf::Client::try_from(surf::Config::new().set_base_url(base_url))?
        .with(surf::middleware::Logger::new());

    tide::log::start();

    let mut app = tide::Server::with_state(State { handlebars, client });
    app.with(tide::log::LogMiddleware::new());
    app.at("").get(endpoint);
    app.at("*").get(endpoint);
    app.listen((host, port)).await?;

    Ok(())
}
