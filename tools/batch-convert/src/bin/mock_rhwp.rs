//! 테스트 전용 rhwp CLI 대역(mock).
//!
//! batch-convert 통합 테스트(tests/cli.rs)가 실제 rhwp 빌드 없이 병렬 상한·
//! 재시도·overwrite·산출물 규약을 검증할 수 있게 한다. 호출 계약은
//! converter.rs 의 `run_rhwp_export` 와 같다:
//! `mock-rhwp <export-pdf|export-png|export-svg|export-text> <입력> -o <출력> [플래그...]`
//!
//! 환경변수 (모두 선택):
//! - `MOCK_RHWP_STATE_DIR`: 관측 기록 폴더. `active/`(실행 중 표식),
//!   `samples/`(호출별 관측 최대 동시 실행 수), `argv/`(호출별 인자 로그),
//!   `attempts/`(입력·명령별 시도 횟수)를 만든다. 없으면 기록하지 않는다.
//! - `MOCK_RHWP_WAIT_FOR`: 표본을 뜨기 전에 기다릴 최소 동시 실행 수 (기본 0 =
//!   대기 없음). 벽시계 시간에 기대지 않고 "동시에 겹쳤다"를 확정하기 위한
//!   랑데부 장치다 — 상한이 1이면 아무리 기다려도 2가 될 수 없다.
//! - `MOCK_RHWP_WAIT_TIMEOUT_MS`: 위 대기의 상한 (기본 5000).
//! - `MOCK_RHWP_FAIL_MATCH`: 입력 경로에 이 부분 문자열이 포함되면 실패 후보.
//! - `MOCK_RHWP_FAIL_TIMES`: 실패 후보 입력에 대해 (하위 명령별로) 처음 N번의
//!   시도만 실패시키고 그 다음부터 성공한다 (기본: 항상 실패).
//! - `MOCK_RHWP_CONTENT`: 산출 파일 첫 줄에 쓸 내용 표식 (기본 "mock-output").

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn main() {
    // run() 밖에서 exit — run() 안의 ActiveGuard Drop(등록 해제)이 먼저 돈다.
    std::process::exit(run());
}

/// 프로세스 단위 유일 이름 (PID + 단조 증가에 준하는 나노초).
fn uid() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}", std::process::id(), nanos)
}

/// active/ 등록 표식 — 어떤 경로로 끝나든 Drop 에서 해제된다.
struct ActiveGuard {
    path: Option<PathBuf>,
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = fs::remove_file(path);
        }
    }
}

fn count_active(state_dir: &Path) -> usize {
    fs::read_dir(state_dir.join("active"))
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}

fn env_parse<T: std::str::FromStr>(name: &str, default: T) -> T {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn run() -> i32 {
    let args: Vec<String> = env::args().skip(1).collect();

    let Some(subcommand) = args.first().cloned() else {
        eprintln!("mock-rhwp: missing subcommand");
        return 2;
    };

    // 위치 인자(입력)와 -o 값만 해석하고 나머지 플래그는 argv 로그로만 남긴다.
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                output = args.get(i + 1).cloned();
                i += 2;
            }
            arg if arg.starts_with('-') => {
                i += 1;
            }
            other => {
                // 첫 비플래그 토큰이 입력 파일이다 (converter 호출 순서와 동일).
                // 이후 비플래그 토큰은 값을 가진 플래그의 값이므로 무시한다.
                if input.is_none() {
                    input = Some(other.to_string());
                }
                i += 1;
            }
        }
    }

    let (Some(input), Some(output)) = (input, output) else {
        eprintln!("mock-rhwp: usage: mock-rhwp <subcommand> <input> -o <output> [flags...]");
        return 2;
    };

    let state_dir = env::var("MOCK_RHWP_STATE_DIR").ok().map(PathBuf::from);
    let my_uid = uid();
    let mut guard = ActiveGuard { path: None };

    if let Some(state_dir) = &state_dir {
        for sub in ["active", "samples", "argv", "attempts"] {
            let _ = fs::create_dir_all(state_dir.join(sub));
        }

        // 호출 인자 로그 — 한 줄에 인자 하나.
        let _ = fs::write(state_dir.join("argv").join(&my_uid), args.join("\n"));

        // 동시 실행 등록.
        let active_path = state_dir.join("active").join(&my_uid);
        let _ = fs::write(&active_path, b"");
        guard.path = Some(active_path);

        // 동시 실행 표본 — WAIT_FOR 이상을 관측할 때까지(또는 타임아웃까지)
        // 본 최대 동시 실행 수를 기록한다.
        let wait_for: usize = env_parse("MOCK_RHWP_WAIT_FOR", 0);
        let timeout_ms: u64 = env_parse("MOCK_RHWP_WAIT_TIMEOUT_MS", 5000);
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let mut max_seen = count_active(state_dir);
        while max_seen < wait_for && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
            max_seen = max_seen.max(count_active(state_dir));
        }
        let _ = fs::write(
            state_dir.join("samples").join(&my_uid),
            max_seen.to_string(),
        );
    }

    // 실패 시뮬레이션 — FAIL_MATCH 에 걸린 입력은 (명령별) 시도 횟수를 세어
    // 처음 FAIL_TIMES 번 실패한다.
    if let Ok(pattern) = env::var("MOCK_RHWP_FAIL_MATCH") {
        if input.contains(&pattern) {
            let fail_times: u64 = env_parse("MOCK_RHWP_FAIL_TIMES", u64::MAX);
            let attempts_so_far = if let Some(state_dir) = &state_dir {
                let stem = Path::new(&input)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("input");
                let key: String = format!("{}-{}", subcommand, stem)
                    .chars()
                    .map(|c| {
                        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect();
                let counter = state_dir.join("attempts").join(key);
                let n: u64 = fs::read_to_string(&counter)
                    .ok()
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(0);
                let _ = fs::write(&counter, (n + 1).to_string());
                n
            } else {
                0
            };
            if attempts_so_far < fail_times {
                eprintln!("mock-rhwp: simulated failure for {}", input);
                return 1;
            }
        }
    }

    // 성공 경로 — 실제 rhwp 산출 규약을 흉내낸다: export-pdf 는 -o 가 단일
    // 파일(부모 폴더 자동 생성), 나머지는 -o 폴더 안에 페이지 파일을 쓴다
    // (단일 페이지 문서의 실제 이름 규칙인 `<stem>.<ext>` 사용).
    let content = env::var("MOCK_RHWP_CONTENT").unwrap_or_else(|_| "mock-output".to_string());
    let stem = Path::new(&input)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page")
        .to_string();
    let write_result = match subcommand.as_str() {
        "export-pdf" => {
            let out = Path::new(&output);
            if let Some(parent) = out.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }
            fs::write(out, format!("{}\npdf:{}\n", content, input))
        }
        "export-png" | "export-svg" | "export-text" => {
            let ext = match subcommand.as_str() {
                "export-png" => "png",
                "export-svg" => "svg",
                _ => "txt",
            };
            let dir = Path::new(&output);
            let _ = fs::create_dir_all(dir);
            fs::write(
                dir.join(format!("{}.{}", stem, ext)),
                format!("{}\n{}:{}\n", content, subcommand, input),
            )
        }
        other => {
            eprintln!("mock-rhwp: unknown subcommand {}", other);
            return 2;
        }
    };

    if let Err(e) = write_result {
        eprintln!("mock-rhwp: write failed: {}", e);
        return 1;
    }
    0
}
