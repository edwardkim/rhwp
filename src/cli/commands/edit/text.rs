//! Document-wide text replacement command adapter.

use std::fs;
use std::path::Path;
use std::process;

use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use super::runtime::{
    check_expect_sha256, edit_output_format, edit_serialize, edit_verify_report, EditOutputFormat,
};
use crate::cli::integrity::{
    cas_test_mark_checked_and_wait, cas_test_synchronize_before_lock, CasPathLock,
};
use crate::{load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

struct ReplaceTextArgs<'a> {
    file_path: &'a str,
    find: &'a str,
    replace: &'a str,
    out_path: Option<String>,
    ignore_case: bool,
    dry_run: bool,
    json_mode: bool,
    verify_mode: bool,
    expect_sha256: Option<String>,
    occurrence: Option<usize>,
}

fn collect_replacement_hits(
    doc: &rhwp::document_core::DocumentCore,
    find: &str,
    case_sensitive: bool,
) -> Result<Vec<serde_json::Value>, String> {
    let json = doc
        .search_all_text_native(find, case_sensitive, true)
        .map_err(|e| format!("{e:?}"))?;
    serde_json::from_str::<Vec<serde_json::Value>>(&json)
        .map_err(|e| format!("search_all_text_native JSON 파싱 실패: {e}"))
}

fn replacement_changed_para(hit: &serde_json::Value) -> Option<(usize, usize)> {
    let sec = hit.get("sec")?.as_u64()? as usize;
    let para = hit
        .get("cellContext")
        .and_then(|cell| cell.get("parentPara"))
        .or_else(|| hit.get("para"))?
        .as_u64()? as usize;
    Some((sec, para))
}

/// `edit replace-text` — 문서 전체 일괄 치환 (기관명 변경·연도 갱신·용어 정비).
///
/// [#3373] 검증된 코어 경로(`replace_all` — 역순 치환으로 오프셋 안전, 본문+표 셀)를
/// 재사용하므로 새 편집 로직이 없다. `--dry-run` 은 파일 생성 경로를 타지 않고
/// 실제 치환과 같은 검색 순회(`search_all_text_native`)로 치환 예정 건수만 보고한다.
/// **0건이면 출력 파일을 만들지
/// 않는다** — 무변경 산출물이 생기지 않게 한다.
pub(super) fn edit_replace_text(args: &[String]) -> i32 {
    let ReplaceTextArgs {
        file_path,
        find,
        replace,
        out_path,
        ignore_case,
        dry_run,
        json_mode,
        verify_mode,
        expect_sha256,
        occurrence,
    } = match parse_replace_text_args(args) {
        Ok(parsed) => parsed,
        Err(code) => return code,
    };

    let _cas_lock = match expect_sha256.as_ref() {
        Some(_) => {
            if let Err(e) = cas_test_synchronize_before_lock() {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
            match CasPathLock::acquire(Path::new(file_path)) {
                Ok(lock) => Some(lock),
                Err(e) => {
                    eprintln!("오류: 입력 문서 CAS 잠금을 얻을 수 없습니다 - {file_path}: {e}");
                    return EXIT_RUNTIME;
                }
            }
        }
        None => None,
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    // [#4378 R24] 파싱 전에 CAS 대조 — 기대 상태가 아니면 여기서 끝(디스크 무변경).
    if let Some(code) = check_expect_sha256(expect_sha256.as_deref(), &bytes, file_path, json_mode)
    {
        return code;
    }
    if expect_sha256.is_some() {
        cas_test_mark_checked_and_wait();
    }
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let replacement_hits = match collect_replacement_hits(&doc, find, !ignore_case) {
        Ok(hits) => hits,
        Err(e) => {
            eprintln!("오류: 치환 대상 검색 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let selected_hits: Vec<&serde_json::Value> = match occurrence {
        Some(n) => replacement_hits.get(n).into_iter().collect(),
        None => replacement_hits.iter().collect(),
    };

    // [#3712] 치환 전 매치 주소를 붙잡는다 — 문자열 치환은 문단 인덱스를 밀지 않는다.
    let changed_paras: Vec<(usize, usize)> = if dry_run {
        Vec::new()
    } else {
        selected_hits
            .iter()
            .filter_map(|hit| replacement_changed_para(hit))
            .collect()
    };

    let replaced_count = if dry_run {
        // 파일을 건드리지 않는다 — 실제 치환과 같은 순회로 예정 건수만 센다.
        selected_hits.len()
    } else {
        let result = match match occurrence {
            Some(n) => doc.replace_nth_native(find, replace, !ignore_case, n),
            None => doc.replace_all_native(find, replace, !ignore_case),
        } {
            Ok(r) => r,
            Err(e) => {
                eprintln!("오류: 치환 실패 - {:?}", e);
                // 실패 시 원본 불변 — 출력 파일을 쓰지 않고 즉시 끝낸다.
                return EXIT_RUNTIME;
            }
        };
        serde_json::from_str::<serde_json::Value>(&result)
            .ok()
            .and_then(|v| v["count"].as_u64())
            .unwrap_or(0) as usize
    };

    // [#3383] 입력 형식을 보존한다 — 기본 확장자도 산출 형식을 따른다.
    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_replaced.{}", stem, out_format.ext())
    });

    // 0건이면 무변경이다 — 산출물을 만들지 않는다 (dry-run 과 동일하게 파일 경로를 타지 않음).
    let wrote_output = !dry_run && replaced_count > 0;
    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if wrote_output {
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
        if expect_sha256.is_some() {
            let latest = match fs::read(file_path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("오류: 저장 직전 입력을 다시 읽을 수 없습니다 - {file_path}: {e}");
                    return EXIT_RUNTIME;
                }
            };
            if let Some(code) =
                check_expect_sha256(expect_sha256.as_deref(), &latest, file_path, json_mode)
            {
                return code;
            }
        }
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

    // [#3712] 눈검증 대상 페이지 — 산출물이 있을 때만 의미가 있다(무산출은 null).
    let changed_pages = if wrote_output {
        match doc.pages_covering_paragraphs(&changed_paras) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    } else {
        serde_json::Value::Null
    };

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "find": find,
            "replace": replace,
            "occurrence": occurrence,
            "caseSensitive": !ignore_case,
            "dryRun": dry_run,
            "changedPages": changed_pages,
            "replacedCount": replaced_count,
        });
        if wrote_output {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        println!("{}", envelope);
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "변경 예정: {} — {:?} → {:?} ({}건)",
            file_path, find, replace, replaced_count
        );
    } else if replaced_count == 0 {
        println!(
            "치환 0건: {} — {:?} 없음 (출력 파일 미생성)",
            file_path, find
        );
    } else {
        println!(
            "치환 완료: {} → {} — {:?} → {:?} ({}건)",
            file_path, output_path, find, replace, replaced_count
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}

fn parse_replace_text_args(args: &[String]) -> Result<ReplaceTextArgs<'_>, i32> {
    let mut file_path: Option<&str> = None;
    let mut find_arg: Option<&str> = None;
    let mut replace_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut ignore_case = false;
    let mut dry_run = false;
    let mut json_mode = false;
    // [#3702] 저장 직후 자기검증 — 판정은 데이터, 차이 시 exit 3.
    let mut verify_mode = false;
    // [#4378 R24] CAS — 입력이 이 해시일 때만 진행(다른 에이전트의 선행 편집 감지).
    let mut expect_sha256: Option<String> = None;
    // [#3395] 문서 순서 k번째(0 기준) 매치만 치환 — 체크박스류 반복 문자 지목.
    let mut occurrence: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--find" => {
                i += 1;
                match args.get(i) {
                    Some(v) => find_arg = Some(v),
                    None => {
                        eprintln!("오류: --find 뒤에 찾을 문자열이 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "--replace" => {
                i += 1;
                match args.get(i) {
                    Some(v) => replace_arg = Some(v),
                    None => {
                        eprintln!("오류: --replace 뒤에 바꿀 문자열이 필요합니다 (삭제는 \"\").");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "--ignore-case" => ignore_case = true,
            "--occurrence" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) => occurrence = Some(n),
                    None => {
                        eprintln!("오류: --occurrence 뒤에 0 이상의 정수가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            "--expect-sha256" => {
                i += 1;
                match args.get(i) {
                    Some(v) => expect_sha256 = Some(v.clone()),
                    None => {
                        eprintln!("오류: --expect-sha256 뒤에 64자리 16진 해시가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
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

    let (Some(file_path), Some(find), Some(replace)) = (file_path, find_arg, replace_arg) else {
        eprintln!(
            "사용법: rhwp edit replace-text <파일.hwp|파일.hwpx> --find <문자열> --replace <문자열> [-o <출력>] [--ignore-case] [--dry-run] [--json]"
        );
        return Err(EXIT_USAGE);
    };
    if find.is_empty() {
        eprintln!("오류: --find 는 빈 문자열일 수 없습니다.");
        return Err(EXIT_USAGE);
    }

    Ok(ReplaceTextArgs {
        file_path,
        find,
        replace,
        out_path,
        ignore_case,
        dry_run,
        json_mode,
        verify_mode,
        expect_sha256,
        occurrence,
    })
}
