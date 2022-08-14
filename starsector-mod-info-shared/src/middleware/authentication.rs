use worker::{Request, Response, RouteContext};

use crate::{mod_info::parse_from_object, worker_result_ext::WorkerResultExt};

/// Checks for Authorization header and that the provided credentials are valid.
/// 
/// # Examples
/// 
/// ```
/// use starsector_mod_info_shared::authenticate;
/// 
/// async fn route<D>(req: worker::Request, ctx: worker::RouteContext<D>) -> worker::Result<worker::Response> {
///   authenticate!(&req, &ctx);
///
///   worker::Response::ok("OK")
/// }
/// ```
#[macro_export]
macro_rules! authenticate {
  ($req:expr, $ctx:expr) => {
    if let Some(res) =
      starsector_mod_info_shared::middleware::authentication::authenticate_internal($req, $ctx)
        .await
        .transpose()
    {
      return res;
    };
  };
}

pub async fn authenticate_internal<D>(
  req: &Request,
  ctx: &RouteContext<D>,
) -> worker::Result<Option<Response>> {
  let (user, pass) = if let Some(auth) = req
    .headers()
    .get("Authorization")
    .conv()?
    .map(parse_auth_header)
    .transpose()
    .conv()?
    .flatten()
  {
    auth
  } else {
    return Response::error("Authorization header malformed or missing", 400).map(Some);
  };

  let bucket = ctx.env.bucket("starsector-mod-auth")?;

  let object = if let Some(object) = bucket.get(user).execute().await? {
    object
  } else {
    return Response::error("Invalid username or password", 401).map(Some);
  };

  let stored: String = parse_from_object(object).await?;

  if pass == stored {
    Ok(None)
  } else {
    Response::error("Invalid username or password", 401).map(Some)
  }
}

fn parse_auth_header(auth: String) -> worker::Result<Option<(String, String)>> {
  let (_, value) = if let Some(split) = auth.split_once(' ') {
    split
  } else {
    return Ok(None);
  };

  let decoded = base64::decode(value).conv()?;

  if let Some((user, pass)) = String::from_utf8(decoded).conv()?.split_once(':') {
    Ok(Some((user.to_owned(), pass.to_owned())))
  } else {
    Ok(None)
  }
}
