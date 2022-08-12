use installed_mods::installed_mods;
use starsector_mod_info_shared::rate_limit;
use worker::*;

mod installed_mods;
mod utils;

fn log_request(req: &Request) {
  console_log!(
    "{} - [{}], located at: {:?}, within: {}",
    Date::now().to_string(),
    req.path(),
    req.cf().coordinates().unwrap_or_default(),
    req.cf().region().unwrap_or("unknown region".into())
  );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
  log_request(&req);

  // Optionally, get more helpful error messages written to the console in the case of a panic.
  utils::set_panic_hook();

  // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
  // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
  // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
  let router = Router::new();

  // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
  // functionality and a `RouteContext` which you can use to  and get route parameters and
  // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
  router
    .post_async("/installed-mods", |req, ctx| async move {
      rate_limit!(&req, 10);
      installed_mods(req, ctx)
        .await
        .or_else(|err| {
          console_error!("Internal server error: {}", err.to_string());

          Response::error(format!("Internal server error: {}", err.to_string()), 500)
        })
    })
    .get("/worker-version", |_, ctx| {
      let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
      Response::ok(version)
    })
    .run(req, env)
    .await
}
