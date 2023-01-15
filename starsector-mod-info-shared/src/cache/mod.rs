use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Rate {
  pub count: u32,
  pub timeout: bool,
}
