use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

use crate::ScoreKey;

mod durable_mod;

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
pub struct Metadata {
  pub total: u32,
  pub canonical: bool,
  pub first_seen: DateTime<Utc>,
  pub contributors: HashMap<String, ScoreKey>,
}

impl Default for Metadata {
  fn default() -> Self {
    Self {
      total: 1,
      canonical: false,
      first_seen: Utc::now(),
      contributors: HashMap::new(),
    }
  }
}
