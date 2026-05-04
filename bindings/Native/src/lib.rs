//! C ABI entry points for language bindings.
//!
//! The API mirrors the CLI `export-text` and `export-markdown` commands and
//! returns a UTF-8 JSON result string. Call `rhwp_string_free` for every string
//! returned from this module.

use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

use rhwp_core::wasm_api::HwpDocument;

const ALL_PAGES: i32 = -1;

#[no_mangle]
pub extern "C" fn rhwp_export_text(
    input_path: *const c_char,
    output_dir: *const c_char,
    page: i32,
) -> *mut c_char {
    ffi_result(|| {
        let input_path = read_utf8(input_path, "input_path")?;
        let output_dir = read_utf8(output_dir, "output_dir")?;
        export_text_to_dir(
            Path::new(&input_path),
            Path::new(&output_dir),
            normalize_page(page)?,
        )
    })
}

#[no_mangle]
pub extern "C" fn rhwp_export_markdown(
    input_path: *const c_char,
    output_dir: *const c_char,
    page: i32,
) -> *mut c_char {
    ffi_result(|| {
        let input_path = read_utf8(input_path, "input_path")?;
        let output_dir = read_utf8(output_dir, "output_dir")?;
        export_markdown_to_dir(
            Path::new(&input_path),
            Path::new(&output_dir),
            normalize_page(page)?,
        )
    })
}

#[no_mangle]
pub extern "C" fn rhwp_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        drop(CString::from_raw(ptr));
    }
}

fn export_text_to_dir(
    input_path: &Path,
    output_dir: &Path,
    target_page: Option<u32>,
) -> Result<String, String> {
    let data = fs::read(input_path)
        .map_err(|e| format!("파일을 읽을 수 없습니다 - {}: {}", input_path.display(), e))?;
    let doc = HwpDocument::from_bytes(&data).map_err(|e| format!("HWP 파싱 실패 - {}", e))?;
    let page_count = doc.page_count();
    let pages = select_pages(page_count, target_page)?;
    fs::create_dir_all(output_dir).map_err(|e| {
        format!(
            "출력 폴더를 생성할 수 없습니다 - {}: {}",
            output_dir.display(),
            e
        )
    })?;

    let file_stem = file_stem(input_path);
    let mut written = Vec::new();

    for page_num in pages {
        let mut text = doc
            .extract_page_text_native(page_num)
            .map_err(|e| format!("페이지 {} 텍스트 추출 실패 - {:?}", page_num, e))?;
        ensure_trailing_newline(&mut text);

        let output_path = output_dir.join(page_file_name(&file_stem, "txt", page_count, page_num));
        fs::write(&output_path, text.as_bytes())
            .map_err(|e| format!("TXT 저장 실패 - {}: {}", output_path.display(), e))?;
        written.push(output_path);
    }

    Ok(success_json(page_count, &written, None))
}

fn export_markdown_to_dir(
    input_path: &Path,
    output_dir: &Path,
    target_page: Option<u32>,
) -> Result<String, String> {
    let data = fs::read(input_path)
        .map_err(|e| format!("파일을 읽을 수 없습니다 - {}: {}", input_path.display(), e))?;
    let doc = HwpDocument::from_bytes(&data).map_err(|e| format!("HWP 파싱 실패 - {}", e))?;
    let page_count = doc.page_count();
    let pages = select_pages(page_count, target_page)?;
    fs::create_dir_all(output_dir).map_err(|e| {
        format!(
            "출력 폴더를 생성할 수 없습니다 - {}: {}",
            output_dir.display(),
            e
        )
    })?;

    let file_stem = file_stem(input_path);
    let assets_dir_name = format!("{}_assets", file_stem);
    let assets_dir_path = output_dir.join(&assets_dir_name);
    let mut written = Vec::new();
    let mut written_image_count = 0usize;

    for page_num in pages {
        let (mut markdown, image_refs) = doc
            .extract_page_markdown_with_images_native(page_num)
            .map_err(|e| format!("페이지 {} Markdown 생성 실패 - {:?}", page_num, e))?;

        for (img_idx, (sec_idx, para_idx, control_idx, bin_data_id)) in
            image_refs.iter().enumerate()
        {
            let token = format!("[[RHWP_IMAGE:{}]]", img_idx + 1);
            let Some((mime, image_data)) =
                extract_image_data(&doc, *sec_idx, *para_idx, *control_idx, *bin_data_id)?
            else {
                markdown = markdown.replace(&token, "");
                continue;
            };

            fs::create_dir_all(&assets_dir_path).map_err(|e| {
                format!(
                    "이미지 출력 폴더 생성 실패 - {}: {}",
                    assets_dir_path.display(),
                    e
                )
            })?;

            let image_filename = format!(
                "{}_p{:03}_img{:03}.{}",
                file_stem,
                page_num + 1,
                img_idx + 1,
                mime_to_ext(&mime),
            );
            let image_path = assets_dir_path.join(&image_filename);
            fs::write(&image_path, &image_data)
                .map_err(|e| format!("이미지 저장 실패 - {}: {}", image_path.display(), e))?;

            let image_link = format!(
                "![image {}]({}/{})",
                img_idx + 1,
                assets_dir_name,
                image_filename
            );
            markdown = markdown.replace(&token, &image_link);
            written_image_count += 1;
        }

        ensure_trailing_newline(&mut markdown);
        let output_path = output_dir.join(page_file_name(&file_stem, "md", page_count, page_num));
        fs::write(&output_path, markdown.as_bytes())
            .map_err(|e| format!("Markdown 저장 실패 - {}: {}", output_path.display(), e))?;
        written.push(output_path);
    }

    Ok(success_json(
        page_count,
        &written,
        Some(written_image_count),
    ))
}

fn extract_image_data(
    doc: &HwpDocument,
    sec_idx: Option<usize>,
    para_idx: Option<usize>,
    control_idx: Option<usize>,
    bin_data_id: u16,
) -> Result<Option<(String, Vec<u8>)>, String> {
    if let (Some(si), Some(pi), Some(ci)) = (sec_idx, para_idx, control_idx) {
        if let (Ok(mime), Ok(data)) = (
            doc.get_control_image_mime_native(si, pi, ci),
            doc.get_control_image_data_native(si, pi, ci),
        ) {
            return Ok(Some((mime, data)));
        }
    }

    if bin_data_id == 0 {
        return Ok(None);
    }

    let mime = doc
        .get_bin_data_image_mime_native(bin_data_id)
        .map_err(|e| format!("이미지 MIME fallback 실패 (bin={}): {:?}", bin_data_id, e))?;
    let data = doc
        .get_bin_data_image_data_native(bin_data_id)
        .map_err(|e| format!("이미지 데이터 fallback 실패 (bin={}): {:?}", bin_data_id, e))?;
    Ok(Some((mime, data)))
}

fn select_pages(page_count: u32, target_page: Option<u32>) -> Result<Vec<u32>, String> {
    if page_count == 0 {
        return Err("문서에 페이지가 없습니다.".to_string());
    }

    match target_page {
        Some(page) if page >= page_count => Err(format!(
            "페이지 번호가 범위를 벗어났습니다 (0~{})",
            page_count - 1
        )),
        Some(page) => Ok(vec![page]),
        None => Ok((0..page_count).collect()),
    }
}

fn normalize_page(page: i32) -> Result<Option<u32>, String> {
    if page == ALL_PAGES {
        Ok(None)
    } else if page < 0 {
        Err("page는 -1(전체) 또는 0 이상의 페이지 번호여야 합니다.".to_string())
    } else {
        Ok(Some(page as u32))
    }
}

fn read_utf8(ptr: *const c_char, name: &str) -> Result<String, String> {
    if ptr.is_null() {
        return Err(format!("{}가 null입니다.", name));
    }

    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| format!("{}는 유효한 UTF-8 문자열이어야 합니다: {}", name, e))
    }
}

fn ffi_result<F>(f: F) -> *mut c_char
where
    F: FnOnce() -> Result<String, String> + std::panic::UnwindSafe,
{
    let json = match std::panic::catch_unwind(f) {
        Ok(Ok(json)) => json,
        Ok(Err(error)) => error_json(&error),
        Err(_) => error_json("FFI 호출 중 panic이 발생했습니다."),
    };

    CString::new(json)
        .unwrap_or_else(|_| {
            CString::new(error_json("결과 문자열에 NUL 문자가 포함되었습니다.")).unwrap()
        })
        .into_raw()
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page")
        .to_string()
}

fn page_file_name(file_stem: &str, ext: &str, page_count: u32, page_num: u32) -> String {
    if page_count == 1 {
        format!("{}.{}", file_stem, ext)
    } else {
        format!("{}_{:03}.{}", file_stem, page_num + 1, ext)
    }
}

fn ensure_trailing_newline(s: &mut String) {
    if !s.ends_with('\n') {
        s.push('\n');
    }
}

fn mime_to_ext(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/webp" => "webp",
        _ => "bin",
    }
}

fn success_json(page_count: u32, files: &[PathBuf], image_count: Option<usize>) -> String {
    let files_json = files
        .iter()
        .map(|p| format!("\"{}\"", json_escape(&p.display().to_string())))
        .collect::<Vec<_>>()
        .join(",");
    let image_json = image_count
        .map(|count| format!(",\"imageCount\":{}", count))
        .unwrap_or_default();

    format!(
        "{{\"ok\":true,\"pageCount\":{},\"files\":[{}]{}}}",
        page_count, files_json, image_json
    )
}

fn error_json(error: &str) -> String {
    format!("{{\"ok\":false,\"error\":\"{}\"}}", json_escape(error))
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\u{20}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
