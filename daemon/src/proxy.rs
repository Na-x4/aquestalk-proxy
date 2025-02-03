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

use std::io::{Read, Write};

use aquestalk_proxy::aquestalk::AquesTalk;
use optional_take::io::Takable;
use serde_json::{Deserializer, Value};

use aquestalk_proxy::messages::{
    Request, Response, ResponsePayload,
    ResponseStatus::{self, *},
};

mod stdio;
pub use stdio::run_stdio_proxy;

mod tcp;
pub use tcp::run_tcp_proxy;

fn new_limit_reached_error() -> ResponsePayload {
    ResponsePayload::IoError {
        message: "Request is too long".to_string(),
    }
}

fn write_response<W>(
    mut writer: W,
    status: ResponseStatus,
    payload: ResponsePayload,
    request: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>>
where
    W: Write,
{
    serde_json::to_writer(&mut writer, &Response::new(status, payload, request))?;
    writer.write(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn proxy<R, W, A>(
    reader: R,
    mut writer: W,
    aqtk: A,
    limit: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: Read,
    W: Write,
    A: AquesTalk,
{
    let mut reader = reader.take_optional(limit);

    let deserializer = Deserializer::from_reader(&mut reader).into_iter::<Value>();
    for request in deserializer {
        let request = match request {
            Ok(request) => request,
            Err(err) => {
                let payload = if err.is_eof() && reader.limit() == Some(0) {
                    new_limit_reached_error()
                } else {
                    ResponsePayload::from(err)
                };

                write_response(&mut writer, Error, payload, None)?;
                break;
            }
        };

        let write_response = {
            let request = request.clone();
            |status, payload| write_response(&mut writer, status, payload, Some(request))
        };

        let request: Request = match serde_json::from_value(request) {
            Ok(request) => request,
            Err(err) => {
                write_response(RecoverableError, ResponsePayload::from(err))?;
                continue;
            }
        };

        match aqtk.synthe(&request.voice_type, &request.koe, request.speed) {
            Err(err) => write_response(RecoverableError, err)?,
            Ok(wav) => write_response(Success, ResponsePayload::from(wav.as_ref()))?,
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::str;

    use aquestalk_proxyd::aquestalk::AquesTalkDll;
    use serde_json::{json, Value};

    use super::proxy;

    const PATH: &str = "../aquestalk";

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_success() {
        let aqtk = AquesTalkDll::new(&PATH).unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, aqtk, None).unwrap();
        let mut response: Value = serde_json::from_str(str::from_utf8(&output).unwrap()).unwrap();
        if response["response"]["wav"].is_string() {
            response["response"]["wav"] = json!("===WAV DATA===");
        }

        assert_eq!(output.iter().filter(|&&c| c == b'\n').count(), 1);
        assert_eq!(
            response,
            json!(
                {
                    "isSuccess": true,
                    "response": { "type": "Wav", "wav": "===WAV DATA===" },
                    "request": { "koe": "こんにちわ、せ'かい" }
                }
            )
        );
    }

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_reach_limit() {
        let aqtk = AquesTalkDll::new(&PATH).unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, aqtk, Some(37)).unwrap();
        let response: Value = serde_json::from_str(str::from_utf8(&output).unwrap()).unwrap();

        assert_eq!(output.iter().filter(|&&c| c == b'\n').count(), 1);
        assert_eq!(
            response,
            json!(
                {
                    "isSuccess": false,
                    "willClose": true,
                    "response": {
                        "type": "IoError",
                        "message": "Request is too long"
                    }
                }
            )
        );
    }

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_json_error() {
        let aqtk = AquesTalkDll::new(&PATH).unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, aqtk, None).unwrap();
        let response: Value = serde_json::from_str(str::from_utf8(&output).unwrap()).unwrap();

        assert_eq!(output.iter().filter(|&&c| c == b'\n').count(), 1);
        assert_eq!(
            response,
            json!(
                {
                    "isSuccess": false,
                    "willClose": true,
                    "response": {
                        "type": "JsonError",
                        "message": "EOF while parsing an object at line 1 column 37"
                    }
                }
            )
        );
    }

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_json_recoverable_error() {
        let aqtk = AquesTalkDll::new(&PATH).unwrap();
        let input = "{\"koee\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, aqtk, None).unwrap();
        let response: Value = serde_json::from_str(str::from_utf8(&output).unwrap()).unwrap();

        assert_eq!(output.iter().filter(|&&c| c == b'\n').count(), 1);
        assert_eq!(
            response,
            json!(
                {
                    "isSuccess": false,
                    "response": {
                        "type": "JsonError",
                        "message": "unknown field `koee`, expected one of `type`, `speed`, `koe`"
                    },
                    "request": { "koee": "こんにちわ、せ'かい" }
                }
            )
        );
    }

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_invalid_voice_type() {
        let aqtk = AquesTalkDll::new(&PATH).unwrap();
        let input = "{\"type\":\"invalid type\",\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, aqtk, None).unwrap();
        let response: Value = serde_json::from_str(str::from_utf8(&output).unwrap()).unwrap();

        assert_eq!(output.iter().filter(|&&c| c == b'\n').count(), 1);
        assert_eq!(
            response,
            json!(
                {
                    "isSuccess": false,
                    "response": {
                        "type": "AquestalkError",
                        "message": "不明な声種 (invalid type)"
                    },
                    "request": { "type": "invalid type", "koe": "こんにちわ、せ'かい" }
                }
            )
        );
    }

    #[test]
    #[cfg_attr(not(all(windows, target_arch = "x86")), ignore)]
    fn test_aqtk_error() {
        let aqtk = AquesTalkDll::new(&PATH).unwrap();
        let input = "{\"koe\":\"🤔\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, aqtk, None).unwrap();
        let response: Value = serde_json::from_str(str::from_utf8(&output).unwrap()).unwrap();

        assert_eq!(output.iter().filter(|&&c| c == b'\n').count(), 1);
        assert_eq!(
            response,
            json!(
                {
                    "isSuccess": false,
                    "response": {
                        "type": "AquestalkError",
                        "code": 105,
                        "message": "音声記号列に未定義の読み記号が指定された"
                    },
                    "request": { "koe": "🤔" }
                }
            )
        );
    }
}
