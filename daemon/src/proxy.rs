use std::collections::HashMap;
use std::io::{Read, Write};

use optional_take::io::Takable;
use serde_json::{Deserializer, Value};

use aquestalk_proxy::aquestalk::AquesTalk;
use aquestalk_proxy::messages::{
    Request, Response, ResponsePayload,
    ResponseStatus::{self, *},
};

mod stdio;
pub use stdio::run_stdio_proxy;

mod tcp;
pub use tcp::run_tcp_proxy;

pub fn new_voice_type_error(voice_type: String) -> ResponsePayload {
    ResponsePayload::AquestalkError {
        code: None,
        message: format!("不明な声種 ({})", voice_type),
    }
}

pub fn new_limit_reached_error() -> ResponsePayload {
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
    writer.flush()?;
    Ok(())
}

pub fn proxy<R, W>(
    reader: R,
    mut writer: W,
    aqtks: HashMap<String, AquesTalk>,
    limit: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: Read,
    W: Write,
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

        let aq = match aqtks.get(&request.voice_type) {
            Some(aq) => aq,
            None => {
                write_response(RecoverableError, new_voice_type_error(request.voice_type))?;
                continue;
            }
        };

        let wav = match aq.synthe(&request.koe, request.speed) {
            Ok(wav) => wav,
            Err(err) => {
                write_response(RecoverableError, ResponsePayload::from(err))?;
                continue;
            }
        };

        write_response(Success, ResponsePayload::from(wav))?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use super::proxy;
    use aquestalk_proxy::aquestalk::load_libs;

    #[test]
    fn test_success() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, libs, None).unwrap();
        let mut response: Value =
            serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();
        if response["response"]["wav"].is_string() {
            response["response"]["wav"] = json!("===WAV DATA===");
        }

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
    fn test_reach_limit() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, libs, Some(37)).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

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
    fn test_json_error() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

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
    fn test_json_recoverable_error() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koee\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

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
    fn test_invalid_voice_type() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"type\":\"invalid type\",\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

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
    fn test_aqtk_error() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koe\":\"🤔\"}".as_bytes();
        let mut output = Vec::new();

        proxy(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

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
