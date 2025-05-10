// TODO: can use T::from_ne_bytes() (per-type tho, not generic)
// and that's probably going to be better than this
// maybe even have like RegisterValue::from([u8; size]) or something?
/*pub unsafe fn from_bytes<T>(bytes: &[u8]) -> T
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
}*/

/*pub fn as_bytes<F>(from: &F) -> &[u8] {
    unsafe { ::core::slice::from_raw_parts((from as *const F) as *const u8, size_of::<F>()) }
}*/

pub fn as_bytes_mut<F>(from: &mut F) -> &mut [u8] {
    unsafe { ::core::slice::from_raw_parts_mut((from as *mut F) as *mut u8, size_of::<F>()) }
}
