use dukbind::*;
use std::ffi::c_void;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use typemap::TypeMap;
#[allow(dead_code)]
static REF_KEY: &'static [u8] = b"refs";
#[allow(dead_code)]
static DATA_KEY: &'static [u8] = b"data";
#[allow(dead_code)]
pub static DUK_VARARGS: duk_int_t = -1;

#[allow(dead_code)]
unsafe extern "C" fn data_dtor(ctx: *mut duk_context) -> duk_ret_t {
    if duk_has_prop_lstring(ctx, 0, "ptr".as_ptr() as *const i8, 3) == 1 {
        duk_get_prop_lstring(ctx, 0, "ptr".as_ptr() as *const i8, 3);
        let ptr = duk_get_pointer(ctx, -1) as *mut TypeMap;
        Box::from_raw(ptr);
    }
    return 0;
}

#[allow(dead_code)]
pub unsafe fn init_data(ctx: *mut duk_context) {
    duk_push_global_stash(ctx);
    if duk_has_prop_lstring(ctx, -1, DATA_KEY.as_ptr() as *const i8, 4) == 1 {
        duk_pop(ctx);
        return;
    }
    duk_push_bare_object(ctx);
    let b = Box::new(TypeMap::new());
    duk_push_pointer(ctx, Box::into_raw(b) as *mut c_void);
    duk_put_prop_lstring(ctx, -2, "ptr".as_ptr() as *const i8, 3);
    duk_push_c_function(ctx, Some(data_dtor), 1);
    duk_set_finalizer(ctx, -2);
    duk_put_prop_lstring(ctx, -2, DATA_KEY.as_ptr() as *const i8, 4);

    duk_pop(ctx);
}

#[allow(dead_code)]
pub unsafe fn get_data(ctx: *mut duk_context) -> *mut TypeMap {
    duk_push_global_stash(ctx);
    if duk_has_prop_lstring(ctx, -1, DATA_KEY.as_ptr() as *const i8, 4) != 1 {
        duk_pop(ctx);
        panic!("not initialized");
    }

    duk_get_prop_lstring(ctx, -1, DATA_KEY.as_ptr() as *const i8, 4);
    duk_get_prop_lstring(ctx, -1, "ptr".as_ptr() as *const i8, 3);
    let ptr = duk_get_pointer(ctx, -1) as *mut TypeMap;

    duk_pop_n(ctx, 3);
    ptr
}

#[allow(dead_code)]
pub unsafe fn init_refs(ctx: *mut duk_context) {
    duk_push_global_stash(ctx);
    if duk_has_prop_lstring(ctx, -1, REF_KEY.as_ptr() as *const i8, 4) == 1 {
        duk_pop(ctx);
        return;
    }
    duk_push_array(ctx);
    duk_push_int(ctx, 0);
    duk_put_prop_index(ctx, -2, 0);
    duk_put_prop_lstring(ctx, -2, REF_KEY.as_ptr() as *const i8, 4);
    duk_pop(ctx);
}

#[allow(dead_code)]
pub unsafe fn get_refs(ctx: *mut duk_context) -> bool {
    duk_push_global_stash(ctx);
    if duk_has_prop_lstring(ctx, -1, REF_KEY.as_ptr() as *const i8, 4) == 0 {
        duk_pop(ctx);
        return false;
    }

    duk_get_prop_lstring(ctx, -1, REF_KEY.as_ptr() as *const i8, 4);

    duk_remove(ctx, -2);

    true
}

#[allow(dead_code)]
pub unsafe fn make_ref(ctx: *mut duk_context) -> u32 {
    if duk_is_undefined(ctx, -1) == 1 {
        duk_pop(ctx);
        return 0;
    }
    // Get the "refs" array in the heap stash
    if !get_refs(ctx) {
        return 0;
    }

    // ref = refs[0]
    duk_get_prop_index(ctx, -1, 0);
    let mut ret = duk_get_int(ctx, -1);
    duk_pop(ctx);

    // If there was a free slot, remove it from the list
    if ret != 0 {
        // refs[0] = refs[ref]
        duk_get_prop_index(ctx, -1, ret as u32);
        duk_put_prop_index(ctx, -2, 0);
    }
    // Otherwise use the end of the list
    else {
        // ref = refs.length;
        ret = duk_get_length(ctx, -1) as i32;
    }

    // swap the array and the user value in the stack
    duk_insert(ctx, -2);

    // refs[ref] = value
    duk_put_prop_index(ctx, -2, ret as u32);

    // Remove the refs array from the stack.
    duk_pop(ctx);

    return ret as u32;
}

#[allow(dead_code)]
pub unsafe fn push_ref(ctx: *mut duk_context, refer: u32) {
    if refer == 0 {
        duk_push_undefined(ctx);
        return;
    }
    // Get the "refs" array in the heap stash
    if !get_refs(ctx) {
        return;
    }

    duk_get_prop_index(ctx, -1, refer);

    duk_remove(ctx, -2);
}

#[allow(dead_code)]
pub unsafe fn unref(ctx: *mut duk_context, refer: u32) {
    if refer == 0 {
        return;
    }
    // Get the "refs" array in the heap stash
    if !get_refs(ctx) {
        return;
    }

    // Insert a new link in the freelist

    // refs[ref] = refs[0]
    duk_get_prop_index(ctx, -1, 0);
    duk_put_prop_index(ctx, -2, refer);
    // refs[0] = ref
    duk_push_int(ctx, refer as i32);
    duk_put_prop_index(ctx, -2, 0);

    duk_pop(ctx);
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_create_heap_default() -> *mut duk_context {
    duk_create_heap(None, None, None, ptr::null_mut(), None)
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_safe_to_string(ctx: *mut duk_context, index: duk_idx_t) -> *const c_char {
    duk_safe_to_lstring(ctx, index, ptr::null_mut())
}

/* PLAIN */
#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_eval(ctx: *mut duk_context) {
    duk_eval_raw(
        ctx,
        ptr::null(),
        0,
        1 | DUK_COMPILE_EVAL | DUK_COMPILE_NOFILENAME,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_eval_noresult(ctx: *mut duk_context) {
    duk_eval_raw(
        ctx,
        ptr::null(),
        0,
        1 | DUK_COMPILE_EVAL | DUK_COMPILE_NORESULT | DUK_COMPILE_NOFILENAME,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_peval(ctx: *mut duk_context) -> duk_int_t {
    duk_eval_raw(
        ctx,
        ptr::null(),
        0,
        1 | DUK_COMPILE_EVAL | DUK_COMPILE_SAFE | DUK_COMPILE_NOFILENAME,
    )
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_peval_noresult(ctx: *mut duk_context) -> duk_int_t {
    duk_eval_raw(
        ctx,
        ptr::null(),
        0,
        1 | DUK_COMPILE_EVAL | DUK_COMPILE_SAFE | DUK_COMPILE_NORESULT | DUK_COMPILE_NOFILENAME,
    )
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_compile(ctx: *mut duk_context, flags: duk_uint_t) {
    duk_compile_raw(ctx, ptr::null(), 0, 2 | flags);
}

#[inline(always)]
pub unsafe fn duk_pcompile(ctx: *mut duk_context, flags: duk_uint_t) -> duk_int_t {
    duk_compile_raw(ctx, ptr::null(), 0, 2 | flags | DUK_COMPILE_SAFE)
}

/* STRING */

#[allow(dead_code)]
pub unsafe fn duk_eval_string(ctx: *mut duk_context, src: *const c_char) {
    duk_eval_raw(
        ctx,
        src,
        0,
        0 | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN | DUK_COMPILE_NOFILENAME,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_eval_string_noresult(ctx: *mut duk_context, src: *const c_char) {
    duk_eval_raw(
        ctx,
        src,
        0,
        0 | DUK_COMPILE_EVAL
            | DUK_COMPILE_NOSOURCE
            | DUK_COMPILE_STRLEN
            | DUK_COMPILE_NORESULT
            | DUK_COMPILE_NOFILENAME,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_peval_string_noresult(ctx: *mut duk_context, src: *const c_char) -> duk_int_t {
    duk_eval_raw(
        ctx,
        src,
        0,
        0 | DUK_COMPILE_EVAL
            | DUK_COMPILE_SAFE
            | DUK_COMPILE_NOSOURCE
            | DUK_COMPILE_STRLEN
            | DUK_COMPILE_NORESULT
            | DUK_COMPILE_NOFILENAME,
    )
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_compile_string(ctx: *mut duk_context, flags: duk_uint_t, src: *const c_char) {
    duk_compile_raw(
        ctx,
        src,
        0,
        0 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN | DUK_COMPILE_NOFILENAME,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_compile_string_filename(
    ctx: *mut duk_context,
    flags: duk_uint_t,
    src: *const c_char,
) {
    duk_compile_raw(
        ctx,
        src,
        0,
        1 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_pcompile_string(
    ctx: *mut duk_context,
    flags: duk_uint_t,
    src: *const c_char,
) -> duk_int_t {
    duk_compile_raw(
        ctx,
        src,
        0,
        0 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN | DUK_COMPILE_NOFILENAME,
    )
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_pcompile_string_filename(
    ctx: *mut duk_context,
    flags: duk_uint_t,
    src: *const c_char,
) -> duk_int_t {
    duk_compile_raw(
        ctx,
        src,
        0,
        1 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN,
    )
}

/* LSTRING */
#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_eval_lstring(ctx: *mut duk_context, buf: *const c_char, len: duk_size_t) {
    duk_eval_raw(
        ctx,
        buf,
        len,
        0 | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NOFILENAME,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_eval_lstring_noresult(
    ctx: *mut duk_context,
    buf: *const c_char,
    len: duk_size_t,
) {
    duk_eval_raw(
        ctx,
        buf,
        len,
        0 | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NORESULT | DUK_COMPILE_NOFILENAME,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_peval_lstring(
    ctx: *mut duk_context,
    buf: *const c_char,
    len: duk_size_t,
) -> duk_int_t {
    duk_eval_raw(
        ctx,
        buf,
        len,
        0 | DUK_COMPILE_SAFE | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NOFILENAME,
    )
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_peval_lstring_noresult(
    ctx: *mut duk_context,
    buf: *const c_char,
    len: duk_size_t,
) -> duk_int_t {
    duk_eval_raw(
        ctx,
        buf,
        len,
        0 | DUK_COMPILE_SAFE
            | DUK_COMPILE_EVAL
            | DUK_COMPILE_NOSOURCE
            | DUK_COMPILE_NORESULT
            | DUK_COMPILE_NOFILENAME,
    )
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_compile_lstring(
    ctx: *mut duk_context,
    flags: duk_uint_t,
    buf: *const c_char,
    len: duk_size_t,
) {
    duk_compile_raw(
        ctx,
        buf,
        len,
        0 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NOFILENAME,
    );
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_compile_lstring_filename(
    ctx: *mut duk_context,
    flags: duk_uint_t,
    buf: *const c_char,
    len: duk_size_t,
) {
    duk_compile_raw(ctx, buf, len, 1 | flags | DUK_COMPILE_NOSOURCE);
}

#[inline(always)]
pub unsafe fn duk_pcompile_lstring(
    ctx: *mut duk_context,
    flags: duk_uint_t,
    buf: *const c_char,
    len: duk_size_t,
) -> duk_int_t {
    duk_compile_raw(
        ctx,
        buf,
        len,
        0 | flags | DUK_COMPILE_SAFE | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NOFILENAME,
    )
}

#[inline(always)]
pub unsafe fn duk_pcompile_lstring_filename(
    ctx: *mut duk_context,
    flags: duk_uint_t,
    buf: *const c_char,
    len: duk_size_t,
) -> duk_int_t {
    duk_compile_raw(
        ctx,
        buf,
        len,
        1 | flags | DUK_COMPILE_SAFE | DUK_COMPILE_NOSOURCE,
    )
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn duk_dump_context_stdout(ctx: *mut duk_context) {
    duk_push_context_dump(ctx);
    let ostr = duk_get_string(ctx, -1);
    let s = CStr::from_ptr(ostr).to_str().unwrap().to_string();
    duk_pop(ctx);
    println!("{}", s);
}

#[inline(always)]
pub unsafe fn duk_push_fixed_buffer(
    ctx: *mut duk_context,
    size: usize,
) -> *mut ::std::os::raw::c_void {
    duk_push_buffer_raw(ctx, size, 0)
}
