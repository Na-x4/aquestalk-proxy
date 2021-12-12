use std::collections::HashMap;
use std::io::{Read, Write};

use serde_json::Deserializer;

use crate::aquestalk::AquesTalk;

use super::messages::{Req, Res};

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
    let reader: Box<dyn Read> = match limit {
        Some(limit) => Box::new(reader.take(limit)),
        None => Box::new(reader),
    };
    let reader = Deserializer::from_reader(reader).into_iter::<Req>();

    for req in reader {
        let req = match req {
            Ok(req) => req,
            Err(ref err) => {
                serde_json::to_writer(&mut writer, &Res::from_error(err))?;
                break;
            }
        };
        let aq = match aqtks.get(&req.voice_type) {
            Some(aq) => aq,
            None => {
                serde_json::to_writer(
                    &mut writer,
                    &Res::from_error_message(&format!("不明な声質 ({})", req.voice_type)),
                )?;
                continue;
            }
        };
        let wav = aq.synthe(&req.koe, req.speed);

        serde_json::to_writer(&mut writer, &Res::from(wav))?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::handle_connection;
    use crate::{aquestalk::load_libs, server::messages::Res};

    #[test]
    fn test_connection() {
        let libs = load_libs(&"./aquestalk").unwrap();
        let input = "{\"koe\":\"こんにちわ、せ'かい\"}".as_bytes();
        let mut output = Vec::new();

        handle_connection(input, &mut output, libs, None).unwrap();
        let output: Res = serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

        match output {
            Res::Success { wav: _ } => (),
            _ => unreachable!(),
        };
    }
}
