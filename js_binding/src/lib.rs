//!
//! 一部分功能对象的原生实现
//!
//! `polyfills` - 主要是对于浏览器js-api的实现
//! `gfx` - 主要是对于opengl的渲染实现
//!
extern crate js_native;

#[macro_use]
extern crate log;
extern crate env_logger;

use chrono::Local;
use std::io::Write;

mod gfx;
mod polyfills;

pub fn init_js_binding() {
    // 初始化日志
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{}:{}:{}]",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                record.line().unwrap_or(0),
                &record.args()
            )
        })
        .init();
}
