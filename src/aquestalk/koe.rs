use encoding_rs::SHIFT_JIS;

use std::ffi::{CStr, CString};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

pub struct Koe(CString);

impl Deref for Koe {
    type Target = CStr;

    fn deref(&self) -> &CStr {
        &self.0
    }
}

impl FromStr for Koe {
    type Err = KoeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 0 || s.find(" ").is_some() {
            return Err(KoeError {
                kind: KoeErrorKind::Empty,
            });
        }

        let (koe, _, had_errors) = SHIFT_JIS.encode(s);
        if had_errors {
            return Err(KoeError {
                kind: KoeErrorKind::Invalid,
            });
        }

        let koe = match CString::new(koe) {
            Err(_) => {
                return Err(KoeError {
                    kind: KoeErrorKind::Invalid,
                })
            }
            Ok(koe) => koe,
        };

        Ok(Koe(koe))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KoeError {
    kind: KoeErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum KoeErrorKind {
    Empty,
    Invalid,
}

impl fmt::Display for KoeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            KoeErrorKind::Empty => "KoeError::Empty",
            KoeErrorKind::Invalid => "KoeError::Invalid",
        }
        .fmt(f)
    }
}

impl std::error::Error for KoeError {}
