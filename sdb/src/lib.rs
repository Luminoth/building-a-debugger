use std::ffi::CString;

use nix::{
    errno::Errno,
    sys::{ptrace, wait::waitpid},
    unistd::{ForkResult, Pid, execvp, fork},
};

#[derive(Debug, thiserror::Error)]
pub enum SdbError {
    #[error("ptrace error: {0}")]
    Ptrace(Errno),
    #[error("fork error: {0}")]
    Fork(Errno),
    #[error("waitpid error: {0}")]
    WaitPid(Errno),
}

pub type Result<T> = std::result::Result<T, SdbError>;

pub fn attach(pid: i32) -> Result<()> {
    let pid = Pid::from_raw(pid);
    ptrace::attach(pid).map_err(SdbError::Ptrace)?;
    waitpid(pid, None).map_err(SdbError::WaitPid)?;

    Ok(())
}

pub fn spawn_and_attach(path: impl Into<String>) -> Result<()> {
    let path = CString::new(path.into()).unwrap();
    let args: Vec<CString> = Vec::default();

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            waitpid(child, None).map_err(SdbError::WaitPid)?;
            Ok(())
        }
        Ok(ForkResult::Child) => {
            ptrace::traceme().map_err(SdbError::Ptrace)?;
            execvp(path.as_c_str(), &args).unwrap();
            unreachable!();
        }
        Err(errno) => Err(SdbError::Fork(errno)),
    }
}
