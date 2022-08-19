use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct User {
  pub id: Uuid,
  pub password: Uuid,
  pub trusted: bool,
  reputation: u64,
}

impl User {
  pub fn new() -> Self {
    Self {
      id: Uuid::new_v4(),
      password: Uuid::new_v4(),
      trusted: false,
      reputation: 1,
    }
  }

  pub fn rep(&self) -> u64 {
    self.reputation
  }

  pub fn decrease_rep(&mut self, val: u64) {
    self.reputation = if val >= self.reputation {
      1
    } else {
      self.reputation - val
    };
  }

  #[must_use]
  pub fn increase_rep(&mut self, val: u64, limit: u64) -> Option<u64> {
    self.reputation += val;
    if self.reputation > limit {
      Some(self.reputation)
    } else {
      None
    }
  }
}
