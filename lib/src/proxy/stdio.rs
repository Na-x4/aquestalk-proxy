// Copyright (c) 2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ffi::OsStr;
use std::io::{self, BufReader};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::aquestalk::AquesTalk;
use crate::messages::ResponsePayload;

use super::AquesTalkProxyClient;

type Client = AquesTalkProxyClient<BufReader<ChildStdout>, ChildStdin>;

struct AquesTalkProxyStdioImpl {
    command: Child,
    client: Client,
}

impl AquesTalkProxyStdioImpl {
    fn has_exited(&mut self) -> bool {
        match self.command.try_wait() {
            Ok(None) => (),
            _ => return true,
        }

        if self.client.is_closed() {
            return true;
        }

        false
    }
}

pub struct AquesTalkProxyStdio<S, F> {
    program: S,
    opener: F,
    inner: Arc<Mutex<Option<AquesTalkProxyStdioImpl>>>,
}

impl<S, F> AquesTalkProxyStdio<S, F>
where
    S: AsRef<OsStr>,
    F: Fn(&mut Command) -> &mut Command,
{
    pub fn new(program: S, opener: F) -> Self {
        Self {
            program,
            opener,
            inner: Arc::new(Mutex::new(None)),
        }
    }

    fn open(&self) -> io::Result<AquesTalkProxyStdioImpl> {
        let mut command = (self.opener)(&mut Command::new(&self.program))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let reader = BufReader::new(command.stdout.take().unwrap());
        let writer = command.stdin.take().unwrap();

        let client = AquesTalkProxyClient::new(reader, writer);

        Ok(AquesTalkProxyStdioImpl { command, client })
    }
}

impl<S, F> AquesTalk for AquesTalkProxyStdio<S, F>
where
    S: AsRef<OsStr>,
    F: Fn(&mut Command) -> &mut Command,
{
    type Wav = Vec<u8>;
    fn synthe(
        &self,
        voice_type: &str,
        koe: &str,
        speed: i32,
    ) -> Result<Self::Wav, crate::messages::ResponsePayload> {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_none() || inner.as_mut().unwrap().has_exited() {
            *inner = Some(self.open().map_err(ResponsePayload::from_io_error)?);
        }

        let client = &mut inner.as_mut().unwrap().client;
        client.synthe(voice_type, koe, speed)
    }
}
