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
