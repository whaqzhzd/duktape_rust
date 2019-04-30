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

#[macro_use]
extern crate lazy_static;

use js_native::prelude::DukResult;
use js_native::DukContext;

mod gfx;
mod polyfills;

use polyfills::console::console_register;
use polyfills::timer::timer_register;

pub fn init_js_binding(ctx: &DukContext) -> DukResult<()> {
    // 注册日志模块
    console_register(ctx)?;
    // 注册时间类模块
    timer_register(ctx)?;

    Ok(())
}

pub use polyfills::timer::enter_frame;
