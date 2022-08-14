use std::collections::{HashSet, HashMap};

use starsector_mod_info_shared::mod_info::{parse_from_object, Metadata};
use worker::{Request, Response, RouteContext};

pub async fn req_mod_data_by_get<D>(
  req: Request,
  ctx: RouteContext<D>,
) -> worker::Result<Response> {
  let url = req.url()?;
  let mut params = url.query_pairs();

  if let Some((_, ids)) = params.find(|pair| pair.0 == "mods") {
    let ids: HashSet<&str> = ids.split("+").collect();

    let threshold = params
      .find_map(|(key, val)| (key == "threshold").then(|| val.parse::<f32>().ok()))
      .flatten()
      .unwrap_or_default();

    let bias_recent = params
      .find_map(|(key, val)| (key == "bias_recent").then(|| val.parse::<bool>().ok()))
      .flatten()
      .unwrap_or_default();

    if ids.len() > 0 {
      return mod_data(ids, threshold, bias_recent, ctx).await;
    };
  }

  return Response::error("No query included in request", 400);
}

async fn mod_data<D>(ids: HashSet<&str>, threshold: f32, bias_recent: bool, ctx: RouteContext<D>) -> worker::Result<Response> {
  if ids.len() > 300 {
    return Response::error("Query too long", 400);
  }

  let bucket = ctx.env.bucket("STARSECTOR_MOD_METADATA")?;

  for id in ids {
    if let Some(body) = bucket.get(id).execute().await? {
      let dataset: HashMap<String, Metadata> = parse_from_object(body).await?;

      let mut data = dataset.into_iter().collect::<Vec<(String, Metadata)>>();
      if bias_recent {
        data.sort_unstable_by_key(|k| k.1.first_seen);
      } else {
        data.sort_by_cached_key(|k| k.0.clone())
      }

      // let fraction = threshold.fract();
      // if fraction > 0.0 {
      //   data.iter().filter(||)
      // } else {
        
      // };
    }
  }

  todo!()
}
