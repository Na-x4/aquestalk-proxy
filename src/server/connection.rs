use std::collections::HashMap;
use std::io::{Read, Write};

use optional_take::io::Takable;
use serde_json::{Deserializer, Value};

use crate::aquestalk::AquesTalk;

use super::messages::{Request, Response, ResponseImpl};

pub fn new_voice_type_error(voice_type: String) -> ResponseImpl {
    ResponseImpl::AquestalkError {
        code: None,
        message: format!("不明な声種 ({})", voice_type),
    }
}

pub fn new_limit_reached_error() -> ResponseImpl {
    ResponseImpl::ConnectionError {
        message: "Request is too long".to_string(),
    }
}

fn write_response<W>(
    writer: &mut W,
    is_success: bool,
    is_connection_reusable: bool,
    response: ResponseImpl,
) -> serde_json::Result<()>
where
    W: Write,
{
    serde_json::to_writer(
        writer,
        &Response {
            is_success,
            is_connection_reusable,
            response,
        },
    )
}

pub fn handle_connection<R, W>(
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
    for req in deserializer {
        let req = match req {
            Ok(req) => req,
            Err(err) => {
                let response = if err.is_eof() && reader.limit() == Some(0) {
                    new_limit_reached_error()
                } else {
                    ResponseImpl::from(err)
                };

                write_response(&mut writer, false, false, response)?;
                break;
            }
        };

        let req: Request = match serde_json::from_value(req) {
            Ok(req) => req,
            Err(err) => {
                write_response(&mut writer, false, true, ResponseImpl::from(err))?;
                continue;
            }
        };

        let aq = match aqtks.get(&req.voice_type) {
            Some(aq) => aq,
            None => {
                write_response(
                    &mut writer,
                    false,
                    true,
                    new_voice_type_error(req.voice_type),
                )?;
                continue;
            }
        };

        let wav = match aq.synthe(&req.koe, req.speed) {
            Ok(wav) => wav,
            Err(err) => {
                write_response(&mut writer, false, true, ResponseImpl::from(err))?;
                continue;
            }
        };

        write_response(&mut writer, true, true, ResponseImpl::from(wav))?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use super::handle_connection;
    use crate::aquestalk::load_libs;

    #[test]
    fn test_connection() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        handle_connection(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

        match response {
            Value::Object(object) => {
                for (key, value) in object {
                    if key == "isConnectionReusable" {
                        assert_eq!(value, json!(true));
                    } else if key == "isSuccess" {
                        assert_eq!(value, json!(true));
                    } else if key == "response" {
                        match value {
                            Value::Object(object) => {
                                for (key, value) in object {
                                    if key == "type" {
                                        assert_eq!(value, json!("Wav"));
                                    } else if key == "wav" {
                                        assert!(value.is_string());
                                    } else {
                                        unreachable!();
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }

            _ => unreachable!(),
        }
    }

    #[test]
    fn test_reach_limit() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        handle_connection(input, &mut output, libs, Some(37)).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

        assert_eq!(
            response,
            json!({
              "isConnectionReusable": false,
              "isSuccess": false,
              "response": {
                "type": "ConnectionError",
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

        handle_connection(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

        assert_eq!(
            response,
            json!({
              "isConnectionReusable": false,
              "isSuccess": false,
              "response": {
                "type": "JsonError",
                "message": "EOF while parsing an object at line 1 column 37"
              }
            }
            )
        );
    }

    #[test]
    fn test_json_error_reusable() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koee\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        handle_connection(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

        assert_eq!(
            response,
            json!({
              "isConnectionReusable": true,
              "isSuccess": false,
              "response": {
                "type": "JsonError",
                "message": "unknown field `koee`, expected one of `type`, `speed`, `koe`"
              }
            }
            )
        );
    }

    #[test]
    fn test_invalid_voice_type() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"type\":\"invalid type\",\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        handle_connection(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

        assert_eq!(
            response,
            json!({
              "isConnectionReusable": true,
              "isSuccess": false,
              "response": {
                "type": "AquestalkError",
                "message": "不明な声種 (invalid type)"
              }
            }
            )
        );
    }

    #[test]
    fn test_aqtk_error() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koe\":\"🤔\"}".as_bytes();
        let mut output = Vec::new();

        handle_connection(input, &mut output, libs, None).unwrap();
        let response: Value = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

        assert_eq!(
            response,
            json!({
              "isConnectionReusable": true,
              "isSuccess": false,
              "response": {
                "type": "AquestalkError",
                "code": 105,
                "message": "音声記号列に未定義の読み記号が指定された"
              }
            }
            )
        );
    }
}
