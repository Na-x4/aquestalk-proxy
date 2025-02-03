// AquesTalk-proxy - Copyright (C) 2021-2022 Na-x4
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

#[cfg(test)]
mod test {
    use aquestalk_proxy::aquestalk::Koe;
    use encoding_rs::SHIFT_JIS;

    use std::{ffi::CString, str::FromStr};

    use crate::aquestalk::{AquesTalkDll, Error as AquesTalkError, Wav};

    const PATH: &str = "../aquestalk";

    fn aqtk_synthe(koe: &str) -> Result<Wav, AquesTalkError> {
        let (koe, _, had_errors) = SHIFT_JIS.encode(koe);
        assert!(!had_errors);
        let aqtk = AquesTalkDll::new(&PATH).unwrap();

        match unsafe { aqtk.synthe_raw("f1", &CString::new(koe).unwrap(), 100) } {
            Some(Ok(wav)) => Ok(wav),
            Some(Err(err)) => Err(err),
            None => unreachable!(),
        }
    }

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_koe_space() {
        let aqtk_err = aqtk_synthe("　");
        let koe_err = Koe::from_str(" ");
        assert_eq!(aqtk_err.err().unwrap(), koe_err.err().unwrap());
    }

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_koe_non_shiftjis_char() {
        let test_str = "🤔";

        let aqtk = AquesTalkDll::new(&PATH).unwrap();
        let aqtk_err = unsafe { aqtk.synthe_raw("f1", &CString::new(test_str).unwrap(), 100) };
        let koe_err = Koe::from_str(test_str);
        assert_eq!(aqtk_err.unwrap().err().unwrap(), koe_err.err().unwrap());
    }

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_koe_long_accent_phrase() {
        let test_str = "あ".repeat(256);

        let aqtk_err = aqtk_synthe(&test_str);
        let koe_err = Koe::from_str(&test_str);
        assert_eq!(aqtk_err.err().unwrap(), koe_err.err().unwrap());
    }
}
