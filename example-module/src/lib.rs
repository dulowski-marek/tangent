use std::alloc::{alloc as rust_alloc, dealloc as rust_dealloc, Layout};

static mut RESULT_PTR: i32 = 0;
static mut RESULT_LEN: i32 = 0;

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    if size <= 0 {
        return 0;
    }
    unsafe {
        let layout = Layout::from_size_align_unchecked(size as usize, 1);
        rust_alloc(layout) as i32
    }
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: i32, size: i32) {
    if ptr == 0 || size <= 0 {
        return;
    }
    unsafe {
        let layout = Layout::from_size_align_unchecked(size as usize, 1);
        rust_dealloc(ptr as *mut u8, layout);
    }
}

#[no_mangle]
pub extern "C" fn render(config_ptr: i32, config_len: i32) {
    let config_json = unsafe {
        let slice = std::slice::from_raw_parts(config_ptr as *const u8, config_len as usize);
        std::str::from_utf8(slice).unwrap_or("{}")
    };

    let config: serde_json::Value =
        serde_json::from_str(config_json).unwrap_or(serde_json::Value::Null);
    let content = format!("// generated\n// config: {}", config);

    let writables = serde_json::json!([{
        "path": "src",
        "filename": "Generated.ts",
        "content": content,
    }]);

    let result = serde_json::to_string(&writables).unwrap_or_else(|_| "[]".into());
    let bytes = result.into_bytes().into_boxed_slice();
    let len = bytes.len() as i32;
    let ptr = Box::leak(bytes).as_ptr() as i32;

    unsafe {
        RESULT_PTR = ptr;
        RESULT_LEN = len;
    }
}

#[no_mangle]
pub extern "C" fn result_ptr() -> i32 {
    unsafe { RESULT_PTR }
}

#[no_mangle]
pub extern "C" fn result_len() -> i32 {
    unsafe { RESULT_LEN }
}
