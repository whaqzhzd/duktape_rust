//!
//! 日志实现
//! https://github.com/svaarala/duktape/blob/master/extras/console/duk_console.c
//! rust 版本
//!

use chrono::Local;
use js_native::prelude::*;
use std::io::Write;

pub fn console_register(ctx: &DukContext) -> DukResult<()> {
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

    let mut builder = class::build();
    let global: Object = ctx.push_global_object().getp()?;
    builder
        .method(
            "log",
            (1, |ctx: &DukContext, _this: &mut class::Instance| {
                match ctx.get_type(0) {
                    Type::Object => {
                        let log = ctx.get::<Object>(0)?;
                        info!("obj : {}", log);
                    }
                    Type::String => {
                        let log = ctx.get::<String>(0)?;
                        info!("string : {:?}", log);
                    }
                    Type::Undefined => {
                        info!("undefined");
                    }
                    Type::Number => {
                        let log = ctx.get::<i32>(0)?;
                        info!("number : {:?}", log);
                    }
                    _ => {}
                }
                Ok(1)
            }),
        )
        .method("error", |ctx: &DukContext, _this: &mut class::Instance| {
            let log = ctx.get::<String>(0)?;
            error!("{:?}", log);
            Ok(1)
        });

    global.set("Console", builder);

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
            
            for(var i=0,l=10;i<l;i++){
                console.log("大明在js里面调用了rust。很强");
            }

            console.log(Duktape.version);
        "#,
        )?
        .get(-1)?;

        ctx;

        Ok(())
    }
}
