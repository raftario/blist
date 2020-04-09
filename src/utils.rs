#[inline]
pub fn str_is_empty_or_has_newlines(s: &str) -> bool {
    s.is_empty() || s.chars().any(|c| c == '\n' || c == '\r')
}

#[inline]
pub fn str_is_hex(s: &str) -> bool {
    !s.chars().any(|c| !c.is_ascii_hexdigit())
}
