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
    collections::HashMap,
    io::{stdin, stdout},
};

use aquestalk_proxy::aquestalk::AquesTalk;
use getopts::Options;

use super::proxy;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} stdio [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_options(program: &str, args: &[String]) -> Option<()> {
    let mut opts = Options::new();
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

    Some(())
}

pub fn run_stdio_proxy(program: &str, args: &[String], libs: HashMap<String, AquesTalk>) {
    let _options = match parse_options(program, args) {
        Some(options) => options,
        None => return,
    };

    proxy(stdin().lock(), stdout().lock(), libs, None).unwrap();
}
