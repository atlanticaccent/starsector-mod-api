use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Mod {
  mod_id: String,
  version: Version,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Version {
  String(String),
  Object(VersionObj),
}

#[derive(Serialize, Deserialize)]
pub struct VersionObj {
  #[serde(deserialize_with = "deserialize_number_from_string")]
  pub major: i32,
  #[serde(deserialize_with = "deserialize_number_from_string")]
  pub minor: i32,
  #[serde(default)]
  #[serde(deserialize_with = "deserialize_string_from_number")]
  pub patch: String,
}
