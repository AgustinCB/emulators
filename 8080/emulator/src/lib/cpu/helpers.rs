pub(crate) fn two_bytes_to_address(high_byte: u8, low_byte: u8) -> u16 {
    (high_byte as u16) << 8 | (low_byte as u16)
}

pub(crate) fn bit_count(n: u8) -> u8 {
    let mut count = 0;
    let mut acc = n;
    while acc > 0 {
        if (acc & 1) == 1{
            count +=1;
        }
        acc >>= 1;
    }
    return count;
}