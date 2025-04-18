mod pipe;
mod register_info;
mod registers;
mod test;

use std::ffi::CString;

use nix::{
    errno::Errno,
    sys::{ptrace, signal, wait},
    unistd::{self, Pid},
};
use tracing::trace;

use pipe::Pipe;

#[derive(Debug, thiserror::Error)]
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

    #[error("procfs error: {0}")]
    Procfs(#[from] procfs::ProcError),

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
    is_attached: bool,
    state: ProcessState,
}

impl Drop for Process {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        if self.pid.as_raw() != 0 {
            if self.is_attached {
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
            }

            if self.terminate_on_drop {
                trace!("Terminating process ...");
                signal::kill(self.pid, signal::SIGKILL);
                wait::waitpid(self.pid, None);
            }
        }
    }
}

impl Process {
    fn new(pid: Pid, terminate_on_drop: bool, is_attached: bool) -> Self {
        Self {
            pid,
            terminate_on_drop,
            is_attached,
            state: ProcessState::default(),
        }
    }

    pub fn attach(pid: i32) -> Result<Self> {
        let mut this = Self::new(Pid::from_raw(pid), false, true);
        ptrace::attach(this.pid).map_err(SdbError::Ptrace)?;
        this.wait_on_signal()?;

        Ok(this)
    }

    pub fn launch(path: impl Into<String>, debug: bool) -> Result<Self> {
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

                let mut this = Self::new(child, true, debug);
                if debug {
                    this.wait_on_signal()?;
                }
                Ok(this)
            }
            Ok(unistd::ForkResult::Child) => {
                channel.close_read();

                if debug {
                    if let Err(errno) = ptrace::traceme() {
                        Self::exit_with_perror(&channel, "Tracing failed", errno);
                    }
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

    #[inline]
    pub fn get_status(&self) -> Result<char> {
        let process = procfs::process::Process::new(self.pid.as_raw())?;
        Ok(process.stat()?.state)
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
    fn process_attach_success() {
        let target = Process::launch("yes", false).unwrap();

        let process = Process::attach(target.get_id().as_raw());
        assert!(process.is_ok());
        let process = process.unwrap();

        assert!(process.get_status().unwrap() == 't');
    }

    #[test]
    fn process_attach_invalid_pid() {
        let process = Process::attach(0);
        assert!(process.is_err());
        assert!(matches!(
            process,
            std::result::Result::Err(SdbError::Ptrace(..))
        ));
    }

    #[test]
    fn process_launch_success() {
        let process = Process::launch("yes", true);
        assert!(process.is_ok());
        let process = process.unwrap();

        assert_eq!(test::process_exists(process.get_id()), Ok(()));
    }

    #[test]
    fn process_launch_no_such_program() {
        let process = Process::launch("you_do_not_have_to_be_good", true);
        assert!(process.is_err());
        assert!(matches!(
            process,
            std::result::Result::Err(SdbError::Child(..))
        ));
    }

    #[test]
    fn process_resume_success() {
        {
            let mut process = Process::launch("yes", true).unwrap();

            let result = process.resume();
            assert!(result.is_ok());

            let status = process.get_status().unwrap();
            assert!(status == 'R' || status == 'S');
        }

        {
            let target = Process::launch("yes", false).unwrap();
            let mut process = Process::attach(target.get_id().as_raw()).unwrap();

            let result = process.resume();
            assert!(result.is_ok());

            let status = process.get_status().unwrap();
            assert!(status == 'R' || status == 'S');
        }
    }

    #[test]
    fn process_resume_already_exited() {
        let mut process = Process::launch("echo", true).unwrap();
        let _ = process.resume();
        let _ = process.wait_on_signal();

        let result = process.resume();
        assert!(matches!(
            result,
            std::result::Result::Err(SdbError::Ptrace(..))
        ));
    }
}
