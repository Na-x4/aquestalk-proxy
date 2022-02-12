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

use std::collections::HashMap;
use std::ffi::{CStr, OsStr};
use std::fs;
use std::path::Path;
use std::slice;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use aquestalk_proxy::aquestalk::{Error, Koe};

mod dll;
use dll::AquesTalkDllRaw;

mod koe;

#[derive(Debug, Clone)]
pub struct AquesTalkDllImpl(Arc<Mutex<AquesTalkDllRaw>>);

impl AquesTalkDllImpl {
    pub fn new<P: AsRef<OsStr>>(filename: P) -> Result<AquesTalkDllImpl, libloading::Error> {
        let dll = AquesTalkDllRaw::new(filename)?;
        Ok(AquesTalkDllImpl(Arc::new(Mutex::new(dll))))
    }

    pub unsafe fn synthe_raw(&self, koe: &CStr, speed: i32) -> Result<Wav, Error> {
        let dll = &self.0;

        let (wav, size) = {
            let mut dll = dll.lock().unwrap();
            dll.synthe(koe.as_ptr(), speed)
        };

        if wav.is_null() {
            return Err(Error::new(size as i32));
        }

        Ok(Wav {
            wav,
            size,
            dll: Arc::clone(dll),
        })
    }

    pub fn synthe_koe(&self, koe: &Koe, speed: i32) -> Result<Wav, Error> {
        unsafe { self.synthe_raw(koe, speed) }
    }

    pub fn synthe(&self, koe: &str, speed: i32) -> Result<Wav, Error> {
        let koe = Koe::from_str(koe)?;
        self.synthe_koe(&koe, speed)
    }
}

#[derive(Debug)]
pub struct Wav {
    wav: *const u8,
    size: usize,
    dll: Arc<Mutex<AquesTalkDllRaw>>,
}

impl AsRef<[u8]> for Wav {
    fn as_ref(&self) -> &[u8] {
        let wav: &[u8];
        unsafe {
            wav = slice::from_raw_parts(self.wav, self.size);
        }
        wav
    }
}

impl Drop for Wav {
    fn drop(&mut self) {
        let mut dll = self.dll.lock().unwrap();
        unsafe {
            dll.free_wave(self.wav);
        }
    }
}

pub fn load_libs<P>(
    path: &P,
) -> Result<HashMap<String, AquesTalkDllImpl>, Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
{
    let mut aqtks = HashMap::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let voice_type = entry.file_name().into_string().unwrap();
            let mut path = entry.path();
            path.push("AquesTalk.dll");
            aqtks.insert(voice_type, AquesTalkDllImpl::new(path.into_os_string())?);
        }
    }
    Ok(aqtks)
}
