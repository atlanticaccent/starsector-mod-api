use serde::Deserialize;

#[derive(Deserialize)]
pub struct Mod {
  mod_id: String,
  version: VersionUnion,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum VersionUnion {
  String(String),
  Object(Version),
}

#[derive(Deserialize)]
pub struct Version {

}
