// Copyright (c) 2021-2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub is_success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_close: Option<bool>,
    pub response: ResponsePayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<Value>,
}

impl Response {
    pub fn new(status: ResponseStatus, payload: ResponsePayload, request: Option<Value>) -> Self {
        let (is_success, close) = match status {
            ResponseStatus::Success => (true, None),
            ResponseStatus::RecoverableError => (false, None),
            ResponseStatus::Error => (false, Some(true)),
        };
        Self {
            is_success,
            will_close: close,
            response: payload,
            request,
        }
    }
}

pub enum ResponseStatus {
    Success,
    RecoverableError,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum ResponsePayload {
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
    IoError {
        message: String,
    },
}

impl From<&'_ [u8]> for ResponsePayload {
    fn from(wav: &'_ [u8]) -> Self {
        Self::Wav {
            wav: base64::encode(wav),
        }
    }
}

impl TryFrom<ResponsePayload> for Vec<u8> {
    type Error = ResponsePayload;
    fn try_from(value: ResponsePayload) -> Result<Self, Self::Error> {
        match value {
            ResponsePayload::Wav { wav } => {
                base64::decode(wav).map_err(|e| ResponsePayload::IoError {
                    message: e.to_string(),
                })
            }
            value => Err(value),
        }
    }
}

impl From<crate::aquestalk::Error> for ResponsePayload {
    fn from(err: crate::aquestalk::Error) -> Self {
        Self::AquestalkError {
            code: Some(err.code()),
            message: err.message().to_string(),
        }
    }
}

impl From<serde_json::Error> for ResponsePayload {
    fn from(err: serde_json::Error) -> Self {
        if !err.is_io() {
            Self::JsonError {
                message: err.to_string(),
            }
        } else {
            let err: io::Error = err.into();
            Self::IoError {
                message: err.to_string(),
            }
        }
    }
}
