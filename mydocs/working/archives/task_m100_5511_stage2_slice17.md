# #5511 Stage 2 열일곱 번째 수직 절편 — watermark 보안 조회 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `77b14ead5`
- 수행일: 2026-08-19
- 상태: 구현 완료 — branch-wide PR 제출 정책 정상화 승인 대기

## 1. 절편 선정과 경계 판단

열일곱 번째 이동 대상으로 앞 절편에서 전용 CLI 계약을 고정한 `inspect watermark`를
선택했다. handler와 `inspect_watermark_scan_unit` helper는 세 탐지축의 결과를 같은 JSON
항목으로 구성하고 같은 순회에서 호출되므로 하나의 수직 경계로 이동했다.

이동 경계는 helper 시작부터 handler의 `EXIT_OK`까지이다. 바로 뒤의
`mcp_tool_name_registry`는 injection·armor가 함께 쓰는 root 공유 등록부이므로 이동 대상에서
제외했다. 최초 기계 추출에서 이 등록부의 설명 주석이 경계에 인접해 함께 잡힌 것을 비교 검사로
발견했고, 구현 커밋 전에 주석과 등록부를 원래 root 소유 위치에 다시 결합했다. 최종 기계 비교는
handler의 `pub(crate)` 표식만 정규화한 뒤 원문과 byte 단위로 일치했다.

절편 시작 시 `upstream/devel`은 `1a6ce79fd`였고 작업 브랜치는 40커밋 앞선 깨끗한 상태였다.
활성 PR 중 `src/main.rs`, `src/cli/queries/security_inspection.rs`, watermark 계약,
CLI catalog 계약과 이 보고서에 겹치는 변경은 없었다.

## 2. 구현 결과와 보호 불변식

- `src/cli/queries/security_inspection.rs`가 `inspect_watermark`와 전용 scan helper를 소유한다.
- `src/main.rs`의 inspect router는 새 query 모듈 API만 호출한다.
- `cli_catalog_contract`가 handler·helper 소유권, helper의 비공개 범위, root 재유입 금지와
  dispatch 경로를 고정한다.
- 탐지 코어 `stego_scan`, 임계값, HWP/HWPX 순회 순서, nested location, 집계, provenance,
  사람용·JSON 출력, 암호 처리와 exit code를 바꾸지 않았다.
- `mcp_tool_name_registry`, `injection_scan_scopes`, injection·armor consumer는 root에 그대로
  남겼다. 공유 seam의 재설계는 다음 물리 이동과 섞지 않는다.

이번 절편은 기존 `load_document_core` seam과 `Control` 순회를 그대로 옮긴 move-only 작업이다.
service layer 이행과 정상 공공문서의 후행 공백 오탐 정책은 별도 이슈·계획 대상으로 남긴다.

## 3. 지표 변화

| 항목 | Stage 2 절편 16 | Stage 2 절편 17 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 39,239 | 38,962 | -277 |
| `src/cli/queries/security_inspection.rs` | 382 | 659 | +277, 모듈 상한 이하 |
| `main.rs` plain 최상위 함수 | 317 | 315 | handler·helper 각 1개 이동 |
| 누적 이동 read-only handler | 19 | 20 | 1개 추가 |
| CLI catalog 계약 | 18 | 19 | watermark 소유권 1개 추가 |
| Rust test source | 751 | 751 | 변화 없음 |
| static test attribute | 3,707 | 3,708 | +1 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | 변화 없음 |
| security inspection 모듈 CC>25 함수 | 0 | 0 | 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 62 | 61 | Control 순회 참조 이동 |
| `rhwp::parser` 직접 참조 | 72 | 72 | 변화 없음 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

## 4. 외부 동작 동등성

열여섯 번째 절편 바이너리와 이동 후 바이너리에 대해 다음 스물한 경로의 exit code와
stdout/stderr byte 수·SHA-256을 비교했다.

1. 정상 HWP5 사람용 결과
2. 정상 HWP5 JSON 봉투
3. 정상 HWP3 JSON 봉투
4. hidden·homoglyph·whitespace를 가진 HWP5 사람용 결과
5. 같은 HWP5 JSON 봉투
6. 실제 변환한 HWPX JSON 봉투
7. `--kind hidden`
8. `--kind homoglyph`
9. `--kind whitespace`
10. `--kind all`
11. 공식 편람 HWP JSON
12. 공식 편람 HWPX JSON
13. 필수 경로 누락
14. kind 값 누락
15. 알 수 없는 kind
16. 알 수 없는 옵션
17. 위치 인자 초과
18. 존재하지 않는 파일
19. 암호 문서에 비밀번호 없음
20. 암호 문서에 잘못된 비밀번호
21. 암호 문서에 올바른 비밀번호

스물한 경로 모두 byte 단위로 일치했다. 정상 HWP5 사람용 결과 100 bytes의 SHA-256은
`c1232145eafa189d47f4d0ca075e17ebb506f8c7650782ceb75a859676e9cf6a`, 세 축을 가진
HWP5 JSON 2,065 bytes는
`bc219d564afc297133a57c84a8ddd7ba33a9d792f64e6e28c2b5472593122a24`, 같은 내용의 실제
HWPX JSON 2,066 bytes는
`44e0d13da7c6bb07b7ed44f84a2ee58f302f328761510f33e8d20263ebf30830`로 일치했다.
올바른 비밀번호 경로 306 bytes의 hash도
`4465108f7c93ea70dc1447c2874ec7f0f531b8f4c157effbf1585f9fcfebc863`로 동일했다. 오류
경로는 stdout이 비어 있으며 기존 exit 1/2와 stderr hash를 그대로 유지했다.

공식 `2025 행정업무운영 편람(최종)` HWP와 HWPX도 각각 22건, medium 15·low 7,
whitespace 22라는 현재 결과와 각각의 출력 hash를 그대로 유지했다. 이는 이동 동등성 증거일
뿐 장기적으로 올바른 clean 판정이라는 승인이 아니다. 후행 공백 오탐 개선은 별도 후속 이슈로
처리한다.

계측용 공격 문서는 정상 표본에서 실행 중 생성하고 HWPX는 실제 변환 명령으로 만들었다. 전후
비교가 끝난 뒤 임시 HWP/HWPX와 디렉터리를 모두 제거했으며 저장소에 추가하지 않았다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 `watermark_inspection_contract` focused nextest | 각각 8/8 통과 |
| `cli_catalog_contract` | 이동 전 18/18, 이동 후 19/19 통과 |
| 정상·세 축·filter·공식 표본·암호·오류 출력 hash equivalence | 21/21 일치 |
| release-test 전체 nextest | 7,757/7,757 통과, 4 slow, 38 skipped, 164.618초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| Rust test suite manifest 현재 상태 | 통과, 751 sources / 3,708 static test attrs / 43 integration targets |
| Rust unit tier 현재 상태 | 통과, 4,225 tests / 298 modules |
| test policy 자체 계약 | integration 16/16, unit tier 12/12 통과 |
| CI impact Node 계약 | classifier 31/31, policy 31/31 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo가 이름이 인접한 두 verifier package의 lockfile 순서만 재정렬했으나
#5511과 무관한 파생 변화이므로 추적 변경에서 복원했다. parser·탐지 알고리즘·조판 로직은
바꾸지 않고 adapter 위치만 옮겼으므로 시각 검증과 WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. branch-wide PR 제출 정책 발견 사항

현재 생성 상태만 검사하는 manifest·unit-tier gate와 그 자체 계약은 모두 통과했다. 그러나 이번에
추가로 실행한 `--base-ref upstream/devel` 제출 모드는 다음 두 branch-wide 차이를 거부했다.

- Stage 1에서 최상위에 추가된 `tests/cli_catalog_contract.rs`
- 같은 시점에 커밋된 `Cargo.toml`의 generated test target 블록

두 차이는 이번 절편 이전 `HEAD^`에도 각각 신규 542줄과 generated block 8줄로 이미 존재해
watermark 이동이 만든 회귀는 아니다. 하지만 PR 전체 제출 관점에서는 기존 절편 보고서들이
현재 상태 검사만 통과한 것을 PR-base 정책 통과로 확대 해석하면 안 된다. `cli_catalog_contract`를
`tests/cases/` 원본 경로로 옮기고 include 경로를 조정하며, Cargo generated block은 커밋에서
제거한 뒤 base-aware gate를 다시 통과시켜야 한다.

## 7. 다음 절편 관문

다음 절편은 기능 이동을 잠시 멈추고 Stage 1에서 누적된 integration source·Cargo 파생물의
PR 제출 구조를 정상화하는 최소 정정 절편이어야 한다. 이 정정은 테스트 의미나 제품 동작을
바꾸지 않고 source 위치·include 경로·파생물 비추적만 다룬다. base-aware manifest와 unit-tier,
전체 회귀까지 통과한 뒤에야 `inspect injection` 이동을 검토한다.

그 다음 기능 후보인 injection은 handler가 공유 `mcp_tool_name_registry`와
`injection_scan_scopes`에 의존하고, 같은 helper를 armor도 사용한다. 따라서 등록부나 scope helper를
handler와 함께 성급히 옮기지 않고, 먼저 기존 `injection_scan_contract`가 CLI·MCP·암호·출력과
공유 consumer 경계를 충분히 보호하는지 별도 조사한다. 다음 절편은 메인테이너 승인 전 시작하지
않으며 remote push도 별도 승인 전 수행하지 않는다.
