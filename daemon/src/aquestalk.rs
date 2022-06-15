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

use aquestalk_proxy::aquestalk::{AquesTalk, Error, Koe};
use aquestalk_proxy::messages::ResponsePayload;

mod dll;
use dll::AquesTalkDllRaw;

mod koe;

fn new_unknown_voice_type_error(voice_type: &str) -> ResponsePayload {
    ResponsePayload::AquestalkError {
        code: None,
        message: format!("不明な声種 ({})", voice_type),
    }
}

#[derive(Clone)]
pub struct AquesTalkDll(HashMap<String, AquesTalkDllImpl>);

impl AquesTalkDll {
    pub fn new<P>(path: &P) -> Result<Self, Box<dyn std::error::Error>>
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
        Ok(Self(aqtks))
    }
}

impl AquesTalk<Wav> for AquesTalkDll {
    fn synthe(&self, voice_type: &str, koe: &str, speed: i32) -> Result<Wav, ResponsePayload> {
        let dll = match self.0.get(&voice_type.to_string()) {
            Some(dll) => dll,
            None => {
                return Err(new_unknown_voice_type_error(voice_type));
            }
        };

        let wav = match dll.synthe(&koe.to_string(), speed) {
            Ok(wav) => wav,
            Err(err) => {
                return Err(ResponsePayload::from(err));
            }
        };

        Ok(wav)
    }
}

#[derive(Debug, Clone)]
struct AquesTalkDllImpl(Arc<Mutex<AquesTalkDllRaw>>);

impl AquesTalkDllImpl {
    fn new<P: AsRef<OsStr>>(filename: P) -> Result<AquesTalkDllImpl, libloading::Error> {
        let dll = AquesTalkDllRaw::new(filename)?;
        Ok(AquesTalkDllImpl(Arc::new(Mutex::new(dll))))
    }

    unsafe fn synthe_raw(&self, koe: &CStr, speed: i32) -> Result<Wav, Error> {
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

    fn synthe_koe(&self, koe: &Koe, speed: i32) -> Result<Wav, Error> {
        unsafe { self.synthe_raw(koe, speed) }
    }

    fn synthe(&self, koe: &str, speed: i32) -> Result<Wav, Error> {
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