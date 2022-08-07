use worker::{Request, Response, RouteContext};
use starsector_mod_info_shared::mod_info::Mod;

pub async fn installed_mods<D>(mut req: Request, ctx: RouteContext<D>) -> worker::Result<Response> {
  let _ = req
    .headers()
    .get("User-Agent")
    .ok()
    .flatten()
    .and_then(|agent| (!agent.is_empty()).then_some(()))
    .ok_or_else(|| Response::error("Invalid User-Agent", 400).unwrap_err())?;

  let json: Vec<Mod> = req.json().await?;

  todo!()
}