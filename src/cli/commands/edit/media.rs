//! Image and picture command adapters.

use std::fs;
use std::path::Path;
use std::process;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use super::runtime::{
    edit_output_format, edit_serialize, edit_verify_report, finish_edit_write, EditOutputFormat,
};
use crate::{load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

/// [#3719 §6-5] `edit insert-image` 가 받는 그림 형식.
///
/// BinData 로 넣을 수 있고 **원본 픽셀 크기를 헤더만 읽어 잴 수 있는** 형식만 담는다.
/// 크기를 못 재면 배율·배치 좌표가 의미를 잃으므로 삽입을 시작하지 않는다.
const INSERT_IMAGE_FORMATS: [&str; 6] = ["png", "jpg", "jpeg", "bmp", "tif", "tiff"];

/// 96dpi 픽셀 1개 = 75 HWPUNIT(7200/96). 코어가 crop 을 `px * 75` 로 잡는 것과 같은 환산비다.
const HWPUNIT_PER_PX: u32 = 75;

/// 그림의 원본 픽셀 크기 — 전체 디코드 없이 헤더만 읽는다.
///
/// 확장자는 거짓말할 수 있으므로 매직 바이트로 형식을 다시 판정한다. 알아보지 못하면
/// `None` — 호출부가 인자 오류(exit 2)로 끊는다.
fn insert_image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    use image::ImageFormat;

    let format = image::guess_format(bytes).ok()?;
    if !matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Bmp | ImageFormat::Tiff
    ) {
        return None;
    }
    let (width, height) = image::ImageReader::with_format(std::io::Cursor::new(bytes), format)
        .into_dimensions()
        .ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

/// `--page` 가 가리키는 쪽의 **앵커 문단**(구역 인덱스, 문단 인덱스).
///
/// 용지 기준(Paper-relative) floating 그림은 앵커 문단이 놓인 쪽에 그려진다. 그래서
/// "몇 쪽" 을 "어느 문단" 으로 옮겨야 하는데, 그 환산은 이미 조판 결과가 알고 있다 —
/// 기존 진단 질의 `dump_page_items_json` 을 그대로 읽어 그 쪽의 첫 본문 항목을 고른다
/// (새 조판 로직 0). 미주(`isEndnote`)는 구역 뒤에 합성된 문단이라 앵커로 쓰지 않는다.
fn insert_image_page_anchor(
    doc: &rhwp::wasm_api::HwpDocument,
    page: u32,
) -> Option<(usize, usize)> {
    let empty: Vec<serde_json::Value> = Vec::new();
    let pages = doc.dump_page_items_json(Some(page));
    let page_json = pages.as_array()?.first()?;
    let section = page_json["section"].as_u64()? as usize;

    for column in page_json["columns"].as_array().unwrap_or(&empty) {
        for item in column["items"].as_array().unwrap_or(&empty) {
            if item["isEndnote"] == true {
                continue;
            }
            if let Some(para) = item["paraIndex"].as_u64() {
                return Some((section, para as usize));
            }
        }
    }
    // 항목이 하나도 없는 쪽(어울림 문단·감춘 빈 줄만 귀속된 쪽)은 extras 로 온다.
    for extra in page_json["extras"].as_array().unwrap_or(&empty) {
        if let Some(para) = extra["paraIndex"].as_u64() {
            return Some((section, para as usize));
        }
    }
    None
}

struct InsertImageArgs<'a> {
    file_path: &'a str,
    image_path: &'a str,
    page: u32,
    x_hu: u32,
    y_hu: u32,
    width: Option<u32>,
    height: Option<u32>,
    output: Option<String>,
    dry_run: bool,
    json: bool,
    verify: bool,
}

fn parse_insert_image_args(args: &[String]) -> Result<InsertImageArgs<'_>, i32> {
    const USAGE: &str = "사용법: rhwp edit insert-image <파일> --image <그림> [--page N] [--x N --y N] [--width N --height N] [-o <출력>] [--dry-run] [--verify] [--json]";

    let mut file_path = None;
    let mut image_path = None;
    let mut page = 0;
    let mut x_hu = 0;
    let mut y_hu = 0;
    let mut width = None;
    let mut height = None;
    let mut output = None;
    let mut dry_run = false;
    let mut json = false;
    let mut verify = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--image" => {
                i += 1;
                match args.get(i) {
                    Some(value) => image_path = Some(value.as_str()),
                    None => {
                        eprintln!("오류: --image 뒤에 그림 파일 경로가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "--page" | "--x" | "--y" | "--width" | "--height" => {
                let name = args[i].as_str();
                // 단위를 오류 문구에도 박아 둔다 — px 로 넣으면 도장이 사라진다.
                let unit = if name == "--page" {
                    " (0부터)"
                } else {
                    " (HWPUNIT, 1/7200 inch)"
                };
                i += 1;
                let Some(raw) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다{unit}.");
                    return Err(EXIT_USAGE);
                };
                let Ok(value) = raw.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다{unit}: {raw}");
                    return Err(EXIT_USAGE);
                };
                match name {
                    "--page" => page = value,
                    "--x" => x_hu = value,
                    "--y" => y_hu = value,
                    "--width" => width = Some(value),
                    _ => height = Some(value),
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(value) => output = Some(value.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json = true,
            "--verify" => verify = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return Err(EXIT_USAGE);
                }
            }
        }
        i += 1;
    }

    let (Some(file_path), Some(image_path)) = (file_path, image_path) else {
        eprintln!("{USAGE}");
        return Err(EXIT_USAGE);
    };
    for (name, value) in [("--width", width), ("--height", height)] {
        if value == Some(0) {
            eprintln!("오류: {name} 는 1 이상이어야 합니다 (HWPUNIT, 1/7200 inch).");
            return Err(EXIT_USAGE);
        }
    }

    Ok(InsertImageArgs {
        file_path,
        image_path,
        page,
        x_hu,
        y_hu,
        width,
        height,
        output,
        dry_run,
        json,
        verify,
    })
}

/// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
///
/// 실물 서식 제출의 마지막 조각이다. 채워 넣은 서식에 직인·서명 이미지를 얹지 못하면
/// 사람이 한 번 더 한컴을 열어야 하고, 그 순간 자동화 사슬이 끊긴다.
///
/// 새 삽입 로직을 만들지 않는다 — 검증된 코어 `insert_picture_native` 의 **본문 floating
/// 분기**(용지 기준 offset, `treat_as_char=false`, 한컴 native 기본값)를 그대로 쓴다.
/// 인자 파싱·저장·봉투·`--verify`·`changedPages` 는 `edit set-cell` 과 같은 형태다.
///
/// **길이 단위는 전부 HWPUNIT(1/7200 inch)** 이다 — px 로 오해하면 도장이 점만 하게
/// 찍히거나 아예 안 보인다. A4 세로는 59528 × 84188 HWPUNIT.
pub(super) fn edit_insert_image(args: &[String]) -> i32 {
    let InsertImageArgs {
        file_path,
        image_path,
        page: page_arg,
        x_hu,
        y_hu,
        width: width_arg,
        height: height_arg,
        output: out_path,
        dry_run,
        json: json_mode,
        verify: verify_mode,
    } = match parse_insert_image_args(args) {
        Ok(parsed) => parsed,
        Err(code) => return code,
    };

    // ── 그림 선검증 — 문서를 읽기 전에 끊는다 ──
    // 지원하지 않는 형식은 **인자 문제**다(런타임 실패가 아니다) → exit 2.
    let image_ext = Path::new(image_path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if !INSERT_IMAGE_FORMATS.contains(&image_ext.as_str()) {
        eprintln!(
            "오류: 지원하지 않는 그림 형식입니다 - {} (지원: {})",
            if image_ext.is_empty() {
                "확장자 없음".to_string()
            } else {
                image_ext.clone()
            },
            INSERT_IMAGE_FORMATS.join(", ")
        );
        return EXIT_USAGE;
    }
    let image_bytes = match fs::read(image_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 그림 파일을 읽을 수 없습니다 - {}: {}", image_path, e);
            return EXIT_RUNTIME;
        }
    };
    // 확장자만 믿지 않는다 — 내용이 그림이 아니면 원본 픽셀 크기를 못 재고,
    // 크기를 모르면 배치 좌표가 의미를 잃는다.
    let Some((natural_w_px, natural_h_px)) = insert_image_dimensions(&image_bytes) else {
        eprintln!(
            "오류: 그림 형식을 알아볼 수 없습니다 - {} (지원: {})",
            image_path,
            INSERT_IMAGE_FORMATS.join(", ")
        );
        return EXIT_USAGE;
    };

    // 크기 결정: 둘 다 없으면 원본 픽셀(96dpi 환산), 하나만 주면 원본 비율 유지.
    // 어느 쪽이든 최종 값은 봉투에 그대로 실어 "조용한 보정" 이 없게 한다.
    let (width_hu, height_hu) = match (width_arg, height_arg) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => (
            w,
            ((w as u64 * natural_h_px as u64) / natural_w_px as u64).max(1) as u32,
        ),
        (None, Some(h)) => (
            ((h as u64 * natural_w_px as u64) / natural_h_px as u64).max(1) as u32,
            h,
        ),
        (None, None) => (
            natural_w_px.saturating_mul(HWPUNIT_PER_PX),
            natural_h_px.saturating_mul(HWPUNIT_PER_PX),
        ),
    };
    // 코어는 offset·크기를 i32/u32 로 다룬다. 범위를 넘는 값이 조용히 감기면 도장이
    // 엉뚱한 곳에 찍히므로 인자 오류로 끊는다.
    for (name, value) in [
        ("--x", x_hu),
        ("--y", y_hu),
        ("--width", width_hu),
        ("--height", height_hu),
    ] {
        if value > i32::MAX as u32 {
            eprintln!(
                "오류: {name} 값이 너무 큽니다 (HWPUNIT 최대 {}): {value}",
                i32::MAX
            );
            return EXIT_USAGE;
        }
    }

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let page_count = doc.page_count();
    if page_arg >= page_count {
        eprintln!(
            "오류: 페이지 번호가 범위를 벗어났습니다 (0~{}): {page_arg}",
            page_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let Some((sec, para)) = insert_image_page_anchor(&doc, page_arg) else {
        eprintln!("오류: {page_arg}쪽(0 기준)에서 그림을 붙일 본문 문단을 찾지 못했습니다.");
        return EXIT_RUNTIME;
    };

    // [#3480 과 같은 취지] 쪽 밖으로 나가면 **조용히 자르지 않는다**. 에이전트는 렌더
    // 결과를 보지 않으므로 신호가 없으면 잘려 나간 도장을 완성본으로 판단한다.
    let page_def = &doc.document().sections[sec].section_def.page_def;
    let (paper_w, paper_h) = if page_def.landscape {
        (page_def.height as i64, page_def.width as i64)
    } else {
        (page_def.width as i64, page_def.height as i64)
    };
    let right = x_hu as i64 + width_hu as i64;
    let bottom = y_hu as i64 + height_hu as i64;
    let overflow = if right > paper_w || bottom > paper_h {
        Some(serde_json::json!({
            "page": page_arg,
            "paperWidthHu": paper_w,
            "paperHeightHu": paper_h,
            "rightHu": right,
            "bottomHu": bottom,
            "overflowXHu": (right - paper_w).max(0),
            "overflowYHu": (bottom - paper_h).max(0),
        }))
    } else {
        None
    };

    let mut bin_data_id = serde_json::Value::Null;
    if !dry_run {
        // 그림 설명(대체 텍스트)은 파일명 — 한컴이 개체 속성에 보여 주는 값이다.
        let description = Path::new(image_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let inserted = match doc.insert_picture_native(
            sec,
            para,
            0,
            &[],
            &image_bytes,
            width_hu,
            height_hu,
            natural_w_px,
            natural_h_px,
            &image_ext,
            &description,
            Some(x_hu as i32),
            Some(y_hu as i32),
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 그림 삽입 실패 - {}", e);
                // 실패 시 원본 불변 — 출력 파일을 쓰지 않고 즉시 끝낸다.
                return EXIT_RUNTIME;
            }
        };
        // binDataId 는 새 조회 API 없이 방금 삽입한 컨트롤에서 직접 읽는다 —
        // 같은 그림을 다시 참조하거나(도장 재사용) 산출물을 감사할 때 쓰는 주소다.
        let ctrl_idx = serde_json::from_str::<serde_json::Value>(&inserted)
            .ok()
            .and_then(|v| v["controlIdx"].as_u64())
            .unwrap_or_default() as usize;
        if let Some(rhwp::model::control::Control::Picture(picture)) = doc
            .document()
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .and_then(|p| p.controls.get(ctrl_idx))
        {
            bin_data_id = serde_json::json!(picture.image_attr.bin_data_id);
        }
    }

    // [#3383] 입력 형식을 보존한다 — 기본 확장자도 산출 형식을 따른다.
    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_image.{}", stem, out_format.ext())
    });

    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if !dry_run {
        let out_bytes = match edit_serialize(&mut doc, out_format) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "오류: {} 직렬화 실패 - {}",
                    out_format.label().to_uppercase(),
                    e
                );
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = fs::write(&output_path, &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        // [#3702] 저장 직후 자기검증 — 편집 후 IR ↔ 저장본 재파싱 IR.
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    // [#3712] 눈검증 대상 페이지 — 앵커 문단이 걸친 쪽 전부.
    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(&[(sec, para)]) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "image": image_path,
            "page": page_arg,
            "x": x_hu,
            "y": y_hu,
            "width": width_hu,
            "height": height_hu,
            "binDataId": bin_data_id,
            "dryRun": dry_run,
            "changedPages": changed_pages,
            "overflow": overflow.clone().map(|o| vec![o]).unwrap_or_default(),
        });
        if !dry_run {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        // [#3885] 이 봉투의 값은 전부 호출자 인자·엔진 판정이라 문서 유래 경로가
        // 없지만, 표지 자체는 항상 싣는다 — 키 부재는 "안전"이 아니라 "판정 안 함"
        // 으로 읽어야 하기 때문이다(S1).
        println!("{}", provenance::marked(envelope, "edit"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "배치 예정: {} {}쪽 ({}, {}) 크기 {}×{} HWPUNIT ← {} (원본 {}×{}px)",
            file_path,
            page_arg,
            x_hu,
            y_hu,
            width_hu,
            height_hu,
            image_path,
            natural_w_px,
            natural_h_px
        );
    } else {
        println!(
            "그림 삽입 완료: {} → {} — {}쪽 ({}, {}) 크기 {}×{} HWPUNIT ← {} (원본 {}×{}px)",
            file_path,
            output_path,
            page_arg,
            x_hu,
            y_hu,
            width_hu,
            height_hu,
            image_path,
            natural_w_px,
            natural_h_px
        );
    }
    if overflow.is_some() {
        eprintln!(
            "경고: 그림이 쪽 밖으로 나갑니다 (용지 {}×{} HWPUNIT, 오른쪽 {} 아래 {}) — 상세는 --json 의 overflow",
            paper_w, paper_h, right, bottom
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}

/// `edit insert-picture` — 문단 좌표에 본문 그림을 끼운다. 코어 `insert_picture_native`.
/// `insert-image`(도장·서명, 쪽 좌표) 와 다르다.
pub(super) fn edit_insert_picture(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-picture <파일> --image <그림> [--section N] [--para N] [--offset N] [--width N] [--height N] [--x N] [--y N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut image_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut x_hu: u32 = 0;
    let mut y_hu: u32 = 0;
    let mut width_arg: Option<u32> = None;
    let mut height_arg: Option<u32> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--image" => {
                i += 1;
                match args.get(i) {
                    Some(v) => image_path = Some(v),
                    None => {
                        eprintln!("오류: --image 뒤에 그림 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--para" | "--offset" | "--x" | "--y" | "--width" | "--height" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--offset" => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--x" => match v.parse::<u32>() {
                        Ok(n) => x_hu = n,
                        Err(_) => {
                            eprintln!("오류: --x 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--y" => match v.parse::<u32>() {
                        Ok(n) => y_hu = n,
                        Err(_) => {
                            eprintln!("오류: --y 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--width" => match v.parse::<u32>() {
                        Ok(n) => width_arg = Some(n),
                        Err(_) => {
                            eprintln!(
                                "오류: --width 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u32>() {
                        Ok(n) => height_arg = Some(n),
                        Err(_) => {
                            eprintln!(
                                "오류: --height 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(image_path)) = (file_path, image_path) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    for (name, value) in [("--width", width_arg), ("--height", height_arg)] {
        if value == Some(0) {
            eprintln!("오류: {name} 는 1 이상이어야 합니다 (HWPUNIT, 1/7200 inch).");
            return EXIT_USAGE;
        }
    }
    let image_ext = Path::new(image_path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if !INSERT_IMAGE_FORMATS.contains(&image_ext.as_str()) {
        eprintln!(
            "오류: 지원하지 않는 그림 형식입니다 - {} (지원: {})",
            if image_ext.is_empty() {
                "확장자 없음".to_string()
            } else {
                image_ext.clone()
            },
            INSERT_IMAGE_FORMATS.join(", ")
        );
        return EXIT_USAGE;
    }
    let image_bytes = match fs::read(image_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 그림 파일을 읽을 수 없습니다 - {}: {}", image_path, e);
            return EXIT_RUNTIME;
        }
    };
    let Some((natural_w_px, natural_h_px)) = insert_image_dimensions(&image_bytes) else {
        eprintln!(
            "오류: 그림 형식을 알아볼 수 없습니다 - {} (지원: {})",
            image_path,
            INSERT_IMAGE_FORMATS.join(", ")
        );
        return EXIT_USAGE;
    };
    let (width_hu, height_hu) = match (width_arg, height_arg) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => (
            w,
            ((w as u64 * natural_h_px as u64) / natural_w_px as u64).max(1) as u32,
        ),
        (None, Some(h)) => (
            ((h as u64 * natural_w_px as u64) / natural_h_px as u64).max(1) as u32,
            h,
        ),
        (None, None) => (
            natural_w_px.saturating_mul(HWPUNIT_PER_PX),
            natural_h_px.saturating_mul(HWPUNIT_PER_PX),
        ),
    };
    for (name, value) in [
        ("--x", x_hu),
        ("--y", y_hu),
        ("--width", width_hu),
        ("--height", height_hu),
    ] {
        if value > i32::MAX as u32 {
            eprintln!(
                "오류: {name} 값이 너무 큽니다 (HWPUNIT 최대 {}): {value}",
                i32::MAX
            );
            return EXIT_USAGE;
        }
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let section_count = doc.document().sections.len();
    if section >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[section].paragraphs.len();
    if para >= para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section} 문단 0~{}): {para}",
            para_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let mut bin_data_id = serde_json::Value::Null;
    if !dry_run {
        let description = Path::new(image_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let inserted = match doc.insert_picture_native(
            section,
            para,
            offset,
            &[],
            &image_bytes,
            width_hu,
            height_hu,
            natural_w_px,
            natural_h_px,
            &image_ext,
            &description,
            Some(x_hu as i32),
            Some(y_hu as i32),
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 그림 삽입 실패 - {e}");
                return EXIT_RUNTIME;
            }
        };
        let ctrl_idx = serde_json::from_str::<serde_json::Value>(&inserted)
            .ok()
            .and_then(|v| v["controlIdx"].as_u64())
            .unwrap_or_default() as usize;
        if let Some(rhwp::model::control::Control::Picture(picture)) = doc
            .document()
            .sections
            .get(section)
            .and_then(|s| s.paragraphs.get(para))
            .and_then(|p| p.controls.get(ctrl_idx))
        {
            bin_data_id = serde_json::json!(picture.image_attr.bin_data_id);
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "picture",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "image": image_path,
            "section": section,
            "paragraph": para,
            "offset": offset,
            "x": x_hu,
            "y": y_hu,
            "width": width_hu,
            "height": height_hu,
            "binDataId": bin_data_id,
        }),
        &[(section, para)],
        &format!(
            "그림 삽입 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset} ← {image_path}"
        ),
        &format!("그림 삽입 완료: {file_path}"),
    )
}

/// `edit delete-picture` — 본문 그림 컨트롤 삭제. 코어 `delete_picture_control_native`.
pub(super) fn edit_delete_picture(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-picture <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_picture_control_native(section, para, ctrl) {
            eprintln!("오류: 그림 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delpic",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("그림 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("그림 삭제 완료: {file_path}"),
    )
}

/// `edit set-picture` — 본문 그림 속성. 코어 `set_picture_properties_native`.
pub(super) fn edit_set_picture(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-picture <파일> --section N --para N --ctrl N --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut props: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) => props = Some(v.clone()),
                    None => {
                        eprintln!("오류: --props 뒤에 JSON 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl), Some(props)) =
        (file_path, section, para, ctrl, props)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if props.trim().is_empty() {
        eprintln!("오류: --props 는 비어 있을 수 없습니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_picture_properties_native(section, para, ctrl, &props) {
            eprintln!("오류: 그림 속성 설정 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "setpic",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("그림 속성 변경 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("그림 속성 변경 완료: {file_path}"),
    )
}
