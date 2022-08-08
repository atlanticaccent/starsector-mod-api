use serde::Serialize;

#[derive(Serialize)]
pub struct HTTPAmqp {
  properties: Stub,
  routing_key: String,
  payload: String,
  payload_encoding: String,
}

impl HTTPAmqp {
  pub fn new(routing_key: &str, payload: impl Serialize) -> Result<Self, serde_json::Error> {
    let encoded = base64::encode(serde_json::to_string(&payload)?);

    Ok(HTTPAmqp {
      properties: Stub {},
      routing_key: routing_key.to_owned(),
      payload: encoded,
      payload_encoding: "base64".to_owned(),
    })
  }
}

impl TryFrom<HTTPAmqp> for String {
  type Error = serde_json::Error;

  fn try_from(value: HTTPAmqp) -> Result<Self, Self::Error> {
    serde_json::to_string(&value)
  }
}

#[derive(Serialize)]
struct Stub {}
