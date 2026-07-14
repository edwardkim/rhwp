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
pub extern "C" fn rhwp_read_text(input_path: *const c_char, page: i32) -> *mut c_char {
    ffi_result(|| {
        let input_path = read_utf8(input_path, "input_path")?;
        read_text(Path::new(&input_path), normalize_page(page)?)
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

/// 바이너리 결과 버퍼 (PDF 등).
///
/// [Task #2267] 기존 FFI 는 UTF-8 JSON 문자열만 반환했으나, PDF 는 바이너리이므로
/// 포인터 + 길이 규약이 필요하다. `data` 가 null 이면 실패이며 `error` 에 사유가 담긴다.
/// 성공/실패와 무관하게 **반드시 `rhwp_buffer_free` 로 해제**해야 한다.
#[repr(C)]
pub struct RhwpBuffer {
    /// 바이트 포인터. 실패 시 null.
    pub data: *mut u8,
    /// 바이트 길이. 실패 시 0.
    pub len: usize,
    /// 실패 사유 (UTF-8 C 문자열). 성공 시 null.
    pub error: *mut c_char,
}

impl RhwpBuffer {
    fn ok(mut bytes: Vec<u8>) -> Self {
        bytes.shrink_to_fit();
        let len = bytes.len();
        let data = bytes.as_mut_ptr();
        std::mem::forget(bytes);
        RhwpBuffer {
            data,
            len,
            error: std::ptr::null_mut(),
        }
    }

    fn err(message: &str) -> Self {
        let error = CString::new(message)
            .unwrap_or_else(|_| CString::new("알 수 없는 오류").unwrap())
            .into_raw();
        RhwpBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            error,
        }
    }
}

/// `RhwpBuffer` 를 해제한다. 성공/실패 모두 호출해야 한다. 이중 해제 금지.
#[no_mangle]
pub extern "C" fn rhwp_buffer_free(buffer: RhwpBuffer) {
    if !buffer.data.is_null() && buffer.len > 0 {
        unsafe {
            drop(Vec::from_raw_parts(buffer.data, buffer.len, buffer.len));
        }
    }
    if !buffer.error.is_null() {
        unsafe {
            drop(CString::from_raw(buffer.error));
        }
    }
}

/// 문서의 페이지 수를 반환한다. 실패 시 -1.
#[no_mangle]
pub extern "C" fn rhwp_page_count(input_path: *const c_char) -> i32 {
    let result = std::panic::catch_unwind(|| -> Result<i32, String> {
        let input_path = read_utf8(input_path, "input_path")?;
        let data = fs::read(&input_path)
            .map_err(|e| format!("파일을 읽을 수 없습니다 - {}: {}", input_path, e))?;
        let doc = HwpDocument::from_bytes(&data).map_err(|e| format!("HWP 파싱 실패 - {}", e))?;
        Ok(doc.page_count() as i32)
    });

    match result {
        Ok(Ok(n)) => n,
        _ => -1,
    }
}

/// 문서를 PDF 로 렌더링해 바이트 버퍼로 반환한다.
///
/// [Task #2267] macOS Quick Look 확장이 쓰는 진입점.
///
/// - `first_page`: 0-based 시작 페이지
/// - `max_pages`: 렌더할 최대 페이지 수. 0 이하면 문서 끝까지.
///   **확장은 메모리·시간 한도가 있으므로 반드시 제한을 건다** (썸네일 1, 미리보기 소수).
/// - `font_dir`: 폰트 탐색 절대경로. null 이면 지정하지 않음.
///   코어의 기본 폰트 탐색은 **작업디렉터리 상대경로**(`ttfs` 등)라 샌드박스된 확장에서는
///   잡히지 않는다. 호출자가 번들 Resources 의 절대경로를 넘겨야 한다.
/// - `embed_text`: 0 이면 글리프를 path 로 변환한다. 폰트 서브셋 경로를 건너뛰어
///   메모리를 크게 줄이는 대신 PDF 의 텍스트 선택·검색을 잃는다 (#2264).
///
/// 반환된 버퍼는 반드시 `rhwp_buffer_free` 로 해제한다.
#[no_mangle]
pub extern "C" fn rhwp_render_pdf(
    input_path: *const c_char,
    first_page: u32,
    max_pages: i32,
    font_dir: *const c_char,
    embed_text: i32,
) -> RhwpBuffer {
    let result = std::panic::catch_unwind(|| -> Result<Vec<u8>, String> {
        let input_path = read_utf8(input_path, "input_path")?;
        let font_dir = if font_dir.is_null() {
            None
        } else {
            Some(read_utf8(font_dir, "font_dir")?)
        };

        let data = fs::read(&input_path)
            .map_err(|e| format!("파일을 읽을 수 없습니다 - {}: {}", input_path, e))?;
        let doc = HwpDocument::from_bytes(&data).map_err(|e| format!("HWP 파싱 실패 - {}", e))?;

        let page_count = doc.page_count();
        if page_count == 0 {
            return Err("페이지가 없습니다.".to_string());
        }
        if first_page >= page_count {
            return Err(format!(
                "first_page가 범위를 벗어났습니다 (0~{}): {}",
                page_count - 1,
                first_page
            ));
        }

        let end = if max_pages <= 0 {
            page_count
        } else {
            page_count.min(first_page.saturating_add(max_pages as u32))
        };
        let pages: Vec<u32> = (first_page..end).collect();

        let mut options = rhwp_core::renderer::pdf::PdfExportOptions {
            embed_text: embed_text != 0,
            ..Default::default()
        };
        if let Some(dir) = font_dir {
            options.font_paths.push(PathBuf::from(dir));
        }

        doc.render_pages_pdf_native_with_options(&pages, &options)
            .map_err(|e| format!("PDF 렌더링 실패 - {:?}", e))
    });

    match result {
        Ok(Ok(bytes)) => RhwpBuffer::ok(bytes),
        Ok(Err(error)) => RhwpBuffer::err(&error),
        Err(_) => RhwpBuffer::err("FFI 호출 중 panic이 발생했습니다."),
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

fn read_text(input_path: &Path, target_page: Option<u32>) -> Result<String, String> {
    let data = fs::read(input_path)
        .map_err(|e| format!("파일을 읽을 수 없습니다 - {}: {}", input_path.display(), e))?;
    let doc = HwpDocument::from_bytes(&data).map_err(|e| format!("HWP 파싱 실패 - {}", e))?;
    let page_count = doc.page_count();
    let pages = select_pages(page_count, target_page)?;

    let mut extracted = Vec::new();
    for page_num in pages {
        let mut text = doc
            .extract_page_text_native(page_num)
            .map_err(|e| format!("페이지 {} 텍스트 추출 실패 - {:?}", page_num, e))?;
        ensure_trailing_newline(&mut text);
        extracted.push((page_num, text));
    }

    Ok(text_json(page_count, &extracted))
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
        // [Task #2267] 코어가 cell_path 인자를 추가(Task #1161)했으나 본 FFI 크레이트는
        // 워크스페이스 밖이라 CI 가 컴파일하지 않아 갱신이 누락되어 있었다.
        // 본문 문단이므로 빈 경로를 넘긴다 (셀/글상자 내부가 아님).
        const BODY_PARA: &[(usize, usize, usize)] = &[];
        if let (Ok(mime), Ok(data)) = (
            doc.get_control_image_mime_native(si, pi, BODY_PARA, ci),
            doc.get_control_image_data_native(si, pi, BODY_PARA, ci),
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

fn text_json(page_count: u32, pages: &[(u32, String)]) -> String {
    let pages_json = pages
        .iter()
        .map(|(index, text)| format!("{{\"index\":{},\"text\":\"{}\"}}", index, json_escape(text)))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"ok\":true,\"pageCount\":{},\"pages\":[{}]}}",
        page_count, pages_json
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 저장소 루트 기준 샘플 경로. 이 크레이트는 `bindings/Native` 에 있다.
    fn sample(rel: &str) -> Option<CString> {
        let path = Path::new("../../").join(rel);
        if !path.exists() {
            eprintln!("샘플 없음, 건너뜀: {}", path.display());
            return None;
        }
        Some(CString::new(path.to_string_lossy().as_ref()).unwrap())
    }

    #[test]
    fn page_count_reads_document() {
        let Some(path) = sample("samples/aift.hwp") else {
            return;
        };
        let count = rhwp_page_count(path.as_ptr());
        assert!(count > 0, "페이지 수가 양수여야 한다: {}", count);
    }

    #[test]
    fn page_count_rejects_missing_file() {
        let path = CString::new("존재하지_않는_파일.hwp").unwrap();
        assert_eq!(rhwp_page_count(path.as_ptr()), -1);
    }

    /// [Task #2267] Quick Look 확장이 타는 경로 그대로: 1페이지 PDF 를 렌더한다.
    #[test]
    fn render_pdf_produces_valid_single_page_pdf() {
        let Some(path) = sample("samples/aift.hwp") else {
            return;
        };

        let buffer = rhwp_render_pdf(path.as_ptr(), 0, 1, std::ptr::null(), 0);
        assert!(buffer.error.is_null(), "렌더 실패");
        assert!(!buffer.data.is_null() && buffer.len > 0, "빈 버퍼");

        let bytes = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) };
        assert!(bytes.starts_with(b"%PDF-"), "PDF 매직이 아니다");
        assert!(
            bytes.windows(5).rev().take(64).any(|w| w == b"%%EOF"),
            "PDF 트레일러가 없다"
        );

        rhwp_buffer_free(buffer);
    }

    #[test]
    fn render_pdf_rejects_out_of_range_page() {
        let Some(path) = sample("samples/aift.hwp") else {
            return;
        };

        let buffer = rhwp_render_pdf(path.as_ptr(), 100_000, 1, std::ptr::null(), 0);
        assert!(buffer.data.is_null(), "범위 밖 페이지는 실패해야 한다");
        assert!(!buffer.error.is_null(), "실패 사유가 있어야 한다");
        rhwp_buffer_free(buffer);
    }

    #[test]
    fn render_pdf_reports_error_for_missing_file() {
        let path = CString::new("존재하지_않는_파일.hwp").unwrap();
        let buffer = rhwp_render_pdf(path.as_ptr(), 0, 1, std::ptr::null(), 0);
        assert!(buffer.data.is_null());
        assert!(!buffer.error.is_null());
        rhwp_buffer_free(buffer);
    }
}
