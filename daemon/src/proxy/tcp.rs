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

use std::io::BufWriter;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use aquestalk_proxyd::aquestalk::AquesTalkDll;
use getopts::Options;
use threadpool::ThreadPool;

use crate::GeneralOptions;

struct TcpProxyOptions {
    lib_path: PathBuf,
    addrs: Vec<String>,
    num_threads: usize,
    timeout: Option<Duration>,
    limit: Option<u64>,
}

fn format_usage(program: &str, opts: Options) -> String {
    format!(
        "\
AquesTalk-proxy TCP Socket Mode

USAGE:
    {} tcp [OPTIONS]

OPTIONS:
{}
",
        program,
        opts.usage_with_format(|opts| { opts.collect::<Vec<String>>().join("\n") })
    )
}

fn parse_options(
    GeneralOptions {
        program,
        args,
        lib_path,
    }: GeneralOptions,
) -> Result<TcpProxyOptions, i32> {
    let mut opts = Options::new();
    opts.optmulti(
        "l",
        "listen",
        "Address and port to listen on (multiple allowed)",
        "ADDR",
    );
    opts.optopt("n", "threads", "Number of threads for handling requests", "NUM");
    opts.optopt(
        "",
        "timeout",
        "Connection timeout in milliseconds",
        "MILLIS",
    );
    opts.optopt(
        "",
        "limit",
        "Max total request size per session",
        "BYTES",
    );
    opts.optflag("h", "help", "Print help");

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}\nERROR: {}", format_usage(&program, opts), f.to_string());
            return Err(1);
        }
    };

    if matches.opt_present("h") {
        println!("{}", format_usage(&program, opts));
        return Err(0);
    }

    let addrs = matches.opt_strs("l");
    let addrs = if addrs.len() > 0 {
        addrs
    } else {
        vec!["127.0.0.1:21569".into(), "[::1]:21569".into()]
    };
    let num_threads = matches.opt_get_default("n", 1).unwrap();
    let timeout = matches
        .opt_get("timeout")
        .unwrap()
        .and_then(|t| Some(Duration::from_millis(t)));
    let limit = matches.opt_get("limit").unwrap();

    Ok(TcpProxyOptions {
        lib_path,
        addrs,
        num_threads,
        timeout,
        limit,
    })
}

fn handle_connection(
    stream: TcpStream,
    aqtk: AquesTalkDll,
    timeout: Option<Duration>,
    limit: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_read_timeout(timeout)?;
    super::proxy(
        stream.try_clone()?,
        BufWriter::new(stream.try_clone()?),
        aqtk,
        limit,
    )?;
    stream.shutdown(Shutdown::Write)?;
    Ok(())
}

pub fn run_tcp_proxy(options: GeneralOptions) -> i32 {
    let options = match parse_options(options) {
        Ok(options) => options,
        Err(err) => return err,
    };

    let aqtk = AquesTalkDll::new(&options.lib_path).unwrap();
    let pool = Arc::new(Mutex::new(ThreadPool::new(options.num_threads)));

    (options.addrs)
        .iter()
        .map(|addr| {
            let listener = TcpListener::bind(addr).unwrap();
            let aqtk = aqtk.clone();
            let timeout = options.timeout;
            let limit = options.limit;
            let pool = Arc::clone(&pool);

            thread::spawn(move || {
                for stream in listener.incoming() {
                    let stream = stream.unwrap();
                    let aqtk = aqtk.clone();
                    let timeout = timeout;
                    let limit = limit;

                    pool.lock().unwrap().execute(move || {
                        handle_connection(stream, aqtk, timeout, limit)
                            .unwrap_or_else(|err| eprintln!("{}", err));
                    });
                }
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|t| t.join().unwrap());

    0
}
