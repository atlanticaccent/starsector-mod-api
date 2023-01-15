use worker::{js_sys::encode_uri_component, Cache, Request, Response};

use crate::cache::Rate;

/// Checks client's IP against cache for number of requests over previous 60 seconds.
/// Exceeding the limit results in a 24 hour IP ban.
///
/// Takes a `worker::Request` and a `u32` limit indicating the maximum number of requests.
///
/// # Examples
///
/// ```
/// use starsector_mod_info_shared::rate_limit;
///
/// async fn route<D>(req: worker::Request, ctx: worker::RouteContext<D>) -> worker::Result<worker::Response> {
///   let limit = 25;
///   rate_limit!(&req, limit);
///
///   worker::Response::ok("OK")
/// }
/// ```
#[macro_export]
macro_rules! rate_limit {
  ($req:expr, $limit:expr, $ident:expr) => {
    if let Some(res) =
      starsector_mod_info_shared::middleware::rate_limit::rate_limit_internal($req, $limit, $ident)
        .await
        .transpose()
    {
      return res;
    };
  };
}

pub async fn rate_limit_internal(
  req: &Request,
  limit: u32,
  ident: impl AsRef<str>,
) -> worker::Result<Option<Response>> {
  if let Some(ip) = req.headers().get("CF-Connecting-IP")? {
    let key = format!(
      "https://{}{}.com",
      ident.as_ref(),
      encode_uri_component(&ip).to_string()
    );
    let cache = Cache::default();
    let rate = if let Some(mut cached_rate) = cache.get(&key, true).await? {
      let mut rate: Rate = match cached_rate.json().await {
        Ok(rate) => rate,
        err => {
          cache.delete(&key, true).await?; //delete me, potentially exploitable
          err?
        }
      };

      // low as each worker has it's own cache, so the real limit is {unknown num of workers * limit}
      if rate.count >= limit || rate.timeout {
        rate.timeout = true;

        let mut fake_response = Response::from_json(&rate)?;
        fake_response
          .headers_mut()
          .set("cache-control", "max-age=86400")?;

        return Response::error(
          "Too many requests. Please wait at least 24 hours before making another request.",
          429,
        )
        .map(Some);
      }

      rate.count += 1;

      rate
    } else {
      Rate {
        count: 1,
        timeout: false,
      }
    };

    let mut fake_response = Response::from_json(&rate)?;
    fake_response
      .headers_mut()
      .set("cache-control", "max-age=60")?;

    cache.put(&key, fake_response).await?;
  };

  Ok(None)
}
