//!
//! setTimeout
//! setInterval
//! clearTimeout
//! clearInterval
//! requestAnimationFrame
//!

use js_native::prelude::*;
use std::ptr::null_mut;
use std::time::{Duration, SystemTime};

/// 默认FPS
const FPS: u128 = 60;

///
const EVENT_TIMES: &'static str = "eventTimers";

/// 非线程安全的全局可变变量
///
/// 用来保存定时器
///
/// 切勿在多线程中使用
static mut V: *mut Vec<EvTime> = null_mut::<Vec<EvTime>>();

/// 非线程安全的全局唯一ID
///
/// 用来自增定时器索引
///
/// 切勿在多线程中使用
static mut GID: *mut i32 = null_mut::<i32>();

/// 非线程安全的RAF
///
/// 用来保存唯一requestAnimationFrame实现
///
/// 切勿在多线程中使用
static mut RAF: *mut EvTime = null_mut::<EvTime>();

/// 非线程安全的秒表
///
/// 用来驱动requestAnimationFrame
///
/// 切勿在多线程中使用
static mut STOPWATCH: *mut Stopwatch = null_mut::<Stopwatch>();

/// 秒表
pub struct Stopwatch {
    watch: SystemTime,
    start: Duration,
    delta_time: u128,
    stop: bool,
}

impl Stopwatch {
    pub fn new() -> Self {
        let sys_now = SystemTime::now();
        Self {
            watch: sys_now,
            start: sys_now.elapsed().unwrap(),
            delta_time: 0_u128,
            stop: false,
        }
    }

    pub fn update(&mut self) -> u128 {
        match self.watch.elapsed() {
            Ok(v) => {
                self.delta_time = (v - self.start).as_millis();
                self.delta_time as u128
            }
            Err(_) => {
                let sys_now = SystemTime::now();
                self.watch = sys_now;
                self.start = sys_now.elapsed().unwrap();
                0
            }
        }
    }

    pub fn elapsed(&self) -> u128 {
        return match self.watch.elapsed() {
            Ok(v) => v.as_millis() as u128,
            Err(_) => 0,
        };
    }

    pub fn get_current(&mut self) -> u128 {
        return match self.watch.elapsed() {
            Ok(v) => {
                self.start = v;
                self.start.as_millis()
            }
            Err(_) => 0,
        };
    }

    pub fn get_last(&self) -> u128 {
        self.start.as_millis() as u128
    }

    pub fn is_stop(&self) -> bool {
        self.stop
    }

    pub fn stop(&mut self, stop: bool) {
        self.stop = stop
    }
}

#[derive(Debug, PartialEq)]
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
    dt: u128,
    delete: bool,
}

/// 初始化
fn init_v() {
    let vec = <Vec<EvTime>>::new();
    unsafe {
        if V == null_mut() {
            V = std::mem::transmute(Box::new(vec));
        }
    }

    let gid = 1;
    unsafe {
        if GID == null_mut() {
            GID = std::mem::transmute(Box::new(gid));
        }
    }

    let stop_watch = Stopwatch::new();
    unsafe {
        if STOPWATCH == null_mut() {
            STOPWATCH = std::mem::transmute(Box::new(stop_watch));
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

    register_set_timeout(ctx);
    register_clear_timeout(ctx);
    register_set_interval(ctx);
    register_clear_interval(ctx);
    register_request_animation_frame(ctx);

    Ok(())
}

pub fn enter_frame(ctx: &DukContext) {
    loop {
        unsafe {
            let cb = || {
                let v = &mut *V;
                for t in v.iter_mut() {
                    if t.time.elapsed().unwrap().as_millis() - t.start >= t.delay {
                        if t.timetype == TimeType::TIMEOUT && t.execute_num >= 1 {
                            continue;
                        }

                        ctx.push_global_stash();
                        ctx.get_prop_string(-1, EVENT_TIMES);
                        ctx.push_number(t.id);
                        ctx.duk_get_prop(-2);
                        ctx.call(0).ok();
                        ctx.pop(1);
                        ctx.pop(2);
                        t.execute_num = t.execute_num + 1;

                        t.start = 0;
                        t.time = SystemTime::now();
                    }
                }

                v.retain(|t| !t.delete);
            };

            cb();

            // 优先执行requestAnimationFrame
            if RAF != null_mut() {
                let t = &mut *RAF;
                let frame_time = t.delay;
                let stop_watch = &mut *STOPWATCH;

                if !stop_watch.is_stop() {
                    if stop_watch.elapsed() - stop_watch.get_last() > frame_time {
                        stop_watch.get_current();

                        ctx.push_global_stash();
                        ctx.get_prop_string(-1, EVENT_TIMES);
                        ctx.push_number(t.id);
                        ctx.duk_get_prop(-2);

                        ctx.push_number(t.dt as f64);
                        ctx.call(1).ok();
                        ctx.pop(1);
                        ctx.pop(2);

                        let delta_time = stop_watch.update();
                        t.dt = delta_time;
                    } else {
                        use std::thread;
                        thread::yield_now();
                    }
                } else {
                    break;
                }
            }
        }
    }
}

///
/// 注册setTimeout 函数实现
///
fn register_set_timeout(ctx: &DukContext) {
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
                    dt: 0,
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
}

///
/// 注册clearTimeout 函数实现
///
fn register_clear_timeout(ctx: &DukContext) {
    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            match ctx.get_type(0) {
                Type::Number => {
                    let id = ctx.get::<i32>(0)?;
                    info!("clearTimeout {:?}", id);
                    unsafe {
                        let v = &mut *V;
                        for t in v.iter_mut() {
                            if t.id == id && t.timetype == TimeType::TIMEOUT {
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
}

///
/// 注册setInterval 函数实现
///
fn register_set_interval(ctx: &DukContext) {
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
                    dt: 0,
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
}

///
/// 注册clearInterval 函数实现
///
fn register_clear_interval(ctx: &DukContext) {
    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| {
            match ctx.get_type(0) {
                Type::Number => {
                    let id = ctx.get::<i32>(0)?;
                    unsafe {
                        let v = &mut *V;
                        for t in v.iter_mut() {
                            if t.id == id && t.timetype == TimeType::INTERVAL {
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
}

///
/// 注册requestAnimationFrame
///
fn register_request_animation_frame(ctx: &DukContext) {
    ctx.push_global_object()
        .push_function((2, |ctx: &DukContext| unsafe {
            if RAF != null_mut() {
                error!("requestAnimationFrame只能注册一个回调");
                Ok(0)
            } else {
                ctx.push_global_stash();
                ctx.get_prop_string(-1, EVENT_TIMES);

                let mut delay = 1000 / FPS;
                if ctx.get_type(1) != Type::Undefined {
                    let mut dt = ctx.get_number(1)? as u128;
                    if dt < FPS {
                        dt = FPS
                    };
                    delay = 1000 / dt;
                }

                let id = generator_id();
                let time = EvTime {
                    id: id,
                    start: 0,
                    execute_num: 0,
                    delay: delay,
                    timetype: TimeType::LOOP,
                    time: SystemTime::now(),
                    delete: false,
                    dt: 0,
                };

                RAF = std::mem::transmute(Box::new(time));
                ctx.push_number(id);
                ctx.dup(0);
                ctx.duk_put_prop(-3);

                Ok(0)
            }
        }))
        .put_prop_string(-2, "requestAnimationFrame")
        .pop(1);
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
           
            // setInterval
            // var id = setInterval(function(){
            //     console.log("setInterval---");
            // },10);
            
            // setTimeout
            // var id = setTimeout(function(){
            //     console.log("setTimeout---:");
            //     clearTimeout(id);
            // },21);

            // requestAnimationFrame
            // `dt` - dt is delta time
            requestAnimationFrame(function(dt){
                console.log("requestAnimationFrame:"+dt);
            });
        "#,
        )?
        .get(-1)?;

        enter_frame(&ctx);
        ctx;
        Ok(())
    }
}
