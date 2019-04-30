//!
//! setTimeout
//! setInterval
//! clearTimeout
//! clearInterval
//! requestAnimationFrame
//!

use js_native::prelude::*;
use std::time::SystemTime;

const FPS: u32 = 60;
const EVENT_TIMES: &'static str = "eventTimers";

lazy_static! {
    // pub static ref GID: Mutex<i32> = Mutex::new(1);
}

/// 非线程安全的全局可变变量
///
/// 用来保存定时器
///
/// 切勿在多线程中使用
static mut V: *mut Vec<EvTime> = 0 as *mut Vec<EvTime>;

/// 非线程安全的全局唯一ID
///
/// 用来自增定时器索引
///
/// 切勿在多线程中使用
static mut GID: *mut i32 = 0 as *mut i32;

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
    delete: bool,
}

/// 初始化
fn init_v() {
    let vec = <Vec<EvTime>>::new();
    unsafe {
        if V == 0 as *mut Vec<EvTime> {
            V = std::mem::transmute(Box::new(vec));
        }
    }

    let gid = 1;
    unsafe {
        if GID == 0 as *mut i32 {
            GID = std::mem::transmute(Box::new(gid));
        }
    }
}

/// id 生成器
fn generator_id() -> i32 {
    unsafe {
        let id = *GID;
        if id >= i32::max_value() {
            *GID = 0;
        } else {
            *GID = id + 1;
        }

        id
    }
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
    init_v();

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
            unsafe {
                let v = &mut *V;
                let id = generator_id();
                let time = EvTime {
                    id: id,
                    delay: ctx.get_number(1)? as u128,
                    start: 0,
                    execute_num: 0,
                    timetype: TimeType::LOOP,
                    time: SystemTime::now(),
                    delete: false,
                };
                ctx.push_number(id);
                v.push(time);
                ctx.dup(0);
                ctx.duk_put_prop(-3);
                Ok(1)
            }
        }),
    );

    global.set("Timer", builder);

    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            ctx.push_global_stash();
            ctx.get_prop_string(-1, EVENT_TIMES);
            unsafe {
                let v = &mut *V;
                let id = generator_id();
                let time = EvTime {
                    id: id,
                    start: 0,
                    execute_num: 0,
                    delay: ctx.get_number(1)? as u128,
                    timetype: TimeType::TIMEOUT,
                    time: SystemTime::now(),
                    delete: false,
                };
                ctx.push_number(id);
                v.push(time);
                ctx.dup(0);
                ctx.duk_put_prop(-3);

                ctx.push_number(id);
                Ok(1)
            }
        }))
        .put_prop_string(-2, "setTimeout")
        .pop(1);

    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            match ctx.get_type(0) {
                Type::Number => {
                    let id = ctx.get::<i32>(0)?;
                    unsafe {
                        let v = &mut *V;
                        for t in v.iter_mut() {
                            if t.id == id {
                                t.delete = true;
                            }
                        }
                    }
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
            unsafe {
                let v = &mut *V;
                let id = generator_id();
                let time = EvTime {
                    id: id,
                    start: 0,
                    execute_num: 0,
                    delay: ctx.get_number(1)? as u128,
                    timetype: TimeType::INTERVAL,
                    time: SystemTime::now(),
                    delete: false,
                };
                ctx.push_number(id);
                v.push(time);
                ctx.dup(0);
                ctx.duk_put_prop(-3);

                ctx.push_number(id);
                Ok(1)
            }
        }))
        .put_prop_string(-2, "setInterval")
        .pop(1);

    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            match ctx.get_type(0) {
                Type::Number => {
                    let id = ctx.get::<i32>(0)?;
                    unsafe {
                        let v = &mut *V;
                        for t in v.iter_mut() {
                            if t.id == id {
                                t.delete = true;
                            }
                        }
                    }
                }
                _ => {
                    error!("clearInterval args must be number");
                }
            }

            Ok(0)
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

    use std::collections::HashSet;
    static mut HASH_MAP: *mut HashSet<String> = 0 as *mut HashSet<String>;

    #[test]
    fn ptr_test() {
        let map: HashSet<String> = HashSet::new();
        let set = unsafe {
            if HASH_MAP == 0 as *mut HashSet<String> {
                HASH_MAP = std::mem::transmute(Box::new(map));
            }
            &mut *HASH_MAP
        };

        set.insert("s".to_string());

        unsafe {
            let c = HASH_MAP.as_ref().unwrap();
            assert_eq!(set, c);
        }

        unsafe {
            let c = HASH_MAP.as_mut().unwrap();
            c.insert("s".to_string());
        }

        unsafe {
            let c = HASH_MAP.as_ref().unwrap();
            assert_eq!(set, c);
        }
    }

    #[test]
    fn test() -> DukResult<()> {
        let ctx = DukContext::new().unwrap();
        init_js_binding(&ctx)?;

        ctx.eval(
            r#"
            console = new Console();
           

            var id = setInterval(function(){
                console.log("setInterval---");
            },10);
            
            setTimeout(function(){
                console.log("setTimeout---:");
                
                // clearInterval(id);
                clearTimeout(id);
            },21);
        "#,
        )?
        .get(-1)?;

        loop {
            unsafe {
                let v = &mut *V;
                for t in v.iter_mut() {
                    if t.time.elapsed().unwrap().as_millis() - t.start >= t.delay {
                        match t.timetype {
                            TimeType::TIMEOUT => {
                                if t.execute_num >= 1 {
                                    continue;
                                }
                            }
                            _ => {}
                        }

                        ctx.push_global_stash();
                        ctx.get_prop_string(-1, EVENT_TIMES);
                        ctx.push_number(t.id);
                        ctx.duk_get_prop(-2);
                        ctx.call(0);
                        ctx.pop(1);
                        ctx.pop(2);
                        t.execute_num = t.execute_num + 1;

                        t.start = 0;
                        t.time = SystemTime::now();
                    }
                }

                v.retain(|t| !t.delete);
            }
        }

        ctx;

        Ok(())
    }
}
