//!
//! setTimeout
//! setInterval
//! clearTimeout
//! clearInterval
//! requestAnimationFrame
//!

use js_native::prelude::*;
const EVENT_TIMES: &'static str = "eventTimers";

pub fn timer_register(ctx: &DukContext) -> DukResult<()> {
    ctx.push_global_stash();
    ctx.push_object();
    ctx.put_prop_string(-2, EVENT_TIMES);
    ctx.pop(1);

    let mut builder = class::build();
    let global: Object = ctx.push_global_object().getp()?;
    builder.method(
        "loop",
        (2, |ctx: &DukContext, _this: &mut class::Instance| {
            // let func = ctx.get::<Function>(0)?;
            ctx.push_global_stash();
            ctx.get_prop_string(-1, EVENT_TIMES);
            ctx.push_number(1);
            ctx.dup(0);
            ctx.duk_put_prop(-3);
            Ok(1)
        }),
    );

    global.set("Timer", builder);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::DukContext;
    use super::*;
    use crate::init_js_binding;

    #[test]
    fn test() -> DukResult<()> {
        let ctx = DukContext::new().unwrap();
        init_js_binding(&ctx)?;

        ctx.eval(
            r#"
            console = new Console();
            timer = new Timer();
            timer.loop(function(){
                console.log("hehe---")
            });
        "#,
        )?
        .get(-1)?;

        loop {
            ctx.push_global_stash();
            ctx.get_prop_string(-1, EVENT_TIMES);
            ctx.push_number(1);
            ctx.duk_get_prop(-2);
            ctx.call(0)?;
            ctx.pop(1);
            ctx.pop(2);
        }

        Ok(())
    }
}
