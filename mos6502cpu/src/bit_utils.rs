#[inline]
pub(crate) fn two_complement(value: u8) -> u8 {
    (!u16::from(value) + 1) as u8
}

#[inline]
// TODO: This function is in three places already. Lets abstract it
pub(crate) fn two_bytes_to_word(high_byte: u8, low_byte: u8) -> u16 {
    u16::from(high_byte) << 8 | u16::from(low_byte)
}

#[inline]
pub(crate) fn word_to_two_bytes(word: u16) -> (u8, u8) {
    (word as u8, ((word & 0xff00) >> 8) as u8)
}
