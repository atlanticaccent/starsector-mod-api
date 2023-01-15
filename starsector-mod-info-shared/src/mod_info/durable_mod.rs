use std::ops::Deref;

use serde::{Deserialize, Serialize};
use worker::{Env, Method, Request, Response, State};

use crate::{assert_method, durable::*, mod_info::Mod, route_from_req, ParseBody, ScoreKey};

use super::Metadata;

#[derive(Clone, Debug, PartialEq, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum ModRoutes {
  Init,
  Verified,
  Add,
  #[strum(default)]
  Unknown(String),
}

impl Deref for ModRoutes {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.into()
  }
}

impl From<&ModRoutes> for Method {
  fn from(value: &ModRoutes) -> Self {
    match value {
      ModRoutes::Init => Method::Put,
      ModRoutes::Verified => Method::Get,
      ModRoutes::Add => Method::Patch,
      ModRoutes::Unknown(_) => Method::Get,
    }
  }
}

impl From<ModRoutes> for Method {
  fn from(value: ModRoutes) -> Self {
    value.into()
  }
}

#[durable_object]
pub struct DurableMod {
  state: State,
  _env: Env,
}

#[durable_object]
impl DurableObject for DurableMod {
  fn new(state: State, _env: Env) -> Self {
    Self { state, _env }
  }

  async fn fetch(&mut self, req: Request) -> worker::Result<Response> {
    match route_from_req(&req)? {
      ModRoutes::Init => {
        assert_method!(req, Method::Put);

        let dom: Mod = req.parse().await?;
        self.put_mod(&dom).await?;

        self.put_meta(&Metadata::default()).await?;

        Response::ok("Successfully created")
      }
      ModRoutes::Verified => {
        todo!()
      }
      ModRoutes::Add => {
        assert_method!(req, Method::Patch);

        let contr: Contribution = req.parse().await?;

        self
          .edit_meta(|meta| {
            match contr {
              Contribution::Max => {
                todo!()
              }
              Contribution::User { user_id, value } => {
                meta.contributors.insert(user_id, value.clone())
              }
            };
          })
          .await?;

        todo!()
      }
      ModRoutes::Unknown(path) => Response::error(format!("Could not find path: {}", path), 404),
    }
  }
}

impl DurableMod {
  async fn get_mod(&self) -> worker::Result<Mod> {
    self.state.storage().get("mod").await
  }

  async fn put_mod(&self, dom: &Mod) -> worker::Result<()> {
    self.state.storage().put("mod", dom).await
  }

  async fn get_meta(&self) -> worker::Result<Metadata> {
    self.state.storage().get("meta").await
  }

  async fn edit_meta(&self, f: impl FnOnce(&mut Metadata)) -> worker::Result<()> {
    let mut meta = self.get_meta().await?;
    f(&mut meta);
    self.put_meta(&meta).await
  }

  async fn put_meta(&self, meta: &Metadata) -> worker::Result<()> {
    self.state.storage().put("meta", meta).await
  }
}

#[derive(Serialize, Deserialize)]
pub enum Contribution {
  Max,
  User { user_id: String, value: ScoreKey },
}
