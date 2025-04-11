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

#[derive(Debug)]
pub enum ProcessState {
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
    pub fn attach(pid: i32) -> Result<Self> {
        let this = Self {
            pid: Pid::from_raw(pid),
            terminate_on_drop: false,
            state: ProcessState::Stopped,
        };

        ptrace::attach(this.pid).map_err(SdbError::Ptrace)?;
        this.wait_on_signal()?;

        Ok(this)
    }

    pub fn spawn_and_attach(path: impl Into<String>) -> Result<Self> {
        let path = CString::new(path.into()).unwrap();
        let args: Vec<CString> = Vec::default();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                let this = Self {
                    pid: child,
                    terminate_on_drop: true,
                    state: ProcessState::Stopped,
                };
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
        waitpid(self.pid, None).map_err(SdbError::WaitPid)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        ptrace::cont(self.pid, None).map_err(SdbError::Ptrace)?;
        // TODO: this is hanging for some reason
        //self.wait_on_signal()?;
        self.state = ProcessState::Running;

        Ok(())
    }
}
