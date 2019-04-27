use super::super::types::{Function, ToDuktape};
use super::super::{
    ctx::{DukContext, Idx},
    error::{ErrorKind, Result},
};
use super::method::{push_method, Instance, Method, CTOR_KEY, DATA_KEY};
use crate::privates::DUK_VARARGS;
use dukbind::*;
use std::collections::HashMap;
use std::ffi::c_void;

pub enum Prototype {
    Method(Box<dyn Method>),
}

#[derive(Default)]
pub struct Builder<'a> {
    name: String,
    ctor: Option<Box<dyn Method>>,
    parent: Option<Function<'a>>,
    methods: HashMap<String, Prototype>,
}

impl<'a> Builder<'a> {
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_owned();
        self
    }

    pub fn set(&mut self, name: &str, prop: Prototype) -> &mut Self {
        self.methods.insert(name.to_owned(), prop);
        self
    }

    pub fn method<T: 'static + Method>(&mut self, name: &str, method: T) -> &mut Self {
        let b: Box<dyn Method> = Box::new(method);
        self.methods.insert(name.to_owned(), Prototype::Method(b));
        self
    }

    pub fn constructor<T: 'static + Method>(&mut self, ctor: T) -> &mut Self {
        let b: Box<dyn Method> = Box::new(ctor);
        self.ctor = Some(b);
        self
    }

    pub fn inherit(&mut self, parent: Function<'a>) -> &mut Self {
        self.parent = Some(parent);
        self
    }
}

impl<'a> ToDuktape for Builder<'a> {
    fn to_context(self, ctx: &DukContext) -> Result<()> {
        unsafe { push_class_builder(ctx, self) }
    }
}

pub(crate) unsafe fn push_class_builder(ctx: &DukContext, builder: Builder) -> Result<()> {
    duk_push_c_function(ctx.inner, Some(class_ctor), DUK_VARARGS);

    if !builder.name.is_empty() {
        ctx.push_string("name").push_string(builder.name);

        duk_def_prop(ctx.inner, -3, DUK_DEFPROP_HAVE_VALUE | DUK_DEFPROP_FORCE);
    }

    if let Some(parent) = builder.parent {
        ctx.get_global_string("Object")
            .push_string("create")
            .push(&parent)?
            .get_prop_string(-1, "prototype")
            .remove(-2)
            .call_prop(-3, 1)?
            .remove(-2);
        ctx.dup(1).put_prop_string(-3, "__super__");
    } else {
        ctx.push_object();
    }

    for (name, method) in builder.methods {
        match method {
            Prototype::Method(m) => {
                push_method(ctx, m);
                ctx.put_prop_string(-2, &name);
            }
        }
    }

    ctx.put_prop_string(-2, "prototype");

    if let Some(ctor) = builder.ctor {
        //debug!("push class constructor");
        let b = Box::new(ctor);
        duk_push_pointer(ctx.inner, Box::into_raw(b) as *mut c_void);
        duk_put_prop_lstring(
            ctx.inner,
            -2,
            CTOR_KEY.as_ptr() as *const i8,
            CTOR_KEY.len(),
        );
    }

    duk_push_c_function(ctx.inner, Some(constructor_dtor), 1);
    duk_set_finalizer(ctx.inner, -2);

    Ok(())
}

unsafe extern "C" fn class_ctor(ctx: *mut duk_context) -> duk_ret_t {
    //debug!("class constructor");
    duk_push_current_function(ctx);

    let mut instance = Box::new(Instance::new());
    // duk_dump_context_stdout(ctx);
    if duk_has_prop_lstring(ctx, -1, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len()) == 1 {
        //debug!("found custom class constructor");
        duk_get_prop_lstring(ctx, -1, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len());
        let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
        duk_pop(ctx);

        let ctor = Box::from_raw(ptr);
        let mut c = DukContext::with(ctx);
        match ctor.call(&mut c, &mut instance) {
            Ok(_) => {}
            Err(e) => {
                Box::into_raw(ctor);
                duk_error_raw(
                    ctx,
                    DUK_ERR_ERROR as i32,
                    "".as_ptr() as *const i8,
                    0,
                    format!("ctor call failed: {}", e).as_ptr() as *const i8,
                );
                return 0;
            }
        };

        // We wanna keep the ctor on the heap
        Box::into_raw(ctor);
    }

    duk_push_this(ctx);
    duk_push_pointer(ctx, Box::into_raw(instance) as *mut c_void);
    duk_put_prop_lstring(ctx, -2, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len());
    duk_push_c_function(ctx, Some(class_dtor), 1);
    duk_set_finalizer(ctx, -2);

    return 0;
}

unsafe extern "C" fn constructor_dtor(ctx: *mut duk_context) -> duk_ret_t {
    //debug!("constructor dtor");

    if duk_has_prop_lstring(ctx, 0, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len()) == 1 {
        //debug!("dropping class constructor");
        duk_get_prop_lstring(ctx, 0, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len());
        let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
        Box::from_raw(ptr);
        duk_pop(ctx);
    }

    return 0;
}

unsafe extern "C" fn class_dtor(ctx: *mut duk_context) -> duk_ret_t {
    //debug!("class dtor");
    if duk_has_prop_lstring(ctx, 0, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len()) == 1 {
        //debug!("dropping instance data");
        duk_get_prop_lstring(ctx, 0, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len());
        let ptr = duk_get_pointer(ctx, -1) as *mut Instance;
        Box::from_raw(ptr);
        duk_pop(ctx);
    }
    0
}

pub fn get_instance<Func: FnOnce(&mut Instance) -> Result<T>, T>(
    ctx: &DukContext,
    idx: Idx,
    cb: Func,
) -> Result<T> {
    if ctx.has_prop_string(idx, DATA_KEY) {
        let mut instance = unsafe {
            duk_get_prop_lstring(
                ctx.inner,
                -1,
                DATA_KEY.as_ptr() as *const i8,
                DATA_KEY.len(),
            );
            let ptr = duk_get_pointer(ctx.inner, -1) as *mut Instance;
            let pp = Box::from_raw(ptr);
            duk_pop(ctx.inner);
            pp
        };

        let ret = cb(&mut instance);
        Box::into_raw(instance);
        return ret;
    }
    bail!(ErrorKind::ReferenceError(format!("not a instance")))
}
