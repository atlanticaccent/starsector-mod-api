use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Rate {
  pub count: u32,
  pub timeout: bool,
}
