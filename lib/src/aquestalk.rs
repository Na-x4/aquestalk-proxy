// Copyright (c) 2021-2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

mod koe;
pub use koe::Koe;

use crate::messages::ResponsePayload;

pub trait AquesTalk<T>
where
    T: AsRef<[u8]>,
{
    fn synthe(&self, voice_type: &str, koe: &str, speed: i32) -> Result<T, ResponsePayload>;
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
