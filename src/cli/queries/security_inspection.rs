//! 문서의 비가시 텍스트를 보고하는 read-only 보안 조회 CLI 어댑터.

use std::fs;

use crate::{
    load_document, provenance, ENVELOPE_SCHEMA_VERSION, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};

/// `inspect hidden-text` — 사람 눈에 안 보이는데 추출기는 읽어 가는 텍스트를 보고한다.
///
/// 탐지 건수가 0이 아니어도 종료 코드는 0이다 — 1은 런타임 실패 전용이고(#2707),
/// "위험 문서 발견"은 실패가 아니라 **정상적으로 얻어낸 판정 결과**다. 소비자는
/// `clean` 필드로 분기한다.
pub(crate) fn inspect_hidden_text(args: &[String]) -> i32 {
    use rhwp::document_core::queries::hidden_text::HiddenTextOptions;

    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    let mut opts = HiddenTextOptions::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--include-offpage" => opts.include_off_page = true,
            "--threshold-pt" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<f64>().ok()) {
                    // 상한은 CharShape.base_size 의 스펙 상한(4096pt)과 같다.
                    Some(n) if n.is_finite() && (0.0..=4096.0).contains(&n) => {
                        opts.threshold_pt = n
                    }
                    _ => {
                        eprintln!(
                            "오류: --threshold-pt 뒤에 0 이상 4096 이하의 실수가 필요합니다."
                        );
                        return EXIT_USAGE;
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다.");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp inspect hidden-text <파일.hwp|파일.hwpx> [--json] [--threshold-pt <N>] [--include-offpage]");
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let report = doc.detect_hidden_text(&opts);

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "thresholdPt": opts.threshold_pt,
            "includeOffPage": opts.include_off_page,
            "hiddenText": report.hidden_text,
            "hiddenCharCount": report.hidden_char_count,
            "clean": report.clean,
        });
        println!("{}", provenance::marked(envelope, "inspect"));
        return EXIT_OK;
    }

    // 기본 출력은 사람용 요약 — 기계 소비는 --json 이 담당한다.
    if report.clean {
        println!("은닉 텍스트 없음: {} (탐지 0건)", file_path);
        return EXIT_OK;
    }
    println!(
        "은닉 텍스트 {}건 (문자 {}개): {}",
        report.hidden_text.len(),
        report.hidden_char_count,
        file_path
    );
    for f in &report.hidden_text {
        let kind = match f.kind {
            rhwp::document_core::queries::hidden_text::HiddenKind::SameAsBackground => {
                "배경색과 같은 글자색"
            }
            rhwp::document_core::queries::hidden_text::HiddenKind::NearInvisible => "극소 글자",
            rhwp::document_core::queries::hidden_text::HiddenKind::ZeroSize => "0pt 글자",
            rhwp::document_core::queries::hidden_text::HiddenKind::OffPage => "쪽 밖 배치",
        };
        let page = f
            .page
            .map(|p| format!("{}쪽", p + 1))
            .unwrap_or_else(|| "미배치".to_string());
        println!(
            "  [{}] 구역{}:문단{} ({}) {}자: {}",
            kind, f.section, f.paragraph, page, f.char_count, f.excerpt
        );
    }
    EXIT_OK
}
