# task_m100_3263 처리결과 보고서 — `capabilities` 도구 자기서술

- **이슈**: [#3263](https://github.com/edwardkim/rhwp/issues/3263) (#2659 Stage 2-④)
- **브랜치**: `pr/task-capabilities` (PR #3262 위에 적층)
- **범위**: `src/main.rs`, `tests/cli_json_contract.rs`, `mydocs/manual/cli_commands.md`,
  `mydocs/manual/cli_json_pipeline_guide.md`
- **분류**: 기능 추가 (조회 기계화 — 도구 자기서술)

## 1. 문제

에이전트가 rhwp 를 발견할 기계적 입구가 없다. `--help` 는 사람용 한국어 산문이라
에이전트는 파싱하거나 플래그를 추측(환각)하며, 함수 호출·MCP 프레임워크가 전제하는
도구 자기서술이 비어 있다. 문서에서 손으로 옮긴 통합은 명령 추가 시 조용히 낡는다.

## 2. 분석 — 설계 결정

- **정적 테이블 + 드리프트 가드**: 디스패치를 런타임 열거할 수 없으므로 정적 테이블로
  서술하되, 낡음을 두 방향에서 테스트로 고정한다 — ① `--help` 에 등장하는 모든 명령이
  capabilities 에 존재(“help 에만 추가” 차단), ② `version` 이 `--version` 과 동일 원천
  (`rhwp::version()`) 임을 실행 대조.
- **계약 명령 상세 서술**: `--json` 축(info/export-text/export-structure/batch)은
  `json:true`·`flags`·`recordFields` 까지 — 에이전트 도구 정의(함수 스키마·MCP 도구 목록)를
  여기서 자동 생성할 수 있는 수준. 나머지 41개 명령은 name·category·summary 로 전량 수록.
- **출력 규약 일관**: stdout 순수 JSON 한 덩어리, `schemaVersion` 포함 — 조회 계약 그대로.
- Stage 5 MCP 서버(#3140)는 이 자기서술을 소비하는 얇은 래퍼가 된다 — 관문 앞의 기반 작업.

## 3. 변경

- `show_capabilities()` — 명령 45개 전량 수록(export 13·query 2·batch 1·diagnostic 24·internal 5),
  `formats`/`exitCodes`/`jsonContract`/`batch` 계약 블록 포함
- 디스패치 arm·`--help` 등재, `cli_commands.md`·가이드(시나리오 0: 에이전트 온보딩) 갱신

## 4. 검증

- **red→green**: 계약 테스트 3종 신설 — 스키마·계약 명령 `json:true` 표시,
  version↔`--version` 실행 대조, help↔capabilities 드리프트 가드
- 기존 계약 15·`cli_exit_codes.rs` 10 무회귀 (합계 28 green, release)
- `cargo clippy --release --bin rhwp -- -D warnings`·`rustfmt --check`·문서 검사 스크립트 clean
- 실측: `rhwp capabilities | jq` 로 machine-readable 명령 5종·batch 3축 발견 확인 (가이드 예시 실행 검증)

## 4-2. 추가 — `--mcp` 도구 정의 생성 (2차 커밋)

로드맵 Stage 5 의 MCP 서버는 별도 저장소(#227)지만, **그 서버가 도구 목록·입력 스키마를
손으로 옮겨 적으면 rhwp 가 바뀔 때마다 조용히 낡는다.** 원천을 도구 자신이 내도록 했다.

- `capabilities --mcp`: `{schemaVersion, protocol:"mcp", server, invocation, tools[]}`.
  각 도구는 MCP 필수 3종(`name`/`description`/`inputSchema`)에 **실행 배선**(`cli.command`·
  `cli.args` 자리표시자)과 `outputFields` 를 더해, 서버가 파싱 없이 등록·호출할 수 있다.
- 현재 5개 도구: `hwp_info`·`hwp_export_text`·`hwp_export_structure`·`hwp_ir_diff`·`hwp_batch`
  (batch 는 stdin 입력이라 `invocation.stdinTools` 로 명시).
- **드리프트 가드 ③** 추가: `--json` 계약 명령인데 MCP 도구로 안 나오면 테스트 실패
  (`capabilities_mcp_covers_every_json_command`). 계약 축이 늘 때 MCP 가 뒤처지지 못한다.
- 계약 테스트 2종 신설 red→green (MCP 스키마 필수 필드·이름 안전 문자·path 필수 선언).

## 5. 남긴 것

- 명령별 인자 상세(전 명령)·JSON Schema 파일 산출은 소비 수요(MCP 서버 설계) 확정 후 확장
- 역방향 가드(디스패치에만 있고 capabilities 에 없는 명령)는 디스패치 정적 열거가 필요해 보류
