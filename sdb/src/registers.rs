#![allow(dead_code)]

use nix::libc;

use crate::{Process, register_info::*};

#[derive(Debug)]
pub enum RegisterValue {
    Todo,
}

#[derive(Debug)]
pub struct Register<'a> {
    process: &'a Process,
    data: Option<libc::user>,
}

impl<'a> Register<'a> {
    pub(crate) fn new(process: &'a Process) -> Self {
        Self {
            process,
            data: None,
        }
    }

    pub fn read(&self, _info: &RegisterInfo) -> RegisterValue {
        RegisterValue::Todo
    }

    pub fn read_by_id_as(&self, id: RegisterId) -> RegisterValue {
        self.read(register_info_by_id(id))
    }

    pub fn write(&self, _info: &RegisterInfo, _val: RegisterValue) {}

    pub fn write_by_id(&self, id: RegisterId, val: RegisterValue) {
        self.write(register_info_by_id(id), val)
    }
}
