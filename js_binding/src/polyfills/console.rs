//!
//!
//!

use js_native::prelude::*;

pub struct Console;

impl Console {
    pub fn log(ctx: &DukContext, this: &mut class::Instance) {
        if !ctx.is(Type::String, 5) {
            error!("argument should be a string");
            return;
        }
        let args = ctx.get::<String>(0).unwrap();
        info!("{}", args);
    }

    pub fn error(ctx: &DukContext, this: &mut class::Instance) {
        if !ctx.is(Type::String, 5) {
            error!("argument should be a string");
            return;
        }
        let args = ctx.get::<String>(0).unwrap();
        error!("{}", args);
    }
}
