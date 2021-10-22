extern crate aquestalk_proxy as lib;
use lib::server::AquesTalkProxyServer;

use getopts::Options;

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

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
    opts.optopt("p", "path", "", "PATH");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let listen = matches
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
    let path = matches
        .opt_get_default("p", {
            let mut path = env::current_dir().unwrap();
            path.push("bin");
            path
        })
        .unwrap();

    let mut server = AquesTalkProxyServer::new(&path.as_os_str()).unwrap();
    server.set_num_threads(num_threads);
    server.set_timeout(timeout);
    server.set_limit(limit);

    server.run(listen);
}
