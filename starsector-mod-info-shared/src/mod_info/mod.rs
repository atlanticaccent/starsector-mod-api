use std::{collections::HashMap, fmt::Display};

use chrono::{Utc, DateTime};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use worker::Object;

#[derive(Serialize, Deserialize, Debug)]
pub struct Mod {
  pub id: String,
  pub version: Version,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Version {
  String(String),
  Object(VersionObj),
}

impl Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    let output: String = match self {
      Version::String(s) => s.to_string(),
      Version::Object(o) => o.to_string(),
    };
    write!(f, "{}", output)
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionObj {
  #[serde(deserialize_with = "deserialize_number_from_string")]
  pub major: i32,
  #[serde(deserialize_with = "deserialize_number_from_string")]
  pub minor: i32,
  #[serde(default)]
  #[serde(deserialize_with = "deserialize_string_from_number")]
  pub patch: String,
}

impl Display for VersionObj {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    if !self.patch.is_empty() {
      write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    } else {
      write!(f, "{}.{}", self.major, self.minor)
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct VersionData {
  pub total: u32,
  pub canonical: bool,
  pub first_seen: DateTime<Utc>,
}

impl Default for VersionData {
  fn default() -> Self {
    Self {
      total: 1,
      canonical: false,
      first_seen: Utc::now()
    }
  }
}

pub async fn parse_map_from_body(body: Object) -> worker::Result<HashMap<String, VersionData>> {
  let stream = body
    .body()
    .ok_or(worker::Error::RustError(String::from("No body")))?
    .stream()?;
  let bytes: Vec<u8> = stream.try_collect::<Vec<Vec<u8>>>().await?.concat();

  serde_json::from_slice::<HashMap<String, VersionData>>(&bytes).map_err(|err| err.into())
}
