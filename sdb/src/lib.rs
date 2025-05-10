mod bit;
mod pipe;
mod register_info;
mod registers;
mod test;
mod types;

use std::cell::RefCell;
use std::ffi::CString;
use std::os::fd::OwnedFd;

use nix::{
    errno::Errno,
    libc,
    sys::{ptrace, signal, wait},
    unistd::{self, Pid},
};
use num_traits::{FromPrimitive, ToPrimitive};
use tracing::trace;

use pipe::Pipe;
use register_info::{RegisterId, register_info_by_id};
use registers::{RegisterValue, Registers};

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

    #[error("register error: {0}")]
    Register(String),

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
    registers: RefCell<Registers>,
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
            registers: RefCell::new(Registers::new()),
        }
    }

    fn read_all_registers(&mut self) -> Result<()> {
        let regs = ptrace::getregs(self.pid).map_err(SdbError::Ptrace)?;
        self.registers.borrow_mut().get_data_mut().regs = regs;

        let regs =
            ptrace::getregset::<ptrace::regset::NT_PRFPREG>(self.pid).map_err(SdbError::Ptrace)?;
        self.registers.borrow_mut().get_data_mut().i387 = regs;

        for i in 0..8_usize {
            let id = RegisterId::dr0.to_usize().unwrap() + i;
            let info = register_info_by_id(RegisterId::from_usize(id).unwrap());

            let data = ptrace::read_user(self.pid, info.offset as ptrace::AddressType)
                .map_err(SdbError::Ptrace)?;
            self.registers.borrow_mut().get_data_mut().u_debugreg[i] = data as u64;
        }

        Ok(())
    }

    pub fn attach(pid: i32) -> Result<Self> {
        let mut this = Self::new(Pid::from_raw(pid), false, true);
        ptrace::attach(this.pid).map_err(SdbError::Ptrace)?;
        this.wait_on_signal()?;

        Ok(this)
    }

    pub fn launch(
        path: impl Into<String>,
        debug: bool,
        stdout_replacement: Option<OwnedFd>,
    ) -> Result<Self> {
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

                if let Some(stdout_replacement) = stdout_replacement {
                    if let Err(errno) = unistd::dup2_stdout(stdout_replacement) {
                        Self::exit_with_perror(&channel, "stdout replacement failed", errno);
                    }
                }

                if debug {
                    if let Err(errno) = ptrace::traceme() {
                        Self::exit_with_perror(&channel, "tracing failed", errno);
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

    /*#[inline]
    pub fn get_registers(&self) -> &Registers {
        &self.registers
    }

    #[inline]
    fn get_registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }*/

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

        if self.is_attached && self.state == ProcessState::Stopped {
            self.read_all_registers()?;
        }

        Ok(status)
    }

    pub fn resume(&mut self) -> Result<()> {
        ptrace::cont(self.pid, None).map_err(SdbError::Ptrace)?;
        self.state = ProcessState::Running;

        Ok(())
    }

    // TODO: this is lame hack to avoid self-referencing in Registers
    #[allow(clippy::missing_safety_doc)]
    pub fn write_register_by_id(&self, id: RegisterId, val: RegisterValue) -> Result<()> {
        self.registers.borrow_mut().write_by_id(id, val, self)
    }

    pub(crate) fn write_user_area(&self, offset: usize, data: u64) -> Result<()> {
        ptrace::write_user(
            self.pid,
            offset as ptrace::AddressType,
            data as libc::c_long,
        )
        .map_err(SdbError::Ptrace)
    }

    // have to write fprs all at once
    pub(crate) fn write_fprs(&self, fprs: libc::user_fpregs_struct) -> Result<()> {
        ptrace::setregset::<ptrace::regset::NT_PRFPREG>(self.pid, fprs).map_err(SdbError::Ptrace)
    }

    /*pub(crate) fn writegprs(&self, gprs: libc::user_regs_struct) -> Result<()> {
        ptrace::setregs(self.pid, gprs).map_err(SdbError::Ptrace)
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipe::Pipe;

    #[test]
    fn process_attach_success() {
        let target = Process::launch("yes", false, None).unwrap();

        let process = Process::attach(target.get_id().as_raw());
        assert!(process.is_ok());
        let process = process.unwrap();

        assert_eq!(process.get_status().unwrap(), 't');
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
        let process = Process::launch("yes", true, None);
        assert!(process.is_ok());
        let process = process.unwrap();

        assert_eq!(test::process_exists(process.get_id()), Ok(()));
    }

    #[test]
    fn process_launch_no_such_program() {
        let process = Process::launch("you_do_not_have_to_be_good", true, None);
        assert!(process.is_err());
        assert!(matches!(
            process,
            std::result::Result::Err(SdbError::Child(..))
        ));
    }

    #[test]
    fn process_resume_success() {
        {
            let mut process = Process::launch("yes", true, None).unwrap();

            let result = process.resume();
            assert!(result.is_ok());

            let status = process.get_status().unwrap();
            assert!(status == 'R' || status == 'S');
        }

        {
            let target = Process::launch("yes", false, None).unwrap();
            let mut process = Process::attach(target.get_id().as_raw()).unwrap();

            let result = process.resume();
            assert!(result.is_ok());

            let status = process.get_status().unwrap();
            assert!(status == 'R' || status == 'S');
        }
    }

    #[test]
    fn process_resume_already_exited() {
        let mut process = Process::launch("echo", true, None).unwrap();
        process.resume().unwrap();
        process.wait_on_signal().unwrap();

        let result = process.resume();
        assert!(matches!(
            result,
            std::result::Result::Err(SdbError::Ptrace(..))
        ));
    }

    #[test]
    fn write_register_works() {
        let mut channel = Pipe::new(false).unwrap();
        let mut process =
            Process::launch("test/targets/reg_write", true, channel.write.take()).unwrap();
        process.resume().unwrap();
        process.wait_on_signal().unwrap();

        // 0xcafecafe == 3405695742
        // [254, 202, 254, 202, 0, 0, 0, 0] as bytes
        let result = process.write_register_by_id(RegisterId::rsi, 0xcafecafe_u64.into());
        assert!(result.is_ok());

        process.resume().unwrap();
        process.wait_on_signal().unwrap();

        let output = String::from_utf8(channel.read().unwrap()).unwrap();
        assert_eq!(output, "0xcafecafe");
    }
}
