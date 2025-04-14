#![cfg(test)]

use nix::{sys::signal, unistd::Pid};

pub fn process_exists(pid: Pid) -> nix::Result<()> {
    Ok(signal::kill(pid, None)?)
}
