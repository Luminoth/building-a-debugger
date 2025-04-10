use nix::{errno::Errno, sys::ptrace, unistd::Pid};

#[derive(Debug, thiserror::Error)]
pub enum SdbError {
    #[error("ptrace error: {0}")]
    Ptrace(Errno),
}

pub type Result<T> = std::result::Result<T, SdbError>;

pub fn attach(pid: i32) -> Result<()> {
    match ptrace::attach(Pid::from_raw(pid)) {
        Ok(_) => Ok(()),
        Err(errno) => Err(SdbError::Ptrace(errno)),
    }
}
