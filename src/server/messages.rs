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

use std::io;

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

impl Response {
    pub fn new(status: ResponseStatus, response: ResponseImpl) -> Self {
        let (is_success, is_connection_reusable) = match status {
            ResponseStatus::Success => (true, true),
            ResponseStatus::Reusable => (false, true),
            ResponseStatus::Failure => (false, false),
        };
        Self {
            is_connection_reusable,
            is_success,
            response,
        }
    }
}

pub enum ResponseStatus {
    Success,
    Reusable,
    Failure,
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
        if !err.is_io() {
            ResponseImpl::JsonError {
                message: err.to_string(),
            }
        } else {
            let err: io::Error = err.into();
            ResponseImpl::ConnectionError {
                message: err.to_string(),
            }
        }
    }
}
