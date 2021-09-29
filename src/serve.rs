use std::ffi::OsStr;

use async_std::{fs, path::PathBuf};
use tide::{http::mime, Body, Request, Response, StatusCode};

use crate::State;

pub(crate) async fn serve(request: &Request<State>, path: PathBuf) -> tide::Result {
    if path.extension().and_then(OsStr::to_str) == Some("hbs") {
        let State { handlebars, client } = request.state();

        let template_string = fs::read_to_string(path).await?;

        let url = request.url();
        let mut client_url = url.path().trim_end_matches(".hbs").to_string();
        if let Some(query) = url.query() {
            client_url.push('?');
            client_url.push_str(query);
        }

        let data = dbg!(
            dbg!(dbg!(client).get(client_url))
                .recv_json::<serde_json::Value>()
                .await
        )
        .map_err(|error| tide::Error::new(StatusCode::BadGateway, error.into_inner()))?;

        let body_string = handlebars.render_template(&template_string, &data)?;
        Ok(Response::builder(StatusCode::Ok)
            .content_type(mime::HTML)
            .body(body_string)
            .build())
    } else {
        Ok(Body::from_file(path).await?.into())
    }
}
