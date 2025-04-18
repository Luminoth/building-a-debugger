#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(non_camel_case_types)]

#[derive(Debug)]
pub enum RegisterId {
    rax,
    rdx,
    rcx,
    rbx,
    rsi,
    rdi,
    rbp,
    rsp,
    r8,
    r9,
    r10,
    r11,
    r12,
    r13,
    r14,
    r15,
    rip,
    eflags,
    cs,
    fs,
    gs,
    ss,
    ds,
    es,
    orig_rax,
}

#[derive(Debug)]
pub enum RegisterType {
    Gpr,
    SubGpr,
    Fpr,
    Dr,
}

#[derive(Debug)]
pub enum RegisterFormat {
    UInt,
    DoubleFloat,
    LongDouble,
    Vector,
}

#[derive(Debug)]
pub struct RegisterInfo {
    id: RegisterId,
    name: &'static str,
    dwarf_id: i32,
    size: usize,
    //offset: usize,
    r#type: RegisterType,
    format: RegisterFormat,
}

// TODO: https://doc.rust-lang.org/nightly/core/mem/macro.offset_of.html

macro_rules! define_gpr_64 {
    ($name:ident, $dwarf_id:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: "$name",
            dwarf_id: $dwarf_id,
            size: 8,
            //offset: 0,
            r#type: RegisterType::Gpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_gpr_32 {
    ($name:ident, $dwarf_id:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: "$name",
            dwarf_id: -1,
            size: 4,
            //offset: 0,
            r#type: RegisterType::SubGpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_gpr_16 {
    ($name:ident, $dwarf_id:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: "$name",
            dwarf_id: -1,
            size: 2,
            //offset: 0,
            r#type: RegisterType::SubGpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_gpr_8h {
    ($name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: "$name",
            dwarf_id: -1,
            size: 1,
            //offset: 0, + 1
            r#type: RegisterType::SubGpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_gpr_8l {
    ($name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: "$name",
            dwarf_id: -1,
            size: 1,
            //offset: 0,
            r#type: RegisterType::SubGpr,
            format: RegisterFormat::UInt,
        }
    };
}

pub const REGISTER_INFOS: &[RegisterInfo] = &[
    define_gpr_64!(rax, 0),
    define_gpr_64!(rdx, 1),
    define_gpr_64!(rcx, 2),
    define_gpr_64!(rbx, 3),
    define_gpr_64!(rsi, 4),
    define_gpr_64!(rdi, 5),
    define_gpr_64!(rbp, 6),
    define_gpr_64!(rsp, 7),
    define_gpr_64!(r8, 8),
    define_gpr_64!(r9, 9),
    define_gpr_64!(r10, 10),
    define_gpr_64!(r11, 11),
    define_gpr_64!(r12, 12),
    define_gpr_64!(r13, 13),
    define_gpr_64!(r14, 14),
    define_gpr_64!(r15, 15),
    define_gpr_64!(rip, 16),
    define_gpr_64!(eflags, 49),
    define_gpr_64!(cs, 51),
    define_gpr_64!(fs, 54),
    define_gpr_64!(gs, 55),
    define_gpr_64!(ss, 52),
    define_gpr_64!(ds, 53),
    define_gpr_64!(es, 50),
    define_gpr_64!(orig_rax, -1),
];
