//! 초소형 모델이 문서를 훑고 이어 읽도록 돕는 read-only digest CLI 어댑터.

use std::fs;

use crate::{
    info_json_value, load_document, ENVELOPE_SCHEMA_VERSION, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};

/// [#3633] `nextStep` 고정 문자열 계약 — 봉투를 받은 초소형 모델이 다음 행동을
/// 지어내지 않고 받아 적게 하는 유도문. 문구 변경은 계약 테스트
/// (`tests/digest_macro_contract.rs`)가 잡는 의도적 결정이어야 한다.
const DIGEST_NEXT_STEP: &str = "더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json";
/// [#3633 후속] sections 모드 nextStep — 절 청크를 받은 모델이 쪽 주소로 원문을
/// 되짚게 하는 고정 유도문. v1 과 같은 고정 문자열 계약이다.
const DIGEST_SECTIONS_NEXT_STEP: &str =
    "절 원문은 export-text --json -p <쪽>, 찾으려면 search --json";
/// [#3633 후속] pages 모드에서 남은 범위가 없을 때의 고정 유도문.
const DIGEST_PAGES_DONE_NEXT_STEP: &str = "범위 발췌 완료 — 더 찾으려면 search --json";
/// [#3633] excerpt 기본 절단 길이(문자 수) — 4B급 모델의 컨텍스트 예산에 맞춘 보수값.
const DIGEST_DEFAULT_MAX_CHARS: usize = 2000;
/// [#3633 후속] sections 모드의 절별 발췌 기본 상한(문자 수) — 절이 수십 개일 수 있어
/// v1 의 2000자보다 훨씬 보수적으로 잡는다. `--max-chars` 로 절별 상한을 바꾼다.
const DIGEST_SECTION_EXCERPT_CHARS: usize = 240;
/// [#3633 후속] sections 봉투에 싣는 청크 최대 개수 — 전체 개수는 sectionCount 로
/// 따로 실어, 봉투만 보고 누락 여부를 판정할 수 있게 한다.
const DIGEST_SECTIONS_LIMIT: usize = 50;
/// [#3633] outline 에 싣는 최상위 노드 제목 최대 개수 — 트리 전체를 싣지 않는다.
const DIGEST_OUTLINE_LIMIT: usize = 20;
/// [#3633] excerpt 원천 페이지 수 — 앞쪽 페이지 0~2 만 발췌한다.
const DIGEST_EXCERPT_PAGES: u32 = 3;

/// [#3633 후속] `--pages a..b` 범위 파서 — 0 기준 양끝 포함, `a<=b` 만 유효.
/// 형식이 어긋나면 None(사용법 오류 처리).
fn parse_digest_pages(s: &str) -> Option<(u32, u32)> {
    let (a, b) = s.split_once("..")?;
    let from = a.parse::<u32>().ok()?;
    let to = b.parse::<u32>().ok()?;
    if from <= to {
        Some((from, to))
    } else {
        None
    }
}

/// [#3633] `digest` — 초소형 모델용 매크로 도구 축 1호.
///
/// 도구 체이닝을 못 하는 모델(4B급)을 위해 "info 로 훑고 → export-structure 로
/// 개요를 얻고 → export-text 로 첫 장을 읽는" 3단 파이프라인을 한 번 호출로
/// 결정론적으로 수행한다. 새 로직 없이 기존 원천만 재사용한다:
/// `load_document` → `info_json_value` 의 필드 + `build_structure` 상위 노드 제목 +
/// `extract_page_text_native` 발췌(`--max-chars` 문자 절단).
///
/// 출력은 항상 봉투 한 줄 JSON 이다(기계 전용 명령 — 표면 규약 통일을 위해
/// `--json` 플래그는 받아만 둔다). 실패 시 stdout 은 0바이트.
pub(crate) fn digest_document(args: &[String]) -> i32 {
    use rhwp::document_core::queries::structure::{build_structure, StructureMode};

    let mut file_path: Option<&str> = None;
    let mut max_chars: Option<usize> = None;
    let mut sections_mode = false;
    let mut pages_range: Option<(u32, u32)> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {}
            "--sections" => sections_mode = true,
            "--pages" => {
                i += 1;
                match args.get(i).and_then(|v| parse_digest_pages(v)) {
                    Some(r) => pages_range = Some(r),
                    None => {
                        eprintln!("오류: --pages 뒤에 a..b 형식(0 기준, a<=b)이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--max-chars" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n > 0 => max_chars = Some(n),
                    _ => {
                        eprintln!("오류: --max-chars 뒤에 1 이상의 숫자가 필요합니다.");
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
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    if sections_mode && pages_range.is_some() {
        eprintln!("오류: --sections 와 --pages 는 동시에 쓸 수 없습니다.");
        return EXIT_USAGE;
    }
    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp digest <파일> [--sections | --pages a..b] [--max-chars N] [--json]"
        );
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let file_size = data.len();
    let detected_format = rhwp::parser::detect_format(&data);
    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    // 메타는 info --json 과 같은 원천(info_json_value)에서 뽑는다 — 어휘 동형 보장.
    let info = info_json_value(file_path, file_size, detected_format, &doc);
    let page_count = doc.page_count();

    // 문자 수 기준 절단 (char 경계 안전). 발췌보다 짧으면 truncated 로 판정을 남긴다.
    let cut = |src: String, cap: usize| -> (String, bool) {
        if src.chars().count() > cap {
            (src.chars().take(cap).collect(), true)
        } else {
            (src, false)
        }
    };

    // ── [#3633 후속] sections 모드: 주소 보존 절 단위 청킹 ──────────────────
    // 페이지 발췌 대신 build_structure 의 최상위 노드를 청크로 낸다. 각 청크는
    // {title,page,charCount,excerpt} — page 는 제목 문단의 글로벌 쪽 번호(기존
    // get_page_of_position_native 재사용)라 요약 결과가 원문 쪽으로 되짚어진다.
    // charCount(절 전체) vs excerpt 길이로 소비자가 잔여량을 판정한다.
    if sections_mode {
        use rhwp::document_core::queries::structure::StructureNode;

        let cap = max_chars.unwrap_or(DIGEST_SECTION_EXCERPT_CHARS);
        let st = build_structure(doc.document(), StructureMode::Auto);

        // 절 본문 수집: 자기 body + 자식 제목·본문 전부 (하위 트리가 절의 내용이다).
        fn collect_section_text(node: &StructureNode, out: &mut String) {
            for line in &node.body {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(line);
            }
            for child in &node.children {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&child.heading);
                collect_section_text(child, out);
            }
        }

        let mut sections = Vec::new();
        let mut any_truncated = false;
        let (sections_mode_label, section_count): (&str, usize) = if st.roots.is_empty() {
            // 구조가 없는 문서: 쪽 단위 폴백으로 강등하되 sectionsMode 로 강등 사실을
            // 명시한다 — 쪽 번호는 그 자체로 주소라 인용 계약은 유지된다.
            for p in 0..page_count.min(DIGEST_SECTIONS_LIMIT as u32) {
                let text = match doc.extract_page_text_native(p) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {:?}", p, e);
                        return EXIT_RUNTIME;
                    }
                };
                let char_count = text.chars().count();
                let (excerpt, truncated) = cut(text, cap);
                any_truncated |= truncated;
                sections.push(serde_json::json!({
                    "title": "",
                    "page": p,
                    "charCount": char_count,
                    "excerpt": excerpt,
                }));
            }
            ("page", page_count as usize)
        } else {
            for node in st.roots.iter().take(DIGEST_SECTIONS_LIMIT) {
                // 제목 문단의 글로벌 쪽 번호 — 기존 위치→쪽 질의를 그대로 재사용한다.
                let page = match doc.get_page_of_position_native(node.section, node.paragraph) {
                    Ok(raw) => serde_json::from_str::<serde_json::Value>(&raw)
                        .ok()
                        .and_then(|v| v["page"].as_u64())
                        .unwrap_or(0),
                    Err(e) => {
                        eprintln!(
                            "오류: 절 '{}' 쪽 번호 조회 실패 - {:?}",
                            node.heading.trim(),
                            e
                        );
                        return EXIT_RUNTIME;
                    }
                };
                let mut text = String::new();
                collect_section_text(node, &mut text);
                let char_count = text.chars().count();
                let (excerpt, truncated) = cut(text, cap);
                any_truncated |= truncated;
                sections.push(serde_json::json!({
                    "title": node.heading.trim(),
                    "page": page,
                    "charCount": char_count,
                    "excerpt": excerpt,
                }));
            }
            (st.mode, st.roots.len())
        };

        let truncated = any_truncated || section_count > sections.len();
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "format": info["format"],
            "pageCount": info["pageCount"],
            "paraCount": info["paraCount"],
            "sectionsMode": sections_mode_label,
            "sectionCount": section_count,
            "sections": sections,
            "truncated": truncated,
            "nextStep": DIGEST_SECTIONS_NEXT_STEP,
        });
        println!("{}", envelope);
        return EXIT_OK;
    }

    // ── [#3633 후속] pages 모드: 범위 지정 발췌 (대형 문서 분할 요약용) ─────
    // nextStep 이 같은 폭의 다음 창을 그대로 받아 적게 안내한다 — 체이닝을 못 하는
    // 모델도 "이어 읽기"를 계획 없이 수행할 수 있다.
    if let Some((from, to)) = pages_range {
        if from >= page_count {
            eprintln!(
                "오류: 시작 쪽 {} 이 문서 범위(0..{}) 밖입니다.",
                from,
                page_count.saturating_sub(1)
            );
            return EXIT_RUNTIME;
        }
        let to = to.min(page_count - 1);
        let mut excerpt_src = String::new();
        for p in from..=to {
            match doc.extract_page_text_native(p) {
                Ok(text) => {
                    if !excerpt_src.is_empty() {
                        excerpt_src.push('\n');
                    }
                    excerpt_src.push_str(&text);
                }
                Err(e) => {
                    eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {:?}", p, e);
                    return EXIT_RUNTIME;
                }
            }
        }
        let (excerpt, truncated) = cut(excerpt_src, max_chars.unwrap_or(DIGEST_DEFAULT_MAX_CHARS));
        let next_step = if to + 1 < page_count {
            let next_from = to + 1;
            let next_to = (next_from + (to - from)).min(page_count - 1);
            format!("이어서 digest --json --pages {next_from}..{next_to}")
        } else {
            DIGEST_PAGES_DONE_NEXT_STEP.to_string()
        };
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "format": info["format"],
            "pageCount": info["pageCount"],
            "paraCount": info["paraCount"],
            "pages": { "from": from, "to": to },
            "excerpt": excerpt,
            "truncated": truncated,
            "nextStep": next_step,
        });
        println!("{}", envelope);
        return EXIT_OK;
    }

    // ── 기본(v1) 모드 — #3633 봉투 무회귀 ───────────────────────────────────
    // 구조 최상위 노드 제목만 싣는다 — 트리 전체는 export-structure 의 몫이다.
    let st = build_structure(doc.document(), StructureMode::Auto);
    let outline: Vec<&str> = st
        .roots
        .iter()
        .take(DIGEST_OUTLINE_LIMIT)
        .map(|n| n.heading.as_str())
        .collect();

    // 앞쪽 페이지 텍스트 발췌 → max_chars 문자에서 절단 (char 경계 안전).
    let mut excerpt_src = String::new();
    for p in 0..page_count.min(DIGEST_EXCERPT_PAGES) {
        match doc.extract_page_text_native(p) {
            Ok(text) => {
                if !excerpt_src.is_empty() {
                    excerpt_src.push('\n');
                }
                excerpt_src.push_str(&text);
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {:?}", p, e);
                return EXIT_RUNTIME;
            }
        }
    }
    let (excerpt, truncated) = cut(excerpt_src, max_chars.unwrap_or(DIGEST_DEFAULT_MAX_CHARS));

    let envelope = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "source": file_path,
        "format": info["format"],
        "pageCount": info["pageCount"],
        "paraCount": info["paraCount"],
        "outline": outline,
        "excerpt": excerpt,
        "truncated": truncated,
        "nextStep": DIGEST_NEXT_STEP,
    });
    println!("{}", envelope);
    EXIT_OK
}
