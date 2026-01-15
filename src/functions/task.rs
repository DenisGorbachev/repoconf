use demand::Confirm;
use errgonomic::handle;
use std::io;
use thiserror::Error;

pub fn task(title: impl Into<String>) -> Result<bool, TaskError> {
    use TaskError::*;
    let title = title.into();
    let confirm = Confirm::new(&title);
    let confirmed = handle!(confirm.run(), ConfirmRunFailed, title);
    Ok(confirmed)
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("failed to confirm task '{title}'")]
    ConfirmRunFailed { source: io::Error, title: String },
}
