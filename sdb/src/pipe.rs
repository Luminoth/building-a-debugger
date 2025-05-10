use std::os::fd::{AsFd, OwnedFd};

use nix::{fcntl, unistd};

use crate::{Result, SdbError};

#[derive(Debug)]
pub struct Pipe {
    read: Option<OwnedFd>,
    pub(crate) write: Option<OwnedFd>,
}

impl Pipe {
    pub fn new(close_on_exec: bool) -> Result<Self> {
        let (read, write) = unistd::pipe2(if close_on_exec {
            fcntl::OFlag::O_CLOEXEC
        } else {
            fcntl::OFlag::empty()
        })
        .map_err(SdbError::Pipe)?;
        Ok(Self {
            read: Some(read),
            write: Some(write),
        })
    }

    pub fn read(&self) -> Result<Vec<u8>> {
        if let Some(read) = &self.read {
            let mut buf = [0; 1024];
            let read = unistd::read(read, &mut buf).map_err(SdbError::Read)?;
            Ok(buf[0..read].to_vec())
        } else {
            Err(SdbError::Other("Invalid Pipe Read".to_owned()))
        }
    }

    pub fn close_read(&mut self) {
        self.read = None;
    }

    pub fn write(&self, from: impl AsRef<[u8]>) -> Result<usize> {
        if let Some(write) = &self.write {
            unistd::write(write.as_fd(), from.as_ref()).map_err(SdbError::Write)
        } else {
            Err(SdbError::Other("Invalid Pipe Write".to_owned()))
        }
    }

    pub fn close_write(&mut self) {
        self.write = None;
    }
}
