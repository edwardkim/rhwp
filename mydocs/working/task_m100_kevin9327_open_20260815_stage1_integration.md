# Task M100 Kevin9327 공개 PR 통합 검토 Stage 1 - 최신 head 정합과 보정

## 목적

`kevin9327`의 공개 PR #4818부터 #4878까지를 최신 `upstream/devel` 위 로컬 통합 브랜치에
누적한 뒤, 반영 시점 이후 원 PR에 추가 push가 있는지 확인하고 즉시 드러난 유지보수 결함을
고정한다.

## 원격 head 확인

2026-08-16 KST에 열린 PR의 `headRefOid`를 다시 조회했다. #4818부터 #4878까지는 로컬에
반영한 최신 head와 모두 일치했고, #4877(`fc6424982d2f`)와 #4878(`b3fee0bede8a`) 이후에도
추가 push는 없었다.

## 보정 내용

1. DSEL 도입으로 `ast.rs`와 `eval.rs`가 1,000줄을 넘어, 오타 제안 로직을 `suggest.rs`로,
   평가기 단위 테스트를 `eval_tests.rs`로 기계 분리했다. 공개 API와 테스트 의미는 유지했고
   두 생산 코드 파일은 각각 928줄과 741줄이 됐다.
2. `threat-scan` 레코드 fixture의 `0u32 << 10` 항등 연산 두 곳을 같은 비트값의 간결한 식으로
   바꿔 `-D warnings` clippy 게이트를 통과하게 했다.
3. `tools/gen_agent_codex.py`로 생성 대전을 갱신했다. 실제 명령 표면은 89개이며 생성기
   `--check`가 변경 0으로 확인했다.
4. 이번 반영에서 포맷되지 않은 threat-scan 소스의 Rust 표준 줄바꿈을 적용했다.

## 검증 결과

- `cargo test --profile release-test --target-dir target/pr-review --lib agent::dsel`
  - 116 passed
- `cargo test --profile release-test --target-dir target/pr-review --test threat_scan_contract`
  - 9 passed
- `RHWP_BIN=target/pr-review/release-test/rhwp python3 tools/gen_agent_codex.py --check`
  - 변경 0
- `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings`
  - passed

## 다음 단계

Stage 2에서 이 커밋을 기준으로 전체 `nextest` 회귀와 필요한 기능 경계 검증을 실행하고,
개별 PR 검토 기록을 작성한다.
