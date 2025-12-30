pub mod converter;

use converter::convert_subscription;
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/convert", |req, _ctx| async move {
            let url = req.url()?;
            let params: std::collections::HashMap<String, String> = url
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            let target_url = match params.get("url") {
                Some(u) => u,
                None => {
                    return Response::error("Missing 'url' parameter", 400);
                }
            };

            let parsed_url: Url = match target_url.parse() {
                Ok(u) => u,
                Err(e) => {
                    return Response::error(format!("Invalid URL: {}", e), 400);
                }
            };

            let response = Fetch::Url(parsed_url).send().await;
            match response {
                Ok(mut resp) => {
                    // Extract headers before consuming body
                    let user_info = resp.headers().get("subscription-userinfo").ok().flatten();
                    let update_interval =
                        resp.headers().get("profile-update-interval").ok().flatten();
                    let web_page_url = resp.headers().get("profile-web-page-url").ok().flatten();

                    match resp.text().await {
                        Ok(content) => {
                            // Convert the subscription
                            match convert_subscription(&content) {
                                Ok(converted) => {
                                    let headers = Headers::new();
                                    headers.set("Content-Type", "text/yaml; charset=utf-8")?;
                                    headers.set(
                                        "Content-Disposition",
                                        "attachment; filename=clash.yaml",
                                    )?;

                                    // Forward subscription info headers
                                    if let Some(val) = user_info {
                                        headers.set("subscription-userinfo", &val)?;
                                    }
                                    if let Some(val) = update_interval {
                                        headers.set("profile-update-interval", &val)?;
                                    }
                                    if let Some(val) = web_page_url {
                                        headers.set("profile-web-page-url", &val)?;
                                    }

                                    Ok(Response::ok(converted)?.with_headers(headers))
                                }
                                Err(e) => Response::error(format!("Conversion failed: {}", e), 500),
                            }
                        }
                        Err(e) => Response::error(format!("Failed to read response: {}", e), 500),
                    }
                }
                Err(e) => Response::error(format!("Fetch failed: {}", e), 500),
            }
        })
        .run(req, env)
        .await
}
