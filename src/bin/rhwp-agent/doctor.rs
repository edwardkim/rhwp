//! [#3918] `doctor` — 환경 자가진단.
//!
//! 에이전트가 낯선 환경(새 CI 러너·새 세션)에서 첫 호출 전에 도구 계약을 스스로
//! 점검한다: 버전, 컴파일된 기능, 임시 디렉터리 쓰기, (선택) 표본 문서 파싱.
//! 전부 통과면 0, 하나라도 실패면 게이트 관례대로 3 — "환경이 기대와 다르다".

use crate::envelope::{envelope, load_core, print_json, read_file, EXIT_GATE, EXIT_OK, EXIT_USAGE};
use serde_json::json;

struct Check {
    name: &'static str,
    ok: bool,
    detail: String,
}

pub fn run(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut sample: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--sample" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --sample 뒤에 파일 경로가 필요합니다.");
                    return EXIT_USAGE;
                };
                sample = Some(value.clone());
                i += 2;
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: rhwp-agent doctor [--json] [--sample <파일>]");
                return EXIT_USAGE;
            }
        }
    }

    let mut checks: Vec<Check> = Vec::new();

    // ① 버전 — 라이브러리와 이 바이너리는 같은 크레이트에서 나오므로 항상 일치해야
    // 한다. 불일치는 배포 사고(다른 빌드의 바이너리 혼입) 신호다.
    checks.push(Check {
        name: "version",
        ok: !rhwp::version().is_empty(),
        detail: format!("rhwp v{}", rhwp::version()),
    });

    // ② 컴파일 기능 — export-png 는 native-skia 기능이 필요하다. 에이전트가
    // 계획 단계에서 "이 환경에서 PNG 가 되는가"를 실행 전에 알 수 있게 한다.
    checks.push(Check {
        name: "features",
        ok: true,
        detail: format!("native-skia={}", cfg!(feature = "native-skia")),
    });

    // ③ 임시 쓰기 — 산출물을 쓰는 명령(evidence -o, fingerprint --write)의 전제.
    checks.push(tmp_write_check());

    // ④ 표본 파싱(선택) — 실제 문서 하나로 적재 경로 전체를 왕복한다.
    if let Some(path) = &sample {
        checks.push(sample_check(path));
    }

    let all_ok = checks.iter().all(|c| c.ok);

    if json_mode {
        // 표본 파싱 실패 detail 은 파서 메시지를 실을 수 있다 — 문서 파생일 수
        // 있으므로 보수적으로 선언한다(과소 선언 금지, #3885).
        let untrusted: &[&str] = if sample.is_some() {
            &["checks[].detail"]
        } else {
            &[]
        };
        let payload = json!({
            "ok": all_ok,
            "checks": checks
                .iter()
                .map(|c| json!({"name": c.name, "ok": c.ok, "detail": c.detail}))
                .collect::<Vec<_>>(),
        });
        print_json(&envelope("doctor", payload, untrusted));
    } else {
        crate::outln!("rhwp-agent doctor — 환경 자가진단");
        for c in &checks {
            crate::outln!(
                "  [{}] {} — {}",
                if c.ok { "통과" } else { "실패" },
                c.name,
                c.detail
            );
        }
        crate::outln!(
            "결과: {}",
            if all_ok {
                "전부 통과"
            } else {
                "실패 있음"
            }
        );
    }

    if all_ok {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}

fn tmp_write_check() -> Check {
    let dir = std::env::temp_dir();
    // 고유 이름 — 병렬 세션이 같은 임시 디렉터리를 쓰므로 PID 로 가른다.
    let path = dir.join(format!("rhwp-agent-doctor-{}.tmp", std::process::id()));
    let result = std::fs::write(&path, b"rhwp-agent")
        .and_then(|_| std::fs::read(&path))
        .and_then(|read| {
            std::fs::remove_file(&path)?;
            Ok(read == b"rhwp-agent")
        });
    match result {
        Ok(true) => Check {
            name: "tmpWrite",
            ok: true,
            detail: format!("{} 쓰기·읽기·삭제 왕복", dir.display()),
        },
        Ok(false) => Check {
            name: "tmpWrite",
            ok: false,
            detail: "읽은 내용이 쓴 내용과 다릅니다".to_string(),
        },
        Err(e) => Check {
            name: "tmpWrite",
            ok: false,
            detail: format!("임시 쓰기 실패 - {e}"),
        },
    }
}

fn sample_check(path: &str) -> Check {
    let started = std::time::Instant::now();
    let result = read_file(path).and_then(|data| {
        load_core(&data).map_err(|e| e.message).map(|core| {
            let pages = core.page_count();
            (data.len(), pages)
        })
    });
    match result {
        Ok((bytes, pages)) => Check {
            name: "sampleParse",
            ok: true,
            detail: format!(
                "{path}: {bytes} 바이트, {pages}쪽, {}ms",
                started.elapsed().as_millis()
            ),
        },
        Err(message) => Check {
            name: "sampleParse",
            ok: false,
            detail: format!("{path}: {message}"),
        },
    }
}
