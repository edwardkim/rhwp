//! [#3918] `rhwp-agent` — 에이전트 운영 실험 표면.
//!
//! 에이전트가 rhwp 를 부려 작업을 완주할 때 반복되는 운영 루프(발견 → 작업 →
//! 사후 검증 → 회귀 감시 → 증빙)의 빈 자리를 채우는 별도 바이너리다.
//!
//! # 왜 별도 바이너리인가
//!
//! 본 CLI(`src/main.rs`)와 그 등록부(capabilities·출처 지도)는 여러 열린 PR 이
//! 동시에 수정하는 최고 경합 지점이다. 이 표면은 `src/bin/rhwp-agent/` 신규
//! 디렉터리로만 서서 **기존 파일을 하나도 만지지 않고**(Cargo 대상 자동 인식,
//! `Cargo.toml` 무변경) 라이브러리 공개 API 만 쓴다. 어떤 열린 PR 과도, 어떤
//! 머지 순서에서도 충돌하지 않는다. 검증된 명령은 본 CLI 로 승격한다(#3918).
//!
//! # 구조 불변식
//!
//! 디스패치·도움말·자기서술(capabilities)은 전부 [`caps::COMMANDS`] 단일
//! 테이블에서 나온다 — "하위 명령 사각" 재발을 구조로 막는다. 미지 명령·미지
//! 플래그는 침묵 무시 없이 exit 2 다(#3884 의 교훈).

mod caps;
mod chunkplan;
mod contextcost;
mod difftext;
mod doctor;
mod envelope;
mod evidence;
mod fingerprint;
mod piiscan;
mod scan;
mod verify;

use envelope::EXIT_USAGE;
use std::process;

fn print_help() {
    crate::outln!(
        "rhwp-agent v{} — 에이전트 운영 실험 표면 (#3918)",
        rhwp::version()
    );
    crate::outln!("사용법: rhwp-agent <명령> [옵션]\n");
    crate::outln!("명령:");
    for c in caps::COMMANDS {
        crate::outln!("  {}", c.usage);
        crate::outln!("      {}", c.summary);
    }
    crate::outln!(
        "\n종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류 · 3 게이트 위반(차이·위반 발견)"
    );
    crate::outln!("자세한 계약: rhwp-agent capabilities --json");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = match args.get(1).map(|s| s.as_str()) {
        Some("--help") | Some("-h") | None => {
            if args.get(1).is_none() {
                // 명령 누락은 사용법 오류다 — 본 CLI 와 같은 계약(#2707).
                eprintln!("오류: 명령을 지정해주세요.");
                eprintln!("사용법: rhwp-agent <명령> [옵션]  ('rhwp-agent --help' 참고)");
                process::exit(EXIT_USAGE);
            }
            print_help();
            0
        }
        Some("--version") | Some("-V") => {
            crate::outln!("rhwp-agent v{}", rhwp::version());
            0
        }
        Some(name) => match caps::find(name) {
            Some(spec) => (spec.handler)(&args[2..]),
            None => {
                eprintln!("오류: 알 수 없는 명령입니다 - {name}");
                if let Some(hint) = envelope::closest(name, caps::COMMANDS.iter().map(|c| c.name)) {
                    eprintln!("힌트: 가장 가까운 명령은 '{hint}' 입니다");
                }
                eprintln!("사용법: rhwp-agent <명령> [옵션]  ('rhwp-agent --help' 참고)");
                EXIT_USAGE
            }
        },
    };
    process::exit(code);
}
