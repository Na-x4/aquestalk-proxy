// Copyright (c) 2021-2022 Na-x4
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod aquestalk;
pub mod messages;

mod proxy;
pub use proxy::stdio::AquesTalkProxyStdio;
pub use proxy::tcp::AquesTalkProxyTcp;
pub use proxy::AquesTalkProxyClient;
