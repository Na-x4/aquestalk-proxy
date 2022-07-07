// Copyright (c) 2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ffi::OsStr;
use std::io::{self, BufReader};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio as StdioEnum};
use std::sync::{Arc, Mutex};

use crate::aquestalk::AquesTalk;
use crate::messages::ResponsePayload;

type Client = super::Client<BufReader<ChildStdout>, ChildStdin>;

struct StdioClientImpl {
    command: Child,
    client: Client,
}

impl StdioClientImpl {
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

pub struct StdioClient<S, F> {
    program: S,
    opener: F,
    inner: Arc<Mutex<Option<StdioClientImpl>>>,
}

impl<S, F> StdioClient<S, F>
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

    fn open(&self) -> io::Result<StdioClientImpl> {
        let mut command = (self.opener)(&mut Command::new(&self.program))
            .stdin(StdioEnum::piped())
            .stdout(StdioEnum::piped())
            .spawn()?;
        let reader = BufReader::new(command.stdout.take().unwrap());
        let writer = command.stdin.take().unwrap();

        let client = Client::new(reader, writer);

        Ok(StdioClientImpl { command, client })
    }
}

impl<S, F> AquesTalk for StdioClient<S, F>
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

#[cfg(test)]
mod test {
    use std::env;
    use std::fs;
    use std::io::ErrorKind;
    use std::path::Path;

    use crate::aquestalk::AquesTalk;
    use crate::StdioClient;

    #[cfg_attr(not(windows), ignore)]
    #[test]
    fn windows_exe() {
        let exe_path = Path::new("../target/i686-pc-windows-gnu/release/aquestalk-proxyd.exe");
        if let Err(err) = fs::metadata(&exe_path) {
            if err.kind() == ErrorKind::NotFound {
                panic!("Run this test after running \"cross build --target=i686-pc-windows-gnu --release\"");
            }

            panic!("{:?}", err);
        }

        let aqtk = StdioClient::new(&exe_path, |c| c.arg("--path=../aquestalk").arg("stdio"));
        aqtk.synthe("f1", "こんにちわ、せ'かい", 100).unwrap();
        aqtk.synthe("f1", "ゆっくりしていってね", 100).unwrap();
    }

    #[test]
    fn docker_container() {
        let aqtk = StdioClient::new("docker", |c| {
            c.arg("run")
                .arg("-i")
                .arg("--rm")
                .arg("--platform=linux/386")
                .arg(env::var("AQTK_PROXY_IMAGE").unwrap_or("nax4/aquestalk-proxy".into()))
                .arg("stdio")
        });
        aqtk.synthe("f1", "こんにちわ、せ'かい", 100).unwrap();
        aqtk.synthe("f1", "ゆっくりしていってね", 100).unwrap();
    }
}
