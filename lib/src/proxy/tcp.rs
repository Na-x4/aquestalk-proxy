// Copyright (c) 2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::BufReader;
use std::net::{TcpStream, ToSocketAddrs};

use crate::aquestalk::AquesTalk;
use crate::messages::ResponsePayload;

type Client<'a> = super::Client<BufReader<&'a TcpStream>, &'a TcpStream>;

pub struct TcpClient<A> {
    addr: A,
}

impl<A> TcpClient<A>
where
    A: ToSocketAddrs,
{
    pub fn new(addr: A) -> Self {
        Self { addr }
    }
}

impl<A> AquesTalk for TcpClient<A>
where
    A: ToSocketAddrs,
{
    type Wav = Vec<u8>;
    fn synthe(
        &self,
        voice_type: &str,
        koe: &str,
        speed: i32,
    ) -> Result<Self::Wav, crate::messages::ResponsePayload> {
        let stream = TcpStream::connect(&self.addr).map_err(ResponsePayload::from_io_error)?;
        let mut client = Client::new(BufReader::new(&stream), &stream);
        client.synthe(voice_type, koe, speed)
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use crate::aquestalk::AquesTalk;
    use crate::TcpClient;

    #[test]
    fn tcp() {
        let aqtk = TcpClient::new(env::var("AQTK_PROXY").unwrap_or("localhost:21569".into()));
        aqtk.synthe("f1", "こんにちわ、せ'かい", 100).unwrap();
        aqtk.synthe("f1", "ゆっくりしていってね", 100).unwrap();
    }
}
