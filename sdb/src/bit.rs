use nix::libc;

use crate::types::Byte128;

pub unsafe fn from_bytes<T>(bytes: &[u8]) -> T
where
    T: Default,
{
    // TODO: bytes must be same or bigger than sizeof(T)
    let mut ret = T::default();
    unsafe {
        libc::memcpy(
            (&mut ret as *mut T) as *mut libc::c_void,
            bytes.as_ptr() as *const libc::c_void,
            size_of::<T>(),
        );
    }
    ret
}

pub fn as_bytes<F>(from: &F) -> &[u8] {
    unsafe { ::core::slice::from_raw_parts((from as *const F) as *const u8, size_of::<F>()) }
}

pub unsafe fn to_byte128<F>(src: F) -> Byte128 {
    // TODO: sizeof(F) must be 128 or less
    let mut ret = Byte128::default();
    unsafe {
        libc::memcpy(
            (&mut ret as *mut Byte128) as *mut libc::c_void,
            (&src as *const F) as *const libc::c_void,
            size_of::<F>(),
        );
    }
    ret
}

/*pub unsafe fn to_byte64<F>(src: F) -> Byte64 {
    // TODO: sizeof(F) must be 128 or less
    let mut ret = Byte64::default();
    unsafe {
        libc::memcpy(
            (&mut ret as *mut Byte64) as *mut libc::c_void,
            (&src as *const F) as *const libc::c_void,
            size_of::<F>(),
        );
    }
    ret
}*/
