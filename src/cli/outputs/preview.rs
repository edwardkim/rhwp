//! 문서에 저장된 미리보기 자산 출력 어댑터.

use std::fs;
use std::path::Path;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn extract_thumbnail(args: &[String]) -> i32 {
    // [#3366] 계약 정합 — 파싱은 #3349 규약(위치 무관, 미지 플래그 즉시 exit 2,
    // 중복 positional exit 2), 종료 코드는 #2707(사용법 오류 = 2). 종전에는 알 수
    // 없는 옵션을 조용히 무시한 채 산출물까지 만들고, 인자 없음이 1 로 끝났다.
    let mut input_path: Option<&str> = None;
    let mut output_path: Option<String> = None;
    let mut mode = "file"; // "file", "base64", "data-uri"
                           // [#3600] --json: 봉투를 stdout 순수 JSON 으로. 추출 동작 무변경.
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(p) => output_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: --output 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--base64" => mode = "base64",
            "--data-uri" => mode = "data-uri",
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if input_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(input_path) = input_path else {
        eprintln!("사용법: rhwp thumbnail <파일.hwp> [옵션]");
        eprintln!("  -o, --output <파일>   출력 파일 경로");
        eprintln!("  --base64              base64 문자열 출력");
        eprintln!("  --data-uri            data:image/... URI 출력");
        return EXIT_USAGE;
    };

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다: {} ({})", input_path, e);
            return EXIT_RUNTIME;
        }
    };

    let result = match rhwp::parser::extract_thumbnail_only(&data) {
        Some(r) => r,
        None => {
            eprintln!("오류: PrvImage 썸네일이 없습니다: {}", input_path);
            return EXIT_RUNTIME;
        }
    };

    let mime = match result.format.as_str() {
        "png" => "image/png",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    };

    // [#3600] JSON 봉투 공통부 — 모드별로 output/base64/dataUri 만 달라진다.
    let envelope_base = |extra: serde_json::Value| {
        let mut v = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": input_path,
            "format": result.format,
            "mime": mime,
            "width": result.width,
            "height": result.height,
            "bytes": result.data.len(),
            "output": serde_json::Value::Null,
        });
        if let (Some(obj), Some(e)) = (v.as_object_mut(), extra.as_object()) {
            for (k, val) in e {
                obj.insert(k.clone(), val.clone());
            }
        }
        // [#3787 S1] base64/dataUri 는 문서에 내장된 미리보기 이미지다 — extra 를
        // 합친 **뒤에** 표지를 찍어야 그 모드의 봉투가 맞게 표시된다.
        provenance::marked(v, "thumbnail")
    };

    match mode {
        "base64" => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&result.data);
            if json_mode {
                println!("{}", envelope_base(serde_json::json!({ "base64": b64 })));
            } else {
                println!("{}", b64);
            }
        }
        "data-uri" => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&result.data);
            let uri = format!("data:{};base64,{}", mime, b64);
            if json_mode {
                println!("{}", envelope_base(serde_json::json!({ "dataUri": uri })));
            } else {
                println!("{}", uri);
            }
        }
        _ => {
            // 파일 출력
            let out = output_path.unwrap_or_else(|| {
                let stem = Path::new(input_path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let ext = &result.format;
                format!("output/{}_thumb.{}", stem, ext)
            });

            // 출력 디렉토리 생성
            if let Some(parent) = Path::new(&out).parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).ok();
                }
            }

            match fs::write(&out, &result.data) {
                Ok(_) => {
                    if json_mode {
                        println!("{}", envelope_base(serde_json::json!({ "output": out })));
                    } else {
                        println!(
                            "썸네일 추출 완료: {} ({}x{}, {} bytes, {})",
                            out,
                            result.width,
                            result.height,
                            result.data.len(),
                            result.format
                        );
                    }
                }
                Err(e) => {
                    eprintln!("오류: 파일 저장 실패: {} ({})", out, e);
                    return EXIT_RUNTIME;
                }
            }
        }
    }
    EXIT_OK
}
