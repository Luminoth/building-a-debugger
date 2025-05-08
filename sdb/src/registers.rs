use std::mem::MaybeUninit;

use nix::libc;
use num_traits::ToPrimitive;

use crate::{
    Process, Result, SdbError,
    bit::*,
    register_info::*,
    types::{Byte64, Byte128},
};

#[derive(Debug)]
pub enum RegisterValue {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Double(f64),
    LongDouble(u128),
    Byte64(Byte64),
    Byte128(Byte128),
}

impl RegisterValue {
    fn get_size(&self) -> usize {
        match self {
            RegisterValue::UInt8(v) => size_of_val(v),
            RegisterValue::UInt16(v) => size_of_val(v),
            RegisterValue::UInt32(v) => size_of_val(v),
            RegisterValue::UInt64(v) => size_of_val(v),
            RegisterValue::Double(v) => size_of_val(v),
            RegisterValue::LongDouble(v) => size_of_val(v),
            RegisterValue::Byte64(v) => size_of_val(v),
            RegisterValue::Byte128(v) => size_of_val(v),
        }
    }

    fn widen(&self, info: &RegisterInfo) -> Byte128 {
        match info.format {
            RegisterFormat::UInt => match info.size {
                2 => match self {
                    RegisterValue::UInt8(v) => unsafe { to_byte128(v.to_i16()) },
                    RegisterValue::UInt16(v) => unsafe { to_byte128(v) },
                    _ => unreachable!(),
                },
                4 => match self {
                    RegisterValue::UInt8(v) => unsafe { to_byte128(v.to_i32()) },
                    RegisterValue::UInt16(v) => unsafe { to_byte128(v.to_i32()) },
                    RegisterValue::UInt32(v) => unsafe { to_byte128(v) },
                    _ => unreachable!(),
                },
                8 => match self {
                    RegisterValue::UInt8(v) => unsafe { to_byte128(v.to_i64()) },
                    RegisterValue::UInt16(v) => unsafe { to_byte128(v.to_i64()) },
                    RegisterValue::UInt32(v) => unsafe { to_byte128(v.to_i64()) },
                    RegisterValue::UInt64(v) => unsafe { to_byte128(v) },
                    RegisterValue::Double(v) => unsafe { to_byte128(v.to_i64()) },
                    RegisterValue::Byte64(v) => unsafe { to_byte128(v) },
                    _ => unreachable!(),
                },
                _ => match self {
                    RegisterValue::UInt8(v) => unsafe { to_byte128(v.to_i128()) },
                    RegisterValue::UInt16(v) => unsafe { to_byte128(v.to_i128()) },
                    RegisterValue::UInt32(v) => unsafe { to_byte128(v.to_i128()) },
                    RegisterValue::UInt64(v) => unsafe { to_byte128(v.to_i128()) },
                    RegisterValue::Double(v) => unsafe { to_byte128(v.to_i128()) },
                    RegisterValue::LongDouble(v) => unsafe { to_byte128(v.to_i128()) },
                    RegisterValue::Byte64(v) => unsafe { to_byte128(v) },
                    RegisterValue::Byte128(v) => unsafe { to_byte128(v) },
                },
            },
            RegisterFormat::DoubleFloat => match self {
                RegisterValue::UInt8(v) => unsafe { to_byte128(v.to_f64()) },
                RegisterValue::UInt16(v) => unsafe { to_byte128(v.to_f64()) },
                RegisterValue::UInt32(v) => unsafe { to_byte128(v.to_f64()) },
                RegisterValue::UInt64(v) => unsafe { to_byte128(v.to_f64()) },
                RegisterValue::Double(v) => unsafe { to_byte128(v) },
                RegisterValue::Byte64(v) => unsafe { to_byte128(v) },
                _ => unreachable!(),
            },
            RegisterFormat::LongDouble => {
                // widen to 64-bit float until https://github.com/rust-lang/rfcs/pull/3453 is implemented
                match self {
                    RegisterValue::UInt8(v) => unsafe { to_byte128(v.to_f64()) },
                    RegisterValue::UInt16(v) => unsafe { to_byte128(v.to_f64()) },
                    RegisterValue::UInt32(v) => unsafe { to_byte128(v.to_f64()) },
                    RegisterValue::UInt64(v) => unsafe { to_byte128(v.to_f64()) },
                    RegisterValue::Double(v) => unsafe { to_byte128(v) },
                    RegisterValue::LongDouble(v) => unsafe { to_byte128(v.to_f64()) },
                    RegisterValue::Byte64(v) => unsafe { to_byte128(v) },
                    RegisterValue::Byte128(v) => unsafe { to_byte128(v) },
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    data: libc::user,
}

impl Registers {
    pub(crate) fn new() -> Self {
        let data = MaybeUninit::<libc::user>::zeroed();
        Self {
            data: unsafe { data.assume_init() },
        }
    }

    pub(crate) fn get_data_mut(&mut self) -> &mut libc::user {
        &mut self.data
    }

    pub unsafe fn read(&self, info: &RegisterInfo) -> Result<RegisterValue> {
        let bytes = as_bytes(&self.data);
        let val = match info.format {
            RegisterFormat::UInt => match info.size {
                1 => RegisterValue::UInt8(unsafe { from_bytes::<u8>(&bytes[info.offset..]) }),
                2 => RegisterValue::UInt16(unsafe { from_bytes::<u16>(&bytes[info.offset..]) }),
                4 => RegisterValue::UInt32(unsafe { from_bytes::<u32>(&bytes[info.offset..]) }),
                8 => RegisterValue::UInt64(unsafe { from_bytes::<u64>(&bytes[info.offset..]) }),
                _ => return Err(SdbError::Register("Unexpected register size".to_owned())),
            },
            RegisterFormat::DoubleFloat => {
                RegisterValue::Double(unsafe { from_bytes::<f64>(&bytes[info.offset..]) })
            }
            RegisterFormat::LongDouble => {
                RegisterValue::LongDouble(unsafe { from_bytes::<u128>(&bytes[info.offset..]) })
            }
            RegisterFormat::Vector => {
                if info.size == 8 {
                    RegisterValue::Byte64(unsafe { from_bytes::<Byte64>(&bytes[info.offset..]) })
                } else {
                    RegisterValue::Byte128(unsafe { from_bytes::<Byte128>(&bytes[info.offset..]) })
                }
            }
        };

        Ok(val)
    }

    pub unsafe fn read_by_id_as(&self, id: RegisterId) -> Result<RegisterValue> {
        unsafe { self.read(register_info_by_id(id)) }
    }

    unsafe fn write(
        &self,
        info: &RegisterInfo,
        val: RegisterValue,
        process: &Process,
    ) -> Result<()> {
        if val.get_size() > info.size {
            return Err(SdbError::Other(
                "Registers::write called with mismatched register and value sizes".to_owned(),
            ));
        }

        let bytes = as_bytes(&self.data);
        let wide = val.widen(info);
        let val_bytes = as_bytes(&wide);

        // TODO: need to verify sizes here
        let mut bytes_ptr = &bytes[info.offset..];
        unsafe {
            libc::memcpy(
                (&mut bytes_ptr as *mut &[u8]) as *mut libc::c_void,
                (&val_bytes as *const &[u8]) as *const libc::c_void,
                info.size,
            );
        }

        let data = unsafe { from_bytes::<i64>(&bytes[info.offset..]) };
        if info.r#type == RegisterType::Fpr {
            // have to write fprs all at once
            process.write_fprs(self.data.i387)
        } else {
            let aligned_offset = info.offset & !0b111; // read / write require 8byte aligned address
            process.write_user_area(aligned_offset, data)
        }
    }

    pub(crate) unsafe fn write_by_id(
        &self,
        id: RegisterId,
        val: RegisterValue,
        process: &Process,
    ) -> Result<()> {
        let info = register_info_by_id(id);
        unsafe { self.write(info, val, process) }
    }
}
