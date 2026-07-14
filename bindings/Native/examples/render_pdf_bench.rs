//! [Task #2267] `rhwp_render_pdf` C ABI 를 Quick Look 확장과 동일한 조건으로 호출해
//! 시간·메모리를 측정한다. 확장 한도는 약 30초 / 80MB 경고 / 120MB 강제 종료다.
//!
//!   /usr/bin/time -l cargo run --release --example render_pdf_bench -- <file> [max_pages] [embed_text]

use std::ffi::{CStr, CString};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: render_pdf_bench <file.hwp|hwpx> [max_pages=1] [embed_text=0]");
        std::process::exit(2);
    }

    let path = CString::new(args[1].clone()).unwrap();
    let max_pages: i32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let embed_text: i32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    let started = std::time::Instant::now();
    let buffer =
        rhwp_native_ffi::rhwp_render_pdf(path.as_ptr(), 0, max_pages, std::ptr::null(), embed_text);
    let elapsed = started.elapsed();

    if !buffer.error.is_null() {
        let msg = unsafe { CStr::from_ptr(buffer.error) }
            .to_string_lossy()
            .into_owned();
        rhwp_native_ffi::rhwp_buffer_free(buffer);
        eprintln!("FAIL: {}", msg);
        std::process::exit(1);
    }

    let bytes = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) };
    let valid = bytes.starts_with(b"%PDF-");
    println!(
        "pdf={:.2}MB valid={} elapsed={:.2}s",
        buffer.len as f64 / 1048576.0,
        valid,
        elapsed.as_secs_f64()
    );

    rhwp_native_ffi::rhwp_buffer_free(buffer);
    if !valid {
        std::process::exit(1);
    }
}
