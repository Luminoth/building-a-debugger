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

#[derive(Debug, Default)]
pub enum ProcessState {
    #[default]
    Stopped,
    Running,
    Exited,
    Terminated,
}

#[derive(Debug)]
pub struct Process {
    pid: Pid,
    terminate_on_drop: bool,
    state: ProcessState,
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.terminate_on_drop {
            // TODO:
        }
    }
}

impl Process {
    fn new(pid: Pid, terminate_on_drop: bool) -> Self {
        Self {
            pid,
            terminate_on_drop,
            state: ProcessState::default(),
        }
    }

    pub fn attach(pid: i32) -> Result<Self> {
        let this = Self::new(Pid::from_raw(pid), false);
        ptrace::attach(this.pid).map_err(SdbError::Ptrace)?;
        this.wait_on_signal()?;

        Ok(this)
    }

    pub fn launch(path: impl Into<String>) -> Result<Self> {
        let path = CString::new(path.into()).unwrap();
        let args: Vec<CString> = Vec::default();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                let this = Self::new(child, true);
                this.wait_on_signal()?;
                Ok(this)
            }
            Ok(ForkResult::Child) => {
                ptrace::traceme().map_err(SdbError::Ptrace)?;
                execvp(path.as_c_str(), &args).ok();
                unreachable!();
            }
            Err(errno) => Err(SdbError::Fork(errno)),
        }
    }

    pub fn wait_on_signal(&self) -> Result<()> {
        waitpid(self.pid, None)
            .map_err(SdbError::WaitPid)
            .map(|_| ())
    }

    pub fn resume(&mut self) -> Result<()> {
        ptrace::cont(self.pid, None).map_err(SdbError::Ptrace)?;
        self.state = ProcessState::Running;

        Ok(())
    }
}
