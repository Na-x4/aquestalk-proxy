// AquesTalk-proxy - Copyright (C) 2021 Na-x4
//
// This file is part of AquesTalk-proxy.
//
// AquesTalk-proxy is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// AquesTalk-proxy is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with AquesTalk-proxy.  If not, see <https://www.gnu.org/licenses/>.

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
        if s.len() == 0 {
            return Err(Error { code: 100 });
        }

        if s.find(" ").is_some() {
            return Err(Error { code: 102 });
        }

        let (koe, _, had_errors) = SHIFT_JIS.encode(s);
        if had_errors {
            return Err(Error { code: 102 });
        }

        let koe = match CString::new(koe) {
            Err(_) => return Err(Error { code: 102 }),
            Ok(koe) => koe,
        };

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
}
