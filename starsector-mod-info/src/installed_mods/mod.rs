use std::convert::TryInto;

use serde_json::Value;
use starsector_mod_info_shared::{amqp::HTTPAmqp, mod_info::Mod};
use worker::{wasm_bindgen::JsValue, Fetch, Headers, Request, RequestInit, Response, RouteContext};

const AMQP_USERNAME: &str = "rbetzayv";

pub async fn installed_mods<D>(mut req: Request, ctx: RouteContext<D>) -> worker::Result<Response> {
  if req
    .headers()
    .get("User-Agent")
    .ok()
    .flatten()
    .and_then(|agent| (!agent.is_empty()).then_some(()))
    .is_none()
  {
    return Response::error("Invalid User-Agent", 400);
  }

  let json: Vec<Mod> = match req.json().await {
    Ok(json) => json,
    Err(err) => {
      return match err {
        worker::Error::SerdeJsonError(_) => Response::error("Malformed request", 400),
        _ => Err(err),
      }
    }
  };

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

  let routed = Fetch::Request(amqp_request)
    .send()
    .await?
    .json::<Value>()
    .await?;

  if routed
    .get("routed")
    .and_then(|routed| routed.as_bool())
    .unwrap_or_default()
  {
    Response::ok("OK")
  } else {
    Response::error("Failed to write to RabbitMQ", 502)
  }
}
