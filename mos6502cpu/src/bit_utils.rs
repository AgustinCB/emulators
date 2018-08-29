#[inline]
pub(crate) fn two_complement(value: u8) -> u8 {
    !value + 1
}

#[inline]
pub(crate) fn two_bytes_to_word(high_byte: u8, low_byte: u8) -> u16 {
    (high_byte as u16) << 8 | (low_byte as u16)
}

#[inline]
pub(crate) fn word_to_two_bytes(word: u16) -> (u8, u8) {
    (word as u8, ((word & 0xff00) >> 8) as u8)
}