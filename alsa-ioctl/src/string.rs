use std::{fmt::Write, os::raw::c_uchar, str::Utf8Error};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AsciiString<const S: usize>(pub [c_uchar; S]);

impl<const S: usize> AsciiString<S> {
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        let i = self.0.iter().position(|v| *v == 0).unwrap_or(self.0.len());
        std::str::from_utf8(&self.0[0..i])
    }
}

impl<const S: usize> std::fmt::Display for AsciiString<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or("Invalid string"))
    }
}

impl<const S: usize> std::fmt::Debug for AsciiString<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        for byte in self.0.iter() {
            if *byte == 0 {
                break;
            }

            for byte in core::ascii::escape_default(*byte) {
                f.write_char(byte as char)?;
            }
        }
        write!(f, "\"")
    }
}
