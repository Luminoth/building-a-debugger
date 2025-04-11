use std::ffi::CString;

use nix::{
    errno::Errno,
    sys::{ptrace, signal, wait::waitpid},
    unistd::{ForkResult, Pid, execvp, fork},
};
use tracing::trace;

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

#[derive(Debug, Default, PartialEq, Eq)]
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
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        if self.pid.as_raw() != 0 {
            // have to stop the process before detaching
            trace!("Stopping process ...");
            if self.state == ProcessState::Running {
                signal::kill(self.pid, signal::SIGSTOP);
                waitpid(self.pid, None);
            }

            // detach and resume the process
            trace!("Detaching and resuming process ...");
            ptrace::detach(self.pid, None);
            signal::kill(self.pid, signal::SIGCONT);

            if self.terminate_on_drop {
                trace!("Terminating process ...");
                signal::kill(self.pid, signal::SIGKILL);
                waitpid(self.pid, None);
            }
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
