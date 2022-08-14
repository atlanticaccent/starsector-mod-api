use worker;

pub(crate) trait WorkerResultExt<T, E> {
  fn conv(self) -> worker::Result<T>;
}

impl<T, E: ToString> WorkerResultExt<T, E> for Result<T, E> {
  fn conv(self) -> worker::Result<T> {
    self.map_err(|e| worker::Error::RustError(e.to_string()))
  }
}
