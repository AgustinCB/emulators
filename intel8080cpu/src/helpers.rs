use intel8080cpu::Address;

pub(crate) fn two_bytes_to_word(high_byte: u8, low_byte: u8) -> u16 {
    (high_byte as u16) << 8 | (low_byte as u16)
}

pub(crate) fn word_to_address(word: u16) -> Address {
    [word as u8, ((word & 0xff00) >> 8) as u8]
}
