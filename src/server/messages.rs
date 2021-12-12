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

use serde::{Deserialize, Serialize};

use crate::aquestalk::Wav;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Req {
    #[serde(rename = "type", default = "default_type")]
    pub voice_type: String,
    #[serde(default = "default_speed")]
    pub speed: i32,
    pub koe: String,
}

fn default_type() -> String {
    "f1".to_string()
}

fn default_speed() -> i32 {
    100
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "snake_case")]
pub enum Res {
    Success {
        wav: String,
    },
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<i32>,
        message: String,
    },
}

impl Res {
    pub fn from_error<T: std::error::Error>(err: &T) -> Self {
        Self::from_error_message(&format!("{}", err))
    }

    pub fn from_error_message(s: &str) -> Self {
        Res::Error {
            code: None,
            message: s.to_string(),
        }
    }
}

impl From<Wav> for Res {
    fn from(wav: Wav) -> Self {
        Res::Success {
            wav: base64::encode(wav.as_ref()),
        }
    }
}

impl From<crate::aquestalk::Error> for Res {
    fn from(err: crate::aquestalk::Error) -> Self {
        Res::Error {
            code: Some(err.code()),
            message: err.message().to_string(),
        }
    }
}

impl From<Result<Wav, crate::aquestalk::Error>> for Res {
    fn from(result: Result<Wav, crate::aquestalk::Error>) -> Self {
        match result {
            Ok(wav) => wav.into(),
            Err(err) => err.into(),
        }
    }
}
