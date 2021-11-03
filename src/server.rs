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

pub mod messages;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::str::FromStr;
use std::time::Duration;

use serde_json::Deserializer;
use threadpool::ThreadPool;

use crate::aquestalk::{AquesTalk, Koe};
use crate::server::messages::{Req, Res};

pub struct AquesTalkProxyServer {
    aqtks: HashMap<String, AquesTalk>,
    num_threads: usize,
    timeout: Option<Duration>,
    limit: Option<u64>,
}

impl AquesTalkProxyServer {
    pub fn new(
        libs: HashMap<String, AquesTalk>,
    ) -> Result<AquesTalkProxyServer, Box<dyn std::error::Error>> {
        Ok(AquesTalkProxyServer {
            aqtks: libs,
            num_threads: 1,
            timeout: None,
            limit: None,
        })
    }

    pub fn set_num_threads(&mut self, num_threads: usize) {
        self.num_threads = num_threads;
    }

    pub fn set_timeout(&mut self, dur: Option<Duration>) {
        self.timeout = dur;
    }

    pub fn set_limit(&mut self, limit: Option<u64>) {
        self.limit = limit;
    }

    pub fn run<A>(&self, addr: A)
    where
        A: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).unwrap();
        let pool = ThreadPool::new(self.num_threads);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let aqtks = self.aqtks.clone();
            let timeout = self.timeout;
            let limit = self.limit;

            pool.execute(move || {
                Self::handle_connection(stream, aqtks, timeout, limit)
                    .unwrap_or_else(|err| eprintln!("{}", err));
            });
        }
    }

    fn handle_connection(
        mut stream: TcpStream,
        aqtks: HashMap<String, AquesTalk>,
        timeout: Option<Duration>,
        limit: Option<u64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        stream.set_read_timeout(timeout)?;

        let reader = stream.try_clone()?;
        let reader: Box<dyn Read> = match limit {
            Some(limit) => Box::new(reader.take(limit)),
            None => Box::new(reader),
        };
        let reader = Deserializer::from_reader(reader).into_iter::<Req>();

        for req in reader {
            let req = match req {
                Ok(req) => req,
                Err(ref err) => {
                    serde_json::to_writer(&stream, &Res::from_error(err))?;
                    break;
                }
            };
            let aq = match aqtks.get(req.voice_type()) {
                Some(aq) => aq,
                None => {
                    serde_json::to_writer(
                        &stream,
                        &Res::from_error_message(&format!("不明な声質 ({})", req.voice_type())),
                    )?;
                    continue;
                }
            };
            let koe = match Koe::from_str(req.koe()) {
                Ok(koe) => koe,
                Err(ref err) => {
                    serde_json::to_writer(&stream, &Res::from_error(err))?;
                    continue;
                }
            };
            let wav = aq.synthe(&koe, *req.speed());

            serde_json::to_writer(&stream, &Res::from(wav))?;
        }

        stream.flush()?;
        Ok(())
    }
}
