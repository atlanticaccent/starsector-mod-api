use worker::{self, console_error, Response};

pub trait ResultExt<T, E> {
  fn conv(self) -> worker::Result<T>;
}

impl<T, E: ToString> ResultExt<T, E> for Result<T, E> {
  fn conv(self) -> worker::Result<T> {
    self.map_err(|e| worker::Error::RustError(e.to_string()))
  }
}

pub trait ResultResponseExt {
  fn or_500(self) -> worker::Result<Response>;
}

impl ResultResponseExt for Result<Response, worker::Error> {
  fn or_500(self) -> worker::Result<Response> {
    self.or_else(|err| {
      console_error!("Internal server error: {}", err.to_string());

      Response::error(format!("Internal server error: {}", err.to_string()), 500)
    })
  }
}
