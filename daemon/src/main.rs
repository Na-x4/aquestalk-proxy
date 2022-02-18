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

use std::{env, path::PathBuf};

use getopts::{Options, ParsingStyle};

mod proxy;
use proxy::{run_stdio_proxy, run_tcp_proxy};

pub struct GeneralOptions {
    program: String,
    args: Vec<String>,
    lib_path: PathBuf,
}

fn format_usage(program: &str, opts: Options) -> String {
    format!(
        "\
AquesTalk-proxy

USAGE:
    {} [OPTIONS] [MODE]

OPTIONS:
{}

MODE:
    tcp                 TCP Socket mode
    stdio               Standard IO mode (Default)
",
        program,
        opts.usage_with_format(|opts| { opts.collect::<Vec<String>>().join("\n") })
    )
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
            eprintln!("{}\nERROR: {}", format_usage(&program, opts), f.to_string());
            return;
        }
    };

    if matches.opt_present("h") {
        println!("{}", format_usage(&program, opts));
        return;
    }

    let lib_path = matches
        .opt_get_default("p", {
            let mut path = env::current_dir().unwrap();
            path.push("aquestalk");
            path
        })
        .unwrap();

    let (mode, args): (&str, Vec<String>) = if !matches.free.is_empty() {
        (&matches.free[0], matches.free[1..].to_vec())
    } else {
        ("stdio", Vec::<String>::new())
    };
    let options = GeneralOptions {
        program,
        args,
        lib_path,
    };
    match mode {
        "tcp" => run_tcp_proxy(options),
        "stdio" => run_stdio_proxy(options),
        _ => {
            eprintln!(
                "{}\nERROR: Unknown mode \"{}\"",
                format_usage(&options.program, opts),
                mode
            );
            return;
        }
    }
}
