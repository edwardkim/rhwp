//! DocLang XML output adapter.

use std::fs;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{paths_refer_to_same_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn export_doclang(args: &[String]) -> i32 {
    // [#3359] 위치 인자 파싱은 export-structure/export-text(#3349) 규약과 동일.
    let mut file_path: Option<&str> = None;
    let mut output_override: Option<std::path::PathBuf> = None;
    let mut assets_dir: Option<std::path::PathBuf> = None;
    // [#3696] --json: 산출 봉투를 stdout 순수 JSON 으로. 변환 동작 무변경.
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_override = Some(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 파일 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--assets-dir" => {
                if i + 1 < args.len() {
                    assets_dir = Some(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --assets-dir 뒤에 디렉터리 경로가 필요합니다.");
                    return EXIT_USAGE;
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
                i += 1;
            }
        }
    }

    let Some(file_path) = file_path else {
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        eprintln!(
            "사용법: rhwp export-doclang <파일.hwp|파일.hwpx> [-o <출력.xml>] [--assets-dir <디렉터리>] [--json] (rhwp --help 참조)"
        );
        return EXIT_USAGE;
    };

    // 기본 출력 경로: 입력 stem + `.dclg.xml` (입력 파일 옆).
    let input_path = std::path::Path::new(file_path);
    let output_path = output_override.unwrap_or_else(|| input_path.with_extension("dclg.xml"));
    if paths_refer_to_same_file(input_path, &output_path) {
        eprintln!("오류: 입력과 출력 경로가 같습니다. 원본을 덮어쓰지 않습니다.");
        return EXIT_USAGE;
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "오류: 파일을 읽을 수 없습니다 - {}: {}",
                input_path.display(),
                e
            );
            return EXIT_RUNTIME;
        }
    };

    // 자원 정책: --assets-dir 지정 시 AssetDir(디렉터리 경로를 URI 접두어로), 아니면 인라인.
    let mut opts = rhwp::doclang::ConvertOptions::default();
    if let Some(dir) = &assets_dir {
        opts.resource_policy =
            rhwp::doclang::ResourcePolicy::asset_dir(dir.to_string_lossy().into_owned());
    }

    let outcome = match rhwp::doclang::convert(&data, &opts) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: DocLang 변환 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    // 이진 자원을 먼저 기록한다(있을 때만) — XML 저장 전에 실패를 드러내기 위함.
    if let Some(dir) = &assets_dir {
        if !outcome.assets.is_empty() {
            if let Err(e) = fs::create_dir_all(dir) {
                eprintln!(
                    "오류: 에셋 디렉터리를 만들 수 없습니다 - {}: {}",
                    dir.display(),
                    e
                );
                return EXIT_RUNTIME;
            }
            for asset in &outcome.assets {
                let asset_path = dir.join(&asset.path);
                if let Err(e) = fs::write(&asset_path, &asset.data) {
                    eprintln!("오류: 에셋 저장 실패 - {}: {}", asset_path.display(), e);
                    return EXIT_RUNTIME;
                }
            }
        }
    }

    match fs::write(&output_path, outcome.xml.as_bytes()) {
        Ok(_) => {
            if json_mode {
                // [#3696] 산출 봉투 — 사람용 출력(크기·에셋·손실 건수)의 기계 대응물.
                // assetsDir 는 --assets-dir 를 준 경우에만 문자열, 아니면 null.
                println!(
                    "{}",
                    provenance::marked(
                        serde_json::json!({
                            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                            "source": file_path,
                            "output": output_path.display().to_string(),
                            "format": "doclang",
                            "doclangVersion": rhwp::doclang::DOCLANG_VERSION,
                            "bytes": outcome.xml.len(),
                            "assetsDir": assets_dir.as_ref().map(|d| d.display().to_string()),
                            "assetCount": outcome.assets.len(),
                            "lossCount": outcome.loss.len(),
                        }),
                        "export-doclang",
                    )
                );
                return EXIT_OK;
            }
            println!(
                "저장 완료: {} ({}KB)",
                output_path.display(),
                outcome.xml.len() / 1024
            );
            if let Some(dir) = &assets_dir {
                if !outcome.assets.is_empty() {
                    println!("에셋 {}개 저장: {}", outcome.assets.len(), dir.display());
                }
            }
            let loss_count = outcome.loss.len();
            if loss_count > 0 {
                println!(
                    "손실 보고: {}건 (DocLang v0.6 으로 표현할 수 없는 정보)",
                    loss_count
                );
            }
            EXIT_OK
        }
        Err(e) => {
            eprintln!("오류: 파일 저장 실패 - {}: {}", output_path.display(), e);
            EXIT_RUNTIME
        }
    }
}
