//!
//! setTimeout
//! setInterval
//! clearTimeout
//! clearInterval
//! requestAnimationFrame
//!

use js_native::prelude::*;
use std::sync::Mutex;
use std::time::SystemTime;

const FPS: u32 = 60;
const EVENT_TIMES: &'static str = "eventTimers";

lazy_static! {
    pub static ref V: Mutex<Vec<EvTime>> = Mutex::new(Vec::new());
    pub static ref GID: Mutex<i32> = Mutex::new(1);
}

#[derive(Debug)]
enum TimeType {
    LOOP,
    TIMEOUT,
    INTERVAL,
}

#[derive(Debug)]
pub struct EvTime {
    id: i32,
    delay: u128,
    timetype: TimeType,
    start: u128,
    execute_num: u32,
    time: SystemTime,
}

///
/// 注册Timer类到js虚拟机中
///
/// Timer 类主要负责时间调度相关
/// 实现了
/// setTimeout
/// setInterval
/// clearTimeout
/// clearInterval
/// requestAnimationFrame
///
/// ## Example
///
/// 一毫秒回调
///
/// var id = setTimeout(function(){},1);
///
/// 清除回调
///
/// clearTimeout(id);
///
/// `ctx` - DukContext
///
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
            ctx.push_global_stash();
            ctx.get_prop_string(-1, EVENT_TIMES);
            let mut v = V.lock().unwrap();
            let id = *GID.lock().unwrap();
            let time = EvTime {
                id: id,
                delay: ctx.get_number(1)? as u128,
                start: 0,
                execute_num: 0,
                timetype: TimeType::LOOP,
                time: SystemTime::now(),
            };
            ctx.push_number(id);
            v.push(time);
            ctx.dup(0);
            ctx.duk_put_prop(-3);

            *GID.lock().unwrap() = id + 1;
            Ok(1)
        }),
    );

    global.set("Timer", builder);

    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            ctx.push_global_stash();
            ctx.get_prop_string(-1, EVENT_TIMES);
            let mut v = V.lock().unwrap();
            let id = *GID.lock().unwrap();
            let time = EvTime {
                id: id,
                start: 0,
                execute_num: 0,
                delay: ctx.get_number(1)? as u128,
                timetype: TimeType::TIMEOUT,
                time: SystemTime::now(),
            };
            ctx.push_number(id);
            v.push(time);
            ctx.dup(0);
            ctx.duk_put_prop(-3);
            *GID.lock().unwrap() = id + 1;

            ctx.push_number(id);
            info!("setTimeout {}", id);
            Ok(1)
        }))
        .put_prop_string(-2, "setTimeout")
        .pop(1);

    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            match ctx.get_type(0) {
                Type::Number => {
                    let id = ctx.get::<i32>(0)?;
                    let mut v = V.lock().unwrap();
                    v.retain(|t| t.id != id);

                    info!("clearTimeout {:?}", v);
                }
                _ => {
                    error!("clearTimeout args must be number");
                }
            }

            Ok(1)
        }))
        .put_prop_string(-2, "clearTimeout")
        .pop(1);

    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            ctx.push_global_stash();
            ctx.get_prop_string(-1, EVENT_TIMES);
            let mut v = V.lock().unwrap();
            let id = *GID.lock().unwrap();
            let time = EvTime {
                id: id,
                start: 0,
                execute_num: 0,
                delay: ctx.get_number(1)? as u128,
                timetype: TimeType::INTERVAL,
                time: SystemTime::now(),
            };
            ctx.push_number(id);
            v.push(time);
            ctx.dup(0);
            ctx.duk_put_prop(-3);
            *GID.lock().unwrap() = id + 1;
            info!("setInterval {}", id);

            ctx.push_number(id);
            Ok(1)
        }))
        .put_prop_string(-2, "setInterval")
        .pop(1);

    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            match ctx.get_type(0) {
                Type::Number => {
                    let id = ctx.get::<i32>(0)?;
                    let mut v = V.lock().unwrap();
                    v.retain(|t| t.id != id);

                    info!("clearInterval {:#?}", v);
                }
                _ => {
                    error!("clearInterval args must be number");
                }
            }

            Ok(1)
        }))
        .put_prop_string(-2, "clearInterval")
        .pop(1);

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
           

            var id = setInterval(function(){
                // console.log("setInterval---");
            },10);

            console.log("setInterval id:"+id);

            
            setTimeout(function(){
                console.log("setTimeout---:");
                // console.log(id);
                clearInterval(id);
            },20);

            
            clearInterval(id);
        "#,
        )?
        .get(-1)?;

        loop {
            let mut v = V.lock().unwrap();

            v.retain(|t| {
                if t.time.elapsed().unwrap().as_millis() - t.start >= t.delay {
                    match t.timetype {
                        TimeType::TIMEOUT => {
                            if t.execute_num >= 1 {
                                return true;
                            } else {
                                return true;
                            }
                        }
                        TimeType::LOOP | TimeType::INTERVAL => {}
                    };

                    ctx.push_global_stash();
                    ctx.get_prop_string(-1, EVENT_TIMES);
                    ctx.push_number(t.id);
                    ctx.duk_get_prop(-2);
                    ctx.call(0);
                    ctx.pop(1);
                    ctx.pop(2);

                    info!("{:?}", t.timetype);
                }
                true
            });
        }

        ctx;

        Ok(())
    }
}
