mod pipe;
mod test;

use std::ffi::CString;

use nix::{
    errno::Errno,
    sys::{ptrace, signal, wait},
    unistd::{self, Pid},
};
use tracing::trace;

use pipe::Pipe;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SdbError {
    #[error("child error: {0}")]
    Child(String),

    #[error("ptrace error: {0}")]
    Ptrace(Errno),

    #[error("fork error: {0}")]
    Fork(Errno),

    #[error("waitpid error: {0}")]
    WaitPid(Errno),

    #[error("pipe error: {0}")]
    Pipe(Errno),

    #[error("read error: {0}")]
    Read(Errno),

    #[error("write error: {0}")]
    Write(Errno),

    #[error("other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, SdbError>;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
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
                wait::waitpid(self.pid, None);
            }

            // detach and resume the process
            trace!("Detaching and resuming process ...");
            ptrace::detach(self.pid, None);
            signal::kill(self.pid, signal::SIGCONT);

            if self.terminate_on_drop {
                trace!("Terminating process ...");
                signal::kill(self.pid, signal::SIGKILL);
                wait::waitpid(self.pid, None);
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
        let mut this = Self::new(Pid::from_raw(pid), false);
        ptrace::attach(this.pid).map_err(SdbError::Ptrace)?;
        this.wait_on_signal()?;

        Ok(this)
    }

    pub fn launch(path: impl Into<String>) -> Result<Self> {
        let path = CString::new(path.into()).unwrap();
        let args: Vec<CString> = Vec::default();

        let mut channel = Pipe::new(true)?;

        match unsafe { unistd::fork() } {
            Ok(unistd::ForkResult::Parent { child }) => {
                channel.close_write();

                let data = channel.read()?;
                if !data.is_empty() {
                    // TODO: waitpid(child) here?
                    return Err(SdbError::Child(String::from_utf8(data).unwrap()));
                }

                let mut this = Self::new(child, true);
                this.wait_on_signal()?;
                Ok(this)
            }
            Ok(unistd::ForkResult::Child) => {
                channel.close_read();

                if let Err(errno) = ptrace::traceme() {
                    Self::exit_with_perror(&channel, "Tracing failed", errno);
                }

                let Err(errno) = unistd::execvp(path.as_c_str(), &args);
                Self::exit_with_perror(&channel, "exec failed", errno);

                unreachable!();
            }
            Err(errno) => Err(SdbError::Fork(errno)),
        }
    }

    fn exit_with_perror(channel: &Pipe, prefix: impl AsRef<str>, errno: Errno) {
        let message = format!("{}: {}", prefix.as_ref(), errno);
        let _ = channel.write(message);
        std::process::exit(-1);
    }

    #[inline]
    pub fn get_id(&self) -> Pid {
        self.pid
    }

    #[inline]
    pub fn get_state(&self) -> ProcessState {
        self.state
    }

    pub fn wait_on_signal(&mut self) -> Result<wait::WaitStatus> {
        let status = wait::waitpid(self.pid, None).map_err(SdbError::WaitPid)?;
        trace!("Wait status {:?}", status);
        match status {
            wait::WaitStatus::Exited(..) => self.state = ProcessState::Exited,
            wait::WaitStatus::Signaled(..) => self.state = ProcessState::Terminated,
            wait::WaitStatus::Stopped(..) => self.state = ProcessState::Stopped,
            _ => (),
        }
        Ok(status)
    }

    pub fn resume(&mut self) -> Result<()> {
        ptrace::cont(self.pid, None).map_err(SdbError::Ptrace)?;
        self.state = ProcessState::Running;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_launch_success() {
        let process = Process::launch("yes");
        assert!(process.is_ok());
        assert_eq!(test::process_exists(process.unwrap().get_id()), Ok(()));
    }

    #[test]
    fn process_launch_failure() {
        let process = Process::launch("you_do_not_have_to_be_good");
        assert!(process.is_err());
        assert!(matches!(
            process,
            std::result::Result::Err(SdbError::Child(..))
        ));
    }
}
