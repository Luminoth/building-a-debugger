use std::mem::MaybeUninit;

use nix::libc;
use num_traits::ToPrimitive;

use crate::{
    Process, Result, SdbError,
    bit::*,
    register_info::*,
    types::{self, Byte64, Byte128},
};

#[derive(Debug)]
pub enum RegisterValue {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float(f32),
    Double(f64),
    LongDouble(f64), // 64-bit float until https://github.com/rust-lang/rfcs/pull/3453 is implemented
    Byte64(Byte64),
    Byte128(Byte128),
}

impl RegisterValue {
    #[inline]
    fn is_float(&self) -> bool {
        matches!(self, Self::Float(..) | Self::Double(..))
    }

    #[inline]
    fn is_signed(&self) -> bool {
        matches!(
            self,
            Self::Int8(..) | Self::Int16(..) | Self::Int32(..) | Self::Int64(..)
        )
    }

    #[inline]
    fn get_size(&self) -> usize {
        match self {
            Self::Int8(v) => size_of_val(v),
            Self::Int16(v) => size_of_val(v),
            Self::Int32(v) => size_of_val(v),
            Self::Int64(v) => size_of_val(v),
            Self::UInt8(v) => size_of_val(v),
            Self::UInt16(v) => size_of_val(v),
            Self::UInt32(v) => size_of_val(v),
            Self::UInt64(v) => size_of_val(v),
            Self::Float(v) => size_of_val(v),
            Self::Double(v) => size_of_val(v),
            Self::LongDouble(v) => size_of_val(v),
            Self::Byte64(v) => size_of_val(v),
            Self::Byte128(v) => size_of_val(v),
        }
    }

    fn widen(&self, info: &RegisterInfo) -> Byte128 {
        if self.is_float() {
            if info.format == RegisterFormat::DoubleFloat {
                return match self {
                    RegisterValue::Float(v) => types::byte128_8(v.to_f64().unwrap().to_ne_bytes()),
                    RegisterValue::Double(v) => types::byte128_8(v.to_ne_bytes()),
                    _ => unreachable!(),
                };
            }

            if info.format == RegisterFormat::LongDouble {
                // widen to 64-bit float until https://github.com/rust-lang/rfcs/pull/3453 is implemented
                return match self {
                    RegisterValue::Float(v) => types::byte128_8(v.to_f64().unwrap().to_ne_bytes()),
                    RegisterValue::Double(v) => types::byte128_8(v.to_f64().unwrap().to_ne_bytes()),
                    RegisterValue::LongDouble(v) => types::byte128_8(v.to_ne_bytes()),
                    _ => unreachable!(),
                };
            }
        } else if self.is_signed() && info.format == RegisterFormat::UInt {
            return match info.size {
                2 => match self {
                    RegisterValue::Int8(v) => types::byte128_2(v.to_i16().unwrap().to_ne_bytes()),
                    RegisterValue::Int16(v) => types::byte128_2(v.to_ne_bytes()),
                    _ => unreachable!(),
                },
                4 => match self {
                    RegisterValue::Int8(v) => types::byte128_4(v.to_i32().unwrap().to_ne_bytes()),
                    RegisterValue::Int16(v) => types::byte128_4(v.to_i32().unwrap().to_ne_bytes()),
                    RegisterValue::Int32(v) => types::byte128_4(v.to_ne_bytes()),
                    _ => unreachable!(),
                },
                8 => match self {
                    RegisterValue::Int8(v) => types::byte128_8(v.to_i64().unwrap().to_ne_bytes()),
                    RegisterValue::Int16(v) => types::byte128_8(v.to_i64().unwrap().to_ne_bytes()),
                    RegisterValue::Int32(v) => types::byte128_8(v.to_i64().unwrap().to_ne_bytes()),
                    RegisterValue::Int64(v) => types::byte128_8(v.to_ne_bytes()),
                    _ => unreachable!(),
                },
                _ => match self {
                    RegisterValue::Int8(v) => v.to_i128().unwrap().to_ne_bytes(),
                    RegisterValue::Int16(v) => v.to_i128().unwrap().to_ne_bytes(),
                    RegisterValue::Int32(v) => v.to_i128().unwrap().to_ne_bytes(),
                    RegisterValue::Int64(v) => v.to_i128().unwrap().to_ne_bytes(),
                    _ => unreachable!(),
                },
            };
        }

        match self {
            Self::UInt8(v) => v.to_u128().unwrap().to_ne_bytes(),
            Self::UInt16(v) => v.to_u128().unwrap().to_ne_bytes(),
            Self::UInt32(v) => v.to_u128().unwrap().to_ne_bytes(),
            Self::UInt64(v) => v.to_u128().unwrap().to_ne_bytes(),
            Self::Byte64(v) => types::byte128_64(v),
            Self::Byte128(v) => *v,
            _ => unreachable!(),
        }
    }
}

impl From<i8> for RegisterValue {
    fn from(value: i8) -> Self {
        Self::Int8(value)
    }
}

impl From<u8> for RegisterValue {
    fn from(value: u8) -> Self {
        Self::UInt8(value)
    }
}

impl From<i16> for RegisterValue {
    fn from(value: i16) -> Self {
        Self::Int16(value)
    }
}

impl From<u16> for RegisterValue {
    fn from(value: u16) -> Self {
        Self::UInt16(value)
    }
}

impl From<i32> for RegisterValue {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<u32> for RegisterValue {
    fn from(value: u32) -> Self {
        Self::UInt32(value)
    }
}

impl From<i64> for RegisterValue {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<u64> for RegisterValue {
    fn from(value: u64) -> Self {
        Self::UInt64(value)
    }
}

impl From<f32> for RegisterValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for RegisterValue {
    fn from(value: f64) -> Self {
        Self::Double(value)
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

    /*pub unsafe fn read(&self, info: &RegisterInfo) -> Result<RegisterValue> {
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
    }*/

    fn write(&mut self, info: &RegisterInfo, val: RegisterValue, process: &Process) -> Result<()> {
        if val.get_size() > info.size {
            return Err(SdbError::Other(
                "Registers::write called with mismatched register and value sizes".to_owned(),
            ));
        }

        let bytes = as_bytes_mut(&mut self.data);
        let wide = val.widen(info);
        for i in 0..info.size {
            bytes[info.offset + i] = wide[i];
        }

        if info.r#type == RegisterType::Fpr {
            // have to write fprs all at once
            process.write_fprs(self.data.i387)
        } else {
            // pokeuser requires 8-byte aligned offset
            // (lowest 3 bits set to 0)
            let aligned_offset = info.offset & !0b111; // read / write require 8byte aligned address

            let mut data: [u8; 8] = [0; 8];
            data[..info.size].copy_from_slice(&bytes[aligned_offset..(info.size + aligned_offset)]);
            let data = u64::from_ne_bytes(data);

            process.write_user_area(aligned_offset, data)
        }
    }

    pub(crate) fn write_by_id(
        &mut self,
        id: RegisterId,
        val: RegisterValue,
        process: &Process,
    ) -> Result<()> {
        let info = register_info_by_id(id);
        self.write(info, val, process)
    }
}
