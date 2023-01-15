use std::ops::Deref;

use rand::{
  distributions::{Alphanumeric, DistString},
  rngs::StdRng,
  SeedableRng,
};
use serde_json::json;
use uuid::Uuid;
use worker::{
  Env, Method, ObjectNamespace, Request, RequestInit, Response, RouteContext, State, Stub,
};

use crate::{
  assert_method,
  durable::{self, *},
  route_from_req,
  worker_result_ext::ResultExt,
  DOProvider, ParseBody, ScoreKey, STARSECTOR_MOD_AUTH,
};

use self::trusted::TrustedUser;

pub mod trusted;

const ZERO_KEY: &str = "ZERO_KEY";
const SCORE_KEY: &str = "SCORE_KEY";
const HIGH_SCORE_KEY: &str = "HIGH_SCORE_KEY";

#[derive(Clone, Debug, PartialEq, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum UserRoutes {
  Init,
  GetAndZero,
  Add,
  Password,
  #[strum(default)]
  Unknown(String),
}

impl Deref for UserRoutes {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.into()
  }
}

impl From<&UserRoutes> for Method {
  fn from(value: &UserRoutes) -> Self {
    match value {
      UserRoutes::Init => Method::Put,
      UserRoutes::GetAndZero => Method::Patch,
      UserRoutes::Add => Method::Patch,
      UserRoutes::Password => Method::Get,
      UserRoutes::Unknown(_) => Method::Get,
    }
  }
}

impl From<UserRoutes> for Method {
  fn from(value: UserRoutes) -> Self {
    value.into()
  }
}

#[durable_object]
pub struct DurableUser {
  state: State,
  env: Env,
}

#[durable_object]
impl DurableObject for DurableUser {
  fn new(state: State, env: Env) -> Self {
    Self { state, env }
  }

  async fn fetch(&mut self, req: Request) -> worker::Result<Response> {
    match route_from_req(&req)? {
      UserRoutes::Init => {
        self.set_score(0).await?;
        self.set_high_score(0).await?;

        let pass = Alphanumeric.sample_string(&mut StdRng::from_entropy(), 16);
        self.set_password(&pass).await?;

        let id = self.state.id().to_string();

        Response::ok(
          json!({
            "id": id,
            "pass": pass,
          })
          .to_string(),
        )
      }
      UserRoutes::Add => {
        assert_method!(req, UserRoutes::Add.into());

        let delta = req
          .url()?
          .query_pairs()
          .find_map(|(key, val)| {
            (key == "value")
              .then_some(&val)
              .and_then(|val| Some(val.parse::<u32>().map(ScoreKey::Score)))
              .or_else(|| (key == "zero_key").then_some(Ok(ScoreKey::ZeroKey(val.to_string()))))
          })
          .transpose()
          .conv()?;

        match delta {
          Some(ScoreKey::Score(score)) => {
            self.increment_score(score).await?;
            Response::empty()
          }
          Some(ScoreKey::ZeroKey(key)) => {
            if let Some(true) = self
              .get_zero_key()
              .await?
              .map(|current| current.to_string() == key)
            {
              let new = self.increment_score(1).await?;
              assert_eq!(new, 1);
              Response::empty()
            } else {
              Response::error("Zero-key invalid", 409)
            }
          }
          _ => Response::error("No values supplied in request", 400),
        }
      }
      UserRoutes::GetAndZero => {
        assert_method!(req, UserRoutes::GetAndZero.into());

        self
          .get_and_zero_score()
          .await
          .and_then(|res| Response::from_json(&res))
      }
      UserRoutes::Password => self.get_password().await.and_then(Response::ok),
      UserRoutes::Unknown(path) => Response::error(format!("Could not find path: {}", path), 404),
    }
  }
}

impl DurableUser {
  async fn get_and_zero_score(&mut self) -> worker::Result<ScoreKey> {
    let score = self.get_score().await?;

    Ok(if score == 0 {
      let zero_key = self.generate_zero_key().await?;
      ScoreKey::ZeroKey(zero_key)
    } else {
      self.set_score(0).await?;
      ScoreKey::Score(score)
    })
  }

  async fn get_zero_key(&self) -> worker::Result<Option<Uuid>> {
    self.state.storage().get(ZERO_KEY).await
  }

  async fn set_zero_key(&self, key: Uuid) -> worker::Result<()> {
    self.state.storage().put(ZERO_KEY, Some(key)).await
  }

  async fn generate_zero_key(&self) -> worker::Result<String> {
    let zero_key = Uuid::new_v4();
    self.set_zero_key(zero_key).await?;

    Ok(zero_key.to_string())
  }

  async fn invalidate_zero_key(&self) -> worker::Result<()> {
    self
      .state
      .storage()
      .put(ZERO_KEY, Option::<Uuid>::None)
      .await
  }

  async fn get_score(&self) -> worker::Result<u32> {
    self.state.storage().get(SCORE_KEY).await
  }

  async fn set_score(&self, score: u32) -> worker::Result<()> {
    self.invalidate_zero_key().await?;
    self.state.storage().put(SCORE_KEY, score).await
  }

  async fn increment_score(&self, value: u32) -> worker::Result<u32> {
    self.invalidate_zero_key().await?;

    let current = self.get_score().await?;
    let high_score = self.get_high_score().await?;

    let new = current + value;

    if new > high_score {
      TrustedUser::trusted(self)?.cmp_and_set(new).await?;
      self.set_high_score(new).await?;
    }
    self.set_score(new).await?;

    Ok(new)
  }

  async fn get_high_score(&self) -> worker::Result<u32> {
    self.state.storage().get(HIGH_SCORE_KEY).await
  }

  async fn set_high_score(&self, new_score: u32) -> worker::Result<()> {
    self.state.storage().put(HIGH_SCORE_KEY, new_score).await
  }

  async fn get_password(&self) -> worker::Result<String> {
    self.state.storage().get("password").await
  }

  async fn set_password(&self, password: &str) -> worker::Result<()> {
    self.state.storage().put("password", password).await
  }
}

pub struct User(Stub);

impl User {
  fn namespace<D>(ctx: &RouteContext<D>) -> worker::Result<ObjectNamespace> {
    ctx.env.durable_object(STARSECTOR_MOD_AUTH)
  }

  pub fn new<D>(ctx: RouteContext<D>) -> worker::Result<Self> {
    let namespace = User::namespace(&ctx)?;

    let id = namespace.unique_id()?;

    id.get_stub().map(Self)
  }

  pub fn from_hex<D>(ctx: &RouteContext<D>, hex: &str) -> worker::Result<Self> {
    let namespace = User::namespace(&ctx)?;

    let id = namespace.id_from_string(hex)?;

    id.get_stub().map(Self)
  }

  pub async fn init(&self) -> worker::Result<Response> {
    self.0.fetch_with_str(&UserRoutes::Init).await
  }

  pub async fn get_and_zero(&self) -> worker::Result<u32> {
    self
      .0
      .fetch_with_request(Request::new_with_init(
        &UserRoutes::GetAndZero,
        RequestInit::new().with_method(UserRoutes::GetAndZero.into()),
      )?)
      .await?
      .parse()
      .await
  }

  pub async fn get_password(&self) -> worker::Result<String> {
    self
      .0
      .fetch_with_str(&UserRoutes::Password)
      .await?
      .parse()
      .await
  }
}

impl DOProvider for DurableUser {
  fn get_env(&self) -> &Env {
    &self.env
  }
}

#[cfg(test)]
mod test {
  use super::UserRoutes;

  #[test]
  fn test_route_serialization() {
    let route = UserRoutes::GetAndZero;

    assert_eq!(&*route, "get_and_zero");

    assert_eq!(UserRoutes::try_from("get_and_zero"), Ok(route))
  }
}
