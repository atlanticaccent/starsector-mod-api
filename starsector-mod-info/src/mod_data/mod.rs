use std::collections::{HashMap, HashSet};

use starsector_mod_info_shared::{mod_info::Metadata, ParseBody};
use worker::{Request, Response, RouteContext};

pub async fn req_mod_data_by_get<D>(
  req: Request,
  ctx: RouteContext<D>,
) -> worker::Result<Response> {
  let url = req.url()?;
  let mut params = url.query_pairs();

  if let Some((_, ids)) = params.find(|pair| pair.0 == "mods") {
    let ids: HashSet<&str> = ids.split("+").collect();

    if ids.len() > 0 {
      return mod_data(ids, ctx).await;
    };
  }

  Response::error("No query included in request", 400)
}

async fn mod_data<D>(ids: HashSet<&str>, ctx: RouteContext<D>) -> worker::Result<Response> {
  if ids.len() > 300 {
    return Response::error("Query too long", 400);
  }

  let bucket = ctx.env.bucket("STARSECTOR_MOD_METADATA")?;

  for id in ids {
    if let Some(body) = bucket.get(id).execute().await? {
      let dataset: HashMap<String, Metadata> = body.parse().await?;
    }
  }

  todo!()
}
