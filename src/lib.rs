use axum::{Router, extract::Query, routing::get};
use serde::Deserialize;
use tower_service::Service;
use worker::*;

fn router() -> Router {
    Router::new().route("/convert", get(convert))
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router().call(req).await?)
}

#[derive(Deserialize)]
pub struct ConvertParams {
    url: String,
}

pub async fn convert(Query(params): Query<ConvertParams>) -> String {
    format!("Received URL: {}", params.url)
}
