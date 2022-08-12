use std::collections::HashMap;

use starsector_mod_info_shared::mod_info::{Mod, VersionData, parse_map_from_body};
use worker::{Request, Response, RouteContext};

pub async fn persist<D>(mut req: Request, ctx: RouteContext<D>) -> worker::Result<Response> {
  let mods = req.json::<Vec<Mod>>().await?;

  let bucket = ctx.env.bucket("STARSECTOR_MOD_METADATA")?;

  for mod_info in mods {
    let mut map = if let Some(body) = bucket.get(mod_info.id.as_str())
      .execute()
      .await?
    {
      parse_map_from_body(body).await?
    } else {
      HashMap::new()
    };

    let version = mod_info.version.to_string();
    if let Some(val) = map.get_mut(&version) {
      val.total += 1;
    } else {
      map.insert(version, VersionData::default());
    }

    let stringified_map = serde_json::to_string(&map)?;

    bucket.put(mod_info.id.as_str(), stringified_map)
      .execute()
      .await?;
  }

  Response::ok("OK")
}
