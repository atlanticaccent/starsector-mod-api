use serde_json::json;
use starsector_mod_info_shared::user::User;
use worker::{Request, RouteContext, Response};

pub async fn generate_user<D>(ctx: RouteContext<D>) -> worker::Result<Response> {
  let bucket = ctx.env.bucket("STARSECTOR_MOD_AUTH")?;

  let new_user = User::new();

  bucket.put(new_user.id.to_string(), serde_json::to_string(&new_user)?).execute().await?;

  Response::from_json(&json!({
    "id": new_user.id,
    "secret": new_user.password
  })).map(|res| res.with_status(200))
}