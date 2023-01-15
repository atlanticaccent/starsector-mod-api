use std::str::FromStr;

use futures_util::TryStreamExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use worker::{async_trait::async_trait, ByteStream, Object, Response, Request, RouteContext, Env, ObjectNamespace};
use worker_result_ext::ResultExt;

pub mod amqp;
pub mod cache;
pub mod middleware;
pub mod mod_info;
pub mod user;
pub mod worker_result_ext;

const STARSECTOR_MOD_AUTH: &'static str = "STARSECTOR_MOD_AUTH";

#[derive(Clone, Serialize, Deserialize)]
pub enum ScoreKey {
  Score(u32),
  ZeroKey(String),
}

#[async_trait(?Send)]
pub trait ParseBody {
  async fn stream(&self) -> worker::Result<ByteStream>;

  async fn parse<T: DeserializeOwned>(&self) -> worker::Result<T> {
    let bytes: Vec<u8> = self
      .stream()
      .await?
      .try_collect::<Vec<Vec<u8>>>()
      .await?
      .concat();

    serde_json::from_slice::<T>(&bytes).map_err(|err| err.into())
  }
}

#[async_trait(?Send)]
impl ParseBody for Object {
  async fn stream(&self) -> worker::Result<ByteStream> {
    self
      .body()
      .ok_or(worker::Error::RustError(String::from("No body")))?
      .stream()
  }
}

#[async_trait(?Send)]
impl ParseBody for Response {
  async fn stream(&self) -> worker::Result<ByteStream> {
    self.stream().await
  }
}

#[async_trait(?Send)]
impl ParseBody for Request {
  async fn stream(&self) -> worker::Result<ByteStream> {
    self.stream().await
  }
}

#[macro_export]
macro_rules! assert_method {
  ($req:expr, $method:expr) => {
    if $req.method() != $method {
      return Response::error("Invalid method", 405);
    }
  };
}

mod durable {
  pub use worker::{
    async_trait, durable_object, js_sys, wasm_bindgen, wasm_bindgen_futures, worker_sys
  };
}

pub fn route_from_req<T: FromStr<Err = impl ToString>>(req: &Request) -> worker::Result<T> {
  req.path().trim_start_matches('/').parse::<T>().conv()
}

pub trait DOProvider {
  fn get_env(&self) -> &Env;

  fn durable_namespace(&self, namespace: &str) -> worker::Result<ObjectNamespace> {
    self.get_env().durable_object(namespace)
  }
}

impl<D> DOProvider for RouteContext<D> {
  fn get_env(&self) -> &Env {
    &self.env
  }
}
