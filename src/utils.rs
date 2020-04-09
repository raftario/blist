pub(crate) const PNG_MAGIC_NUMBER_LEN: usize = 8;
pub(crate) const PNG_MAGIC_NUMBER: &[u8; PNG_MAGIC_NUMBER_LEN] =
    &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[inline]
pub(crate) fn str_is_empty_or_has_newlines(s: &str) -> bool {
    s.is_empty() || s.chars().any(|c| c == '\n' || c == '\r')
}

#[inline]
pub(crate) fn str_is_hex(s: &str) -> bool {
    !s.chars().any(|c| !c.is_ascii_hexdigit())
}
