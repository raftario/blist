use std::path::Path;

pub(crate) const PNG_MAGIC_NUMBER_LEN: usize = 8;
pub(crate) const PNG_MAGIC_NUMBER: &[u8; PNG_MAGIC_NUMBER_LEN] =
    &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

pub(crate) const JPG_MAGIC_NUMBER_LEN: usize = 3;
pub(crate) const JPG_MAGIC_NUMBER: &[u8; JPG_MAGIC_NUMBER_LEN] = &[0xFF, 0xD8, 0xFF];

#[inline]
pub(crate) fn str_is_empty_or_has_newlines(s: &str) -> bool {
    s.is_empty() || s.chars().any(|c| c == '\n' || c == '\r')
}

#[inline]
pub(crate) fn str_is_hex(s: &str) -> bool {
    !s.chars().any(|c| !c.is_ascii_hexdigit())
}

#[inline]
pub(crate) fn path_is_invalid<P: AsRef<Path>>(p: P) -> bool {
    let p = p.as_ref();
    !p.is_file() || !p.is_relative() || p.extension().is_none() || p.parent().is_some()
}
