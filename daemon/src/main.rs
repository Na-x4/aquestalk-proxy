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

use std::env;

use getopts::{Options, ParsingStyle};

use aquestalk_proxy::aquestalk::load_libs;

mod proxy;
use proxy::{run_stdio_proxy, run_tcp_proxy};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.parsing_style(ParsingStyle::StopAtFirstFree);
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

    let lib_path = matches
        .opt_get_default("p", {
            let mut path = env::current_dir().unwrap();
            path.push("aquestalk");
            path
        })
        .unwrap();

    let sub_command: &str = if !matches.free.is_empty() {
        &matches.free[0]
    } else {
        print_usage(&program, opts);
        return;
    };
    let sub_command_args = &matches.free[1..];
    match sub_command {
        "tcp" => run_tcp_proxy(&program, sub_command_args, load_libs(&lib_path).unwrap()),
        "stdio" => run_stdio_proxy(&program, sub_command_args, load_libs(&lib_path).unwrap()),
        _ => panic!("Unknown sub command ({})", sub_command),
    }
}
