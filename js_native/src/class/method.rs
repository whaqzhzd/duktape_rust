use super::super::{ctx::DukContext, error::Result};
use crate::privates::DUK_VARARGS;
use dukbind::*;
use std::ffi::{c_void, CString};
use typemap::TypeMap;

pub struct Instance {
    types: TypeMap,
}

impl Instance {
    pub fn new() -> Instance {
        Instance {
            types: TypeMap::new(),
        }
    }

    pub fn data(&self) -> &TypeMap {
        &self.types
    }

    pub fn data_mut(&mut self) -> &mut TypeMap {
        &mut self.types
    }
}

static KEY: &'static [u8] = b"\xFFmethod_ptr";
pub static DATA_KEY: &'static [u8] = b"\xFFdata_ptr";
pub static CTOR_KEY: &'static [u8] = b"\xFFctor_ptr";

pub trait Method {
    fn argc(&self) -> i32 {
        DUK_VARARGS
    }
    fn call(&self, ctx: &DukContext, instance: &mut Instance) -> Result<i32>;
}

impl<T: Fn(&DukContext, &mut Instance) -> Result<i32>> Method for (i32, T) {
    fn argc(&self) -> i32 {
        self.0
    }

    fn call(&self, ctx: &DukContext, instance: &mut Instance) -> Result<i32> {
        self.1(ctx, instance)
    }
}

impl<T: Fn(&DukContext, &mut Instance) -> Result<i32>> Method for T {
    fn argc(&self) -> i32 {
        0
    }

    fn call(&self, ctx: &DukContext, instance: &mut Instance) -> Result<i32> {
        self(ctx, instance)
    }
}

pub(crate) unsafe fn push_method(ctx: &DukContext, method: Box<dyn Method>) {
    duk_push_c_function(ctx.inner, Some(call), method.argc());
    let m = Box::new(method);
    duk_push_pointer(ctx.inner, Box::into_raw(m) as *mut c_void);
    duk_put_prop_lstring(ctx.inner, -2, KEY.as_ptr() as *const i8, KEY.len());
    duk_push_c_function(ctx.inner, Some(dtor), 1);
    duk_set_finalizer(ctx.inner, -2);
}

unsafe extern "C" fn call(ctx: *mut duk_context) -> duk_ret_t {
    duk_push_current_function(ctx);

    // Get Function ptr
    duk_get_prop_lstring(ctx, -1, KEY.as_ptr() as *const i8, KEY.len());
    let mut c = DukContext::with(ctx);
    let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
    let method = Box::from_raw(ptr);
    duk_pop_2(ctx);

    duk_push_this(ctx);
    if duk_has_prop_lstring(ctx, -1, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len()) != 1 {
        // Keep it
        Box::into_raw(method);
        duk_error_raw(
            ctx,
            DUK_ERR_ERROR as i32,
            "".as_ptr() as *const i8,
            0,
            "could find data ptr".as_ptr() as *const i8,
        );
        return 0;
    }

    duk_get_prop_lstring(ctx, -1, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len());
    let ptr = duk_get_pointer(ctx, -1) as *mut Instance;
    let mut pp = Box::from_raw(ptr);
    duk_pop(ctx);

    let ret = match method.call(&mut c, &mut pp) {
        Err(e) => {
            // Keep it
            Box::into_raw(method);
            Box::into_raw(pp);
            duk_error_raw(
                ctx,
                DUK_ERR_ERROR as i32,
                "".as_ptr() as *const i8,
                0,
                CString::new(format!("{}", e.0)).unwrap().as_ptr(),
            );
            return 0;
        }
        Ok(ret) => ret,
    };

    // Keep it
    Box::into_raw(method);
    Box::into_raw(pp);

    return ret;
}

unsafe extern "C" fn dtor(ctx: *mut duk_context) -> duk_ret_t {
    //debug!("method ctor");
    duk_get_prop_lstring(ctx, -1, KEY.as_ptr() as *const i8, KEY.len());
    let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
    duk_pop(ctx);
    duk_del_prop_lstring(ctx, -1, KEY.as_ptr() as *const i8, KEY.len());
    let pp = Box::from_raw(ptr);
    drop(pp);
    return 0;
}
