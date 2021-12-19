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
#[serde(deny_unknown_fields)]
pub struct Request {
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Response {
    pub is_connection_reusable: bool,
    pub is_success: bool,
    pub response: ResponseImpl,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum ResponseImpl {
    Wav {
        wav: String,
    },
    AquestalkError {
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<i32>,
        message: String,
    },
    JsonError {
        message: String,
    },
    ConnectionError {
        message: String,
    },
}

impl ResponseImpl {
    pub fn new_voice_type_error(voice_type: String) -> Self {
        ResponseImpl::AquestalkError {
            code: None,
            message: format!("不明な声質 ({})", voice_type),
        }
    }
}

impl From<Wav> for ResponseImpl {
    fn from(wav: Wav) -> Self {
        ResponseImpl::Wav {
            wav: base64::encode(wav.as_ref()),
        }
    }
}

impl From<crate::aquestalk::Error> for ResponseImpl {
    fn from(err: crate::aquestalk::Error) -> Self {
        ResponseImpl::AquestalkError {
            code: Some(err.code()),
            message: err.message().to_string(),
        }
    }
}

impl From<serde_json::Error> for ResponseImpl {
    fn from(err: serde_json::Error) -> Self {
        ResponseImpl::JsonError {
            message: err.to_string(),
        }
    }
}
