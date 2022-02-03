// Copyright (c) 2021-2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashMap;
use std::ffi::{CStr, OsStr};
use std::fmt;
use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::slice;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

mod dll;
use dll::AquesTalkDll;

mod koe;
pub use koe::Koe;

#[derive(Debug, Clone)]
pub struct AquesTalk(Arc<Mutex<AquesTalkDll>>);

impl AquesTalk {
    pub fn new<P: AsRef<OsStr>>(filename: P) -> Result<AquesTalk, libloading::Error> {
        let dll = AquesTalkDll::new(filename)?;
        Ok(AquesTalk(Arc::new(Mutex::new(dll))))
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    code: i32,
}

impl Error {
    pub fn new(code: i32) -> Error {
        let error = Error { code };
        error.message();
        error
    }

    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn message(&self) -> &'static str {
        match self.code {
            100 => "その他のエラー",
            101 => "メモリ不足",
            102 => "音声記号列に未定義の読み記号が指定された",
            103 => "韻律データの時間長がマイナスなっている",
            104 => "内部エラー(未定義の区切りコード検出）",
            105 => "音声記号列に未定義の読み記号が指定された",
            106 => "音声記号列のタグの指定が正しくない",
            107 => "タグの長さが制限を越えている（または[>]がみつからない）",
            108 => "タグ内の値の指定が正しくない",
            109 => "WAVE 再生ができない（サウンドドライバ関連の問題）",
            110 => "WAVE 再生ができない（サウンドドライバ関連の問題非同期再生）",
            111 => "発声すべきデータがない",
            200 => "音声記号列が長すぎる",
            201 => "１つのフレーズ中の読み記号が多すぎる",
            202 => "音声記号列が長い（内部バッファオーバー1）",
            203 => "ヒープメモリ不足",
            204 => "音声記号列が長い（内部バッファオーバー1）",
            _ => panic!("unknown error code ({})", self.code),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Wav {
    wav: *const u8,
    size: usize,
    dll: Arc<Mutex<AquesTalkDll>>,
}

impl Wav {
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Deref for Wav {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
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

pub fn load_libs<P>(path: &P) -> Result<HashMap<String, AquesTalk>, Box<dyn std::error::Error>>
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
            aqtks.insert(voice_type, AquesTalk::new(path.into_os_string())?);
        }
    }
    Ok(aqtks)
}
