use std::convert::TryInto;

use starsector_mod_info_shared::{amqp::HTTPAmqp, mod_info::Mod};
use worker::{wasm_bindgen::JsValue, Fetch, Headers, Request, RequestInit, Response, RouteContext};

const AMQP_USERNAME: &str = "rbetzayv";

pub async fn installed_mods<D>(mut req: Request, ctx: RouteContext<D>) -> worker::Result<Response> {
  let _ = req
    .headers()
    .get("User-Agent")
    .ok()
    .flatten()
    .and_then(|agent| (!agent.is_empty()).then_some(()))
    .ok_or_else(|| Response::error("Invalid User-Agent", 400).unwrap_err())?;

  let json: Vec<Mod> = req.json().await?;

  let http_amqp: String = HTTPAmqp::new("write", json)?.try_into()?;

  let key = ctx.secret("AMQP_KEY")?.to_string();

  let mut headers = Headers::new();
  headers.append("Content-Type", "application/json")?;
  let credentials = base64::encode(format!("{}:{}", AMQP_USERNAME, key));
  let base64 = format!("Basic {}", credentials);
  headers.append("Authorization", &base64)?;

  let amqp_request = Request::new_with_init(
    &format!(
      "https://moose.rmq.cloudamqp.com/api/exchanges/{}/amq.default/publish",
      AMQP_USERNAME
    ),
    RequestInit::new()
      .with_method(worker::Method::Post)
      .with_headers(headers)
      .with_body(Some(JsValue::from_str(&http_amqp))),
  )?;

  Fetch::Request(amqp_request).send().await
}
