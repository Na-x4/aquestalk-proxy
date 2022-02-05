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

use std::{
    io::{stdin, stdout},
    path::PathBuf,
};

use aquestalk_proxy::aquestalk::load_libs;
use getopts::Options;

use crate::GeneralOptions;

use super::proxy;

struct StdioProxyOptions {
    lib_path: PathBuf,
}

fn format_usage(program: &str, opts: Options) -> String {
    format!(
        "\
AquesTalk-proxy Standard IO mode

USAGE:
    {} stdio [OPTIONS]

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
) -> Option<StdioProxyOptions> {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}\nERROR: {}", format_usage(&program, opts), f.to_string());
            return None;
        }
    };

    if matches.opt_present("h") {
        println!("{}", format_usage(&program, opts));
        return None;
    }

    Some(StdioProxyOptions { lib_path })
}

pub fn run_stdio_proxy(options: GeneralOptions) {
    let options = match parse_options(options) {
        Some(options) => options,
        None => return,
    };

    let libs = load_libs(&options.lib_path).unwrap();

    proxy(stdin().lock(), stdout().lock(), libs, None).unwrap();
}
