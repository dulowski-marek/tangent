use std::ptr::addr_of;

static mut RESULT_BUF: [u8; 65536] = [0u8; 65536];
static mut RESULT_LEN: usize = 0;

#[no_mangle]
pub extern "C" fn render(config_ptr: i32, config_len: i32) {
    let config_json = unsafe {
        let slice = std::slice::from_raw_parts(config_ptr as *const u8, config_len as usize);
        std::str::from_utf8(slice).unwrap_or("{}")
    };

    let config: serde_json::Value = serde_json::from_str(config_json).unwrap_or(serde_json::Value::Null);
    let content = format!("// generated\n// config: {}", config);

    let writables = serde_json::json!([{
        "path": "src",
        "filename": "Generated.ts",
        "content": content,
    }]);

    let result = serde_json::to_string(&writables).unwrap_or_else(|_| "[]".into());
    let bytes = result.as_bytes();

    unsafe {
        let len = bytes.len().min(65536);
        RESULT_BUF[..len].copy_from_slice(&bytes[..len]);
        RESULT_LEN = len;
    }
}

#[no_mangle]
pub extern "C" fn result_ptr() -> i32 {
    #[allow(unused_unsafe)]
    unsafe { addr_of!(RESULT_BUF) as i32 }
}

#[no_mangle]
pub extern "C" fn result_len() -> i32 {
    unsafe { RESULT_LEN as i32 }
}
