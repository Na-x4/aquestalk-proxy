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

use std::collections::HashMap;
use std::io::BufWriter;
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use getopts::Options;
use threadpool::ThreadPool;

use aquestalk_proxy::aquestalk::{load_libs, AquesTalk};

use crate::GeneralOptions;

struct TcpProxyOptions {
    lib_path: PathBuf,
    addr: SocketAddr,
    num_threads: usize,
    timeout: Option<Duration>,
    limit: Option<u64>,
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} tcp [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_options(
    GeneralOptions {
        program,
        args,
        lib_path,
    }: GeneralOptions,
) -> Option<TcpProxyOptions> {
    let mut opts = Options::new();
    opts.optopt(
        "l",
        "listen",
        "specify the port/address to listen on",
        "ADDR",
    );
    opts.optopt("n", "threads", "specifies the number of threads", "NUM");
    opts.optopt("", "timeout", "", "MILLIS");
    opts.optopt("", "limit", "", "BYTES");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return None;
    }

    let addr = matches
        .opt_get_default::<SocketAddr>(
            "l",
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 21569),
        )
        .unwrap();
    let num_threads = matches.opt_get_default("n", 1).unwrap();
    let timeout = matches
        .opt_get("timeout")
        .unwrap()
        .and_then(|t| Some(Duration::from_millis(t)));
    let limit = matches.opt_get("limit").unwrap();

    Some(TcpProxyOptions {
        lib_path,
        addr,
        num_threads,
        timeout,
        limit,
    })
}

fn handle_connection(
    stream: TcpStream,
    aqtks: HashMap<String, AquesTalk>,
    timeout: Option<Duration>,
    limit: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_read_timeout(timeout)?;
    super::proxy(
        stream.try_clone()?,
        BufWriter::new(stream.try_clone()?),
        aqtks,
        limit,
    )?;
    stream.shutdown(Shutdown::Write)?;
    Ok(())
}

pub fn run_tcp_proxy(options: GeneralOptions) {
    let options = match parse_options(options) {
        Some(options) => options,
        None => return,
    };

    let libs = load_libs(&options.lib_path).unwrap();
    let listener = TcpListener::bind(options.addr).unwrap();
    let pool = ThreadPool::new(options.num_threads);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let aqtks = libs.clone();
        let timeout = options.timeout;
        let limit = options.limit;

        pool.execute(move || {
            handle_connection(stream, aqtks, timeout, limit)
                .unwrap_or_else(|err| eprintln!("{}", err));
        });
    }
}
