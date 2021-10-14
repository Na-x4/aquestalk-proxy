use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::aquestalk::Wav;

#[derive(Serialize, Deserialize, Getters, Debug)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Req {
    #[serde(rename = "type", default = "default_type")]
    voice_type: String,
    #[serde(default = "default_speed")]
    speed: i32,
    koe: String,
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
    #[serde(rename = "error")]
    AquesTalkError {
        code: i32,
        message: String,
    },
    #[serde(rename = "error")]
    ServerInternalError {
        message: String,
    },
}

impl Res {
    pub fn from_error<T: std::error::Error>(err: &T) -> Self {
        Self::from_error_message(&format!("{}", err))
    }

    pub fn from_error_message(s: &str) -> Self {
        Res::ServerInternalError {
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
        Res::AquesTalkError {
            code: err.code(),
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
