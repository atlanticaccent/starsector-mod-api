use std::ops::Deref;

use worker::{Env, Method, Request, Response, State, Stub};

use crate::{
  route_from_req, worker_result_ext::ResultExt, DOProvider, ParseBody, STARSECTOR_MOD_AUTH,
};

use super::durable::*;

const MAX_SCORE_KEY: &str = "max_score";
const CMP_SET_KEY: &str = "cmp_set";
const INIT_MAX: u32 = 100;

#[derive(Clone, Debug, PartialEq, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum TrustedRoutes {
  Get,
  CmpAndSet,
  Unknown(String),
}

impl Deref for TrustedRoutes {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.into()
  }
}

impl From<&TrustedRoutes> for Method {
  fn from(value: &TrustedRoutes) -> Self {
    match value {
      TrustedRoutes::Get => Method::Get,
      TrustedRoutes::CmpAndSet => Method::Patch,
      TrustedRoutes::Unknown(_) => Method::Get,
    }
  }
}

impl From<TrustedRoutes> for Method {
  fn from(value: TrustedRoutes) -> Self {
    value.into()
  }
}

#[durable_object]
pub struct DurableTrusted {
  state: State,
  _env: Env,
}

#[durable_object]
impl DurableObject for DurableTrusted {
  fn new(state: State, _env: Env) -> Self {
    Self { state, _env }
  }

  async fn fetch(&mut self, req: Request) -> worker::Result<Response> {
    let max: u32 = match self.state.storage().get(MAX_SCORE_KEY).await {
      Ok(max) => max,
      Err(worker::Error::JsError(val)) if val == "No such value in storage." => {
        self.state.storage().put(MAX_SCORE_KEY, INIT_MAX).await?;
        INIT_MAX
      }
      err => err?,
    };

    match route_from_req(&req)? {
      TrustedRoutes::Get => Response::ok(max.to_string()),
      TrustedRoutes::CmpAndSet => {
        let url = req.url()?;
        let Some(val) = url.query_pairs().find_map(|(key, val)| (key == CMP_SET_KEY).then_some(val)) else {
          return Response::error("No value in request", 400)
        };
        let other: u32 = val.parse().conv()?;

        if other >= max {
          self.state.storage().put(MAX_SCORE_KEY, other + 1).await?;
        }

        Response::ok("")
      }
      TrustedRoutes::Unknown(path) => {
        Response::error(format!("Could not find path: {}", path), 404)
      }
    }
  }
}

pub struct TrustedUser(pub Stub);

impl TrustedUser {
  pub const TRUSTED_ID: &str = "trusted";

  pub fn trusted(provider: &impl DOProvider) -> worker::Result<TrustedUser> {
    let namespace = provider.durable_namespace(STARSECTOR_MOD_AUTH)?;

    let id = namespace.id_from_name(TrustedUser::TRUSTED_ID)?;

    id.get_stub().map(TrustedUser)
  }

  pub async fn get_max(&self) -> worker::Result<u32> {
    self
      .0
      .fetch_with_str(&TrustedRoutes::Get)
      .await?
      .parse()
      .await
  }

  pub async fn cmp_and_set(&self, other: u32) -> worker::Result<()> {
    self
      .0
      .fetch_with_request(Request::new(
        &format!("{}?{}={}", &*TrustedRoutes::CmpAndSet, CMP_SET_KEY, other),
        Method::Patch,
      )?)
      .await?;

    Ok(())
  }
}
