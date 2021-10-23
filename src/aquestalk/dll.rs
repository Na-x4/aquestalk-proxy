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

use std::ffi::OsStr;
use std::os::raw::{c_char, c_int, c_uchar};

type SyntheFuncPtr = unsafe extern "stdcall" fn(*const c_char, c_int, *mut c_int) -> *const c_uchar;
type FreeWaveFuncPtr = unsafe extern "stdcall" fn(*const c_uchar) -> ();

#[derive(Debug)]
pub struct AquesTalkDll(libloading::Library);

impl AquesTalkDll {
    pub fn new<P: AsRef<OsStr>>(filename: P) -> Result<AquesTalkDll, libloading::Error> {
        let lib;
        unsafe {
            lib = libloading::Library::new(filename)?;
            let _synthe: libloading::Symbol<SyntheFuncPtr> = lib.get(b"AquesTalk_Synthe")?;
            let _free_wave: libloading::Symbol<FreeWaveFuncPtr> = lib.get(b"AquesTalk_FreeWave")?;
        }
        Ok(AquesTalkDll(lib))
    }

    pub unsafe fn synthe(&mut self, koe: *const c_char, speed: c_int) -> (*const c_uchar, usize) {
        let lib = &self.0;
        let synthe: libloading::Symbol<SyntheFuncPtr> = lib.get(b"AquesTalk_Synthe").unwrap();

        let mut size: c_int = 0;
        let wav = synthe(koe, speed, &mut size as *mut c_int);
        (wav, size as usize)
    }

    pub unsafe fn free_wave(&mut self, wav: *const c_uchar) {
        let lib = &self.0;
        let free_wave: libloading::Symbol<FreeWaveFuncPtr>;
        free_wave = lib.get(b"AquesTalk_FreeWave").unwrap();
        free_wave(wav);
    }
}
