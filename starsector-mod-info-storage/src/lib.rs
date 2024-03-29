use persist::persist;
use starsector_mod_info_shared::worker_result_ext::ResultResponseExt;
use worker::*;

mod persist;
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

  let webhook_key = env.secret("WEBHOOK_KEY")?.to_string();

  // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
  // functionality and a `RouteContext` which you can use to  and get route parameters and
  // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
  router
    .post_async(
      &format!("/persist/{}", webhook_key),
      |req, ctx| async move { persist(req, ctx).await.or_500() },
    )
    .get("/worker-version", |_, ctx| {
      let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
      Response::ok(version)
    })
    .run(req, env)
    .await
}
