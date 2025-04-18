#![allow(dead_code)]
#![allow(non_camel_case_types)]

use nix::libc;

macro_rules! gpr_offset {
    ($reg:ident) => {
        std::mem::offset_of!(libc::user, regs) + std::mem::offset_of!(libc::user_regs_struct, $reg)
    };
}

macro_rules! fpr_offset {
    ($reg:ident) => {
        std::mem::offset_of!(libc::user, i387)
            + std::mem::offset_of!(libc::user_fpregs_struct, $reg)
    };
}

macro_rules! dr_offset {
    ($number:literal) => {
        std::mem::offset_of!(libc::user, u_debugreg) + $number * 8
    };
}

// https://internals.rust-lang.org/t/official-way-to-get-the-size-of-a-field/22123
const fn size_of_return_value<F, T, U>(_f: &F) -> usize
where
    F: FnOnce(T) -> U,
{
    std::mem::size_of::<U>()
}

macro_rules! size_of_field {
    ($type:ty, $field:ident) => {
        size_of_return_value(&|s: $type| s.$field)
    };
}

macro_rules! fpr_size {
    ($reg:ident) => {
        size_of_field!(libc::user_fpregs_struct, $reg)
    };
}

// TODO: there's probably a way to turn this into a more X-macro style?
// where the enum and the array are automatically kept in sync

#[derive(Debug, PartialEq, Eq)]
pub enum RegisterId {
    // 64-bit GPRs
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

    // special ptrace value
    orig_rax,

    // 32-bit subregisters
    eax,
    edx,
    ecx,
    ebx,
    esi,
    edi,
    ebp,
    esp,
    r8d,
    r9d,
    r10d,
    r11d,
    r12d,
    r13d,
    r14d,
    r15d,

    // 16-bit subregisters
    ax,
    dx,
    cx,
    bx,
    si,
    di,
    bp,
    sp,
    r8w,
    r9w,
    r10w,
    r11w,
    r12w,
    r13w,
    r14w,
    r15w,

    // high 8-bit subregisters
    ah,
    dh,
    ch,
    bh,

    // low 8-bit subregisters
    al,
    dl,
    cl,
    bl,
    sil,
    dil,
    bpl,
    spl,
    r8b,
    r9b,
    r10b,
    r11b,
    r12b,
    r13b,
    r14b,
    r15b,

    // FPRs
    fcw,
    fsw,
    ftw,
    fop,
    frip,
    frdp,
    mxcsr,
    mxcsrmask,

    // ST registers
    st0,
    st1,
    st2,
    st3,
    st4,
    st5,
    st6,
    st7,

    // MM registers
    mm0,
    mm1,
    mm2,
    mm3,
    mm4,
    mm5,
    mm6,
    mm7,

    // XMM registers
    xmm0,
    xmm1,
    xmm2,
    xmm3,
    xmm4,
    xmm5,
    xmm6,
    xmm7,
    xmm8,
    xmm9,
    xmm10,
    xmm11,
    xmm12,
    xmm13,
    xmm14,
    xmm15,

    // Debug registers
    dr0,
    dr1,
    dr2,
    dr3,
    dr4,
    dr5,
    dr6,
    dr7,
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
    offset: usize,
    r#type: RegisterType,
    format: RegisterFormat,
}

macro_rules! define_gpr_64 {
    ($name:ident, $dwarf_id:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: $dwarf_id,
            size: 8,
            offset: gpr_offset!($name),
            r#type: RegisterType::Gpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_gpr_32 {
    ($name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: -1,
            size: 4,
            offset: gpr_offset!($super),
            r#type: RegisterType::SubGpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_gpr_16 {
    ($name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: -1,
            size: 2,
            offset: gpr_offset!($super),
            r#type: RegisterType::SubGpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_gpr_8h {
    ($name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: -1,
            size: 1,
            offset: gpr_offset!($super) + 1,
            r#type: RegisterType::SubGpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_gpr_8l {
    ($name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: -1,
            size: 1,
            offset: gpr_offset!($super),
            r#type: RegisterType::SubGpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_fpr {
    ($name:ident, $dwarf_id:literal, $user_name:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: $dwarf_id,
            size: fpr_size!($user_name),
            offset: fpr_offset!($user_name),
            r#type: RegisterType::Fpr,
            format: RegisterFormat::UInt,
        }
    };
}

macro_rules! define_fpr_st {
    ($name:ident, $number:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: 33 + $number,
            size: 16,
            offset: (fpr_offset!(st_space) + $number * 16),
            r#type: RegisterType::Fpr,
            format: RegisterFormat::LongDouble,
        }
    };
}

macro_rules! define_fpr_mm {
    ($name:ident, $number:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: 41 + $number,
            size: 8,
            offset: (fpr_offset!(st_space) + $number * 16),
            r#type: RegisterType::Fpr,
            format: RegisterFormat::Vector,
        }
    };
}

macro_rules! define_fpr_xmm {
    ($name:ident, $number:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: 17 + $number,
            size: 16,
            offset: (fpr_offset!(st_space) + $number * 16),
            r#type: RegisterType::Fpr,
            format: RegisterFormat::Vector,
        }
    };
}

macro_rules! define_dr {
    ($name:ident, $number:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: -1,
            size: 8,
            offset: dr_offset!($number),
            r#type: RegisterType::Dr,
            format: RegisterFormat::UInt,
        }
    };
}

const REGISTER_INFOS: &[RegisterInfo] = &[
    // 64-bit GPRs
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
    // special ptrace value
    define_gpr_64!(orig_rax, -1),
    // 32-bit subregisters
    define_gpr_32!(eax, rax),
    define_gpr_32!(edx, rdx),
    define_gpr_32!(ecx, rcx),
    define_gpr_32!(ebx, rbx),
    define_gpr_32!(esi, rsi),
    define_gpr_32!(edi, rdi),
    define_gpr_32!(ebp, rbp),
    define_gpr_32!(esp, rsp),
    define_gpr_32!(r8d, r8),
    define_gpr_32!(r9d, r9),
    define_gpr_32!(r10d, r10),
    define_gpr_32!(r11d, r11),
    define_gpr_32!(r12d, r12),
    define_gpr_32!(r13d, r13),
    define_gpr_32!(r14d, r14),
    define_gpr_32!(r15d, r15),
    // 16-bit subregisters
    define_gpr_16!(ax, rax),
    define_gpr_16!(dx, rdx),
    define_gpr_16!(cx, rcx),
    define_gpr_16!(bx, rbx),
    define_gpr_16!(si, rsi),
    define_gpr_16!(di, rdi),
    define_gpr_16!(bp, rbp),
    define_gpr_16!(sp, rsp),
    define_gpr_16!(r8w, r8),
    define_gpr_16!(r9w, r9),
    define_gpr_16!(r10w, r10),
    define_gpr_16!(r11w, r11),
    define_gpr_16!(r12w, r12),
    define_gpr_16!(r13w, r13),
    define_gpr_16!(r14w, r14),
    define_gpr_16!(r15w, r15),
    // high 8-bit subregisters
    define_gpr_8h!(ah, rax),
    define_gpr_8h!(dh, rdx),
    define_gpr_8h!(ch, rcx),
    define_gpr_8h!(bh, rbx),
    // low 8-bit subregisters
    define_gpr_8l!(al, rax),
    define_gpr_8l!(dl, rdx),
    define_gpr_8l!(cl, rcx),
    define_gpr_8l!(bl, rbx),
    define_gpr_8l!(sil, rsi),
    define_gpr_8l!(dil, rdi),
    define_gpr_8l!(bpl, rbp),
    define_gpr_8l!(spl, rsp),
    define_gpr_8l!(r8b, r8),
    define_gpr_8l!(r9b, r9),
    define_gpr_8l!(r10b, r10),
    define_gpr_8l!(r11b, r11),
    define_gpr_8l!(r12b, r12),
    define_gpr_8l!(r13b, r13),
    define_gpr_8l!(r14b, r14),
    define_gpr_8l!(r15b, r15),
    // FPRs
    define_fpr!(fcw, 65, cwd),
    define_fpr!(fsw, 66, swd),
    define_fpr!(ftw, -1, ftw),
    define_fpr!(fop, -1, fop),
    define_fpr!(frip, -1, rip),
    define_fpr!(frdp, -1, rdp),
    define_fpr!(mxcsr, 64, mxcsr),
    define_fpr!(mxcsrmask, -1, mxcr_mask),
    // ST registers
    define_fpr_st!(st0, 0),
    define_fpr_st!(st1, 1),
    define_fpr_st!(st2, 2),
    define_fpr_st!(st3, 3),
    define_fpr_st!(st4, 4),
    define_fpr_st!(st5, 5),
    define_fpr_st!(st6, 6),
    define_fpr_st!(st7, 7),
    // MM registers
    define_fpr_mm!(mm0, 0),
    define_fpr_mm!(mm1, 1),
    define_fpr_mm!(mm2, 2),
    define_fpr_mm!(mm3, 3),
    define_fpr_mm!(mm4, 4),
    define_fpr_mm!(mm5, 5),
    define_fpr_mm!(mm6, 6),
    define_fpr_mm!(mm7, 7),
    // XMM registers
    define_fpr_xmm!(xmm0, 0),
    define_fpr_xmm!(xmm1, 1),
    define_fpr_xmm!(xmm2, 2),
    define_fpr_xmm!(xmm3, 3),
    define_fpr_xmm!(xmm4, 4),
    define_fpr_xmm!(xmm5, 5),
    define_fpr_xmm!(xmm6, 6),
    define_fpr_xmm!(xmm7, 7),
    define_fpr_xmm!(xmm8, 8),
    define_fpr_xmm!(xmm9, 9),
    define_fpr_xmm!(xmm10, 10),
    define_fpr_xmm!(xmm11, 11),
    define_fpr_xmm!(xmm12, 12),
    define_fpr_xmm!(xmm13, 13),
    define_fpr_xmm!(xmm14, 14),
    define_fpr_xmm!(xmm15, 15),
    // Debug registers
    define_dr!(dr0, 0),
    define_dr!(dr1, 1),
    define_dr!(dr2, 2),
    define_dr!(dr3, 3),
    define_dr!(dr4, 4),
    define_dr!(dr5, 5),
    define_dr!(dr6, 6),
    define_dr!(dr7, 7),
];

pub fn register_info_by_id(id: RegisterId) -> &'static RegisterInfo {
    REGISTER_INFOS.iter().find(|&info| info.id == id).unwrap()
}

pub fn register_info_by_name(name: impl AsRef<str>) -> Option<&'static RegisterInfo> {
    REGISTER_INFOS
        .iter()
        .find(|&info| info.name == name.as_ref())
}

pub fn register_info_by_dwarf(dwarf_id: i32) -> Option<&'static RegisterInfo> {
    REGISTER_INFOS
        .iter()
        .find(|&info| info.dwarf_id == dwarf_id)
}
