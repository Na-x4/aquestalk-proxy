// Copyright (c) 2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::{BufRead, Write};

use crate::messages::{Request, Response, ResponsePayload};

pub mod stdio;

pub struct AquesTalkProxyClient<R, W>
where
    R: BufRead,
    W: Write,
{
    reader: R,
    writer: Option<W>,
}

impl<R, W> AquesTalkProxyClient<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn new(reader: R, writer: W) -> Self {
        AquesTalkProxyClient {
            reader,
            writer: Some(writer),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.writer.is_none()
    }

    pub fn synthe(
        &mut self,
        voice_type: &str,
        koe: &str,
        speed: i32,
    ) -> Result<Vec<u8>, ResponsePayload> {
        let mut writer = self.writer.as_mut().ok_or(ResponsePayload::IoError {
            message: "Writer is already closed.".to_string(),
        })?;

        serde_json::to_writer(
            &mut writer,
            &Request {
                voice_type: voice_type.into(),
                koe: koe.into(),
                speed,
            },
        )?;
        writer.flush().map_err(ResponsePayload::from_io_error)?;

        let mut response = String::new();
        self.reader
            .read_line(&mut response)
            .map_err(ResponsePayload::from_io_error)?;
        let response: Response = serde_json::from_str(&response)?;

        if response.will_close.unwrap_or(false) {
            drop(self.writer.take().unwrap());
        }
        if !response.is_success {
            return Err(response.response);
        }
        let wav = response.response.try_into()?;

        Ok(wav)
    }
}
