pub type Byte64 = [u8; 8];

pub type Byte128 = [u8; 16];

#[inline]
pub fn byte128_2(v: [u8; 2]) -> Byte128 {
    let mut ret = Byte128::default();
    ret[..2].copy_from_slice(&v);
    ret
}

#[inline]
pub fn byte128_4(v: [u8; 4]) -> Byte128 {
    let mut ret = Byte128::default();
    ret[..4].copy_from_slice(&v);
    ret
}

#[inline]
pub fn byte128_8(v: [u8; 8]) -> Byte128 {
    let mut ret = Byte128::default();
    ret[..8].copy_from_slice(&v);
    ret
}

#[inline]
pub fn byte128_64(v: &Byte64) -> Byte128 {
    let mut ret = Byte128::default();
    ret[..8].copy_from_slice(v);
    ret
}
