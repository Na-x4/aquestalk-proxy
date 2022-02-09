// Copyright (c) 2021-2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use encoding_rs::SHIFT_JIS;

use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::str::FromStr;

use super::Error;

pub struct Koe(CString);

impl Deref for Koe {
    type Target = CStr;

    fn deref(&self) -> &CStr {
        &self.0
    }
}

impl FromStr for Koe {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error { code: 100 });
        }

        if s.contains(" ") || s.contains("\0") {
            return Err(Error { code: 105 });
        }

        for accent_phrase in s.split(&['。', '？', '、', ',', ';', '/', '+'][..]) {
            if accent_phrase.chars().count() > 255 {
                return Err(Error { code: 102 });
            }
        }

        let (koe, _, had_errors) = SHIFT_JIS.encode(s);
        if had_errors {
            return Err(Error { code: 105 });
        }

        let koe = CString::new(koe).unwrap();

        Ok(Koe(koe))
    }
}

#[cfg(test)]
mod test {
    use encoding_rs::SHIFT_JIS;

    use std::{ffi::CString, str::FromStr};

    use super::Koe;
    use crate::aquestalk::{load_libs, Error as AquesTalkError, Wav};

    fn aqtk_synthe(koe: &str) -> Result<Wav, AquesTalkError> {
        let (koe, _, had_errors) = SHIFT_JIS.encode(koe);
        assert!(!had_errors);
        let libs = load_libs(&"./aquestalk").unwrap();
        let f1 = libs.get("f1").unwrap();

        unsafe { f1.synthe_raw(&CString::new(koe).unwrap(), 100) }
    }

    #[test]
    fn test_koe_space() {
        let aqtk_err = aqtk_synthe("　");
        let koe_err = Koe::from_str(" ");
        assert_eq!(aqtk_err.err().unwrap(), koe_err.err().unwrap());
    }

    #[test]
    fn test_koe_non_shiftjis_char() {
        let test_str = "🤔";

        let libs = load_libs(&"./aquestalk").unwrap();
        let f1 = libs.get("f1").unwrap();
        let aqtk_err = unsafe { f1.synthe_raw(&CString::new(test_str).unwrap(), 100) };
        let koe_err = Koe::from_str(test_str);
        assert_eq!(aqtk_err.err().unwrap(), koe_err.err().unwrap());
    }

    #[test]
    fn test_koe_long_accent_phrase() {
        let test_str = "あ".repeat(256);

        let aqtk_err = aqtk_synthe(&test_str);
        let koe_err = Koe::from_str(&test_str);
        assert_eq!(aqtk_err.err().unwrap(), koe_err.err().unwrap());
    }
}