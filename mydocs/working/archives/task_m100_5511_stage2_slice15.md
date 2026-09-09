# #5511 Stage 2 열다섯 번째 수직 절편 — unicode 보안 조회 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `d89aaf787`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 경계 판단

열다섯 번째 이동 대상으로 `inspect unicode`를 선택했다. 이 명령은 문서의 본문·표 셀·
텍스트 상자·수식 script를 한 번 순회하며 제로폭 문자, 방향 제어, 태그 문자와 동형자를
보고하는 read-only query다. handler와 `inspect_unicode_scan_unit` helper는 같은 탐지 결과와
위치 표기를 구성하므로 둘을 하나의 수직 경계로 이동했다.

`unicode_deception_contract`의 14개 테스트는 네 탐지축, HWP5·HWPX 기록 경로, 필터, 중첩
위치, MCP·help 선언, 오류 출력, 원본 무변경, 정상 한국 문서 오탐과 선형 비용을 보호한다.
이동 전 handler와 helper 모두 cognitive complexity 25 초과 경고가 없었고, 기존
`security_inspection.rs`와 합친 뒤에도 382줄로 모듈 상한 1,200줄 이하이다.

절편 시작·종료 시 `upstream/devel`은 `1a6ce79fd`로 동일했다. 활성 PR 중 `src/main.rs`,
`src/cli/queries/security_inspection.rs`, query module index, unicode·CLI catalog 계약과 이
보고서에 겹치는 변경은 없었다.

## 2. 구현 결과와 보호 불변식

- `src/cli/queries/security_inspection.rs`가 `inspect_unicode`와 전용 scan helper를 소유한다.
- `src/main.rs`의 inspect router는 새 query 모듈 API만 호출한다.
- 공개 함수 표식과 경계의 빈 줄만 정규화한 기계 비교에서 이동 블록이 byte 단위로 일치했다.
- `cli_catalog_contract`가 handler·helper 소유권, root 재유입 금지와 dispatch를 고정한다.
- 탐지축, Control 순회 순서, location 문자열, severity·kind 집계, provenance, format·암호
  처리, exit code와 stdout/stderr 분리를 바꾸지 않았다.

이번 절편은 기존 `load_document_core` seam을 그대로 사용했다. service layer 이행이나 탐지
알고리즘 변경을 물리 이동에 섞지 않았으며, 이는 Stage 3의 의존성 역전 대상으로 남긴다.

## 3. 지표 변화

| 항목 | Stage 2 절편 14 | Stage 2 절편 15 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 39,502 | 39,239 | -263 |
| `src/cli/queries/security_inspection.rs` | 118 | 382 | +264, 모듈 상한 이하 |
| `main.rs` plain 최상위 함수 | 319 | 317 | handler·helper 각 1개 이동 |
| 누적 이동 read-only handler | 18 | 19 | 1개 추가 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | 변화 없음 |
| security inspection 모듈 CC>25 함수 | 0 | 0 | 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 62 | Control 순회 참조 이동 |
| `rhwp::parser` 직접 참조 | 72 | 72 | 변화 없음 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

## 4. 외부 동작 동등성

열네 번째 절편 통합 바이너리와 이동 후 바이너리에 대해 다음 스물한 경로의 exit code와
stdout/stderr byte 수·SHA-256을 비교했다.

1. 정상 HWP5 사람용 결과
2. 정상 HWP5 JSON 봉투
3. 정상 HWPX JSON 봉투
4. 정상 HWP3 JSON 봉투
5. 네 탐지축을 가진 HWP5 사람용 결과
6. 네 탐지축을 가진 HWP5 JSON 봉투
7. 같은 탐지축을 HWPX로 변환한 JSON 봉투
8. `--kind zero-width`
9. `--kind bidi`
10. `--kind tag`
11. `--kind confusable`
12. `--kind all`
13. 필수 경로 누락
14. kind 값 누락
15. 알 수 없는 kind
16. 알 수 없는 옵션
17. 위치 인자 초과
18. 존재하지 않는 파일
19. 암호 문서에 비밀번호 없음
20. 암호 문서에 잘못된 비밀번호
21. 암호 문서에 올바른 비밀번호

스물한 경로 모두 byte 단위로 일치했다. 정상 HWP5 사람용 결과 106 bytes의 SHA-256은
`4bc9dedefcac631a0943c418888d6eb9283d72eac894b22b13ac49a62972b02e`, 정상 HWP5 JSON
312 bytes는 `2716bb344591caea729c91c078ba848a048bd8befd709cdf09138d3939e4cd13`이다. 네 축을
가진 HWP5 JSON 3,707 bytes는
`d7a3378816182aa0302b7a994fe9df0ef25310030161ceac7ead42a023f8c82e`, 같은 내용의 실제
HWPX JSON 3,713 bytes는
`4dc9dd88b2d9f3ac2660a90a30a5438e46b4487d3216066953b4d1c046a67b34`로 일치했다.
올바른 비밀번호 경로 322 bytes의 hash도
`4b0201d85541ae36698f6f00e88f103f27b110e9cc888885bfdec15e0df1d69d`로 동일했다.
오류 경로는 stdout이 비어 있으며 기존 exit 1/2와 stderr hash를 그대로 유지했다.

계측용 공격 문서는 정상 샘플에서 실행 중 생성하고 HWPX는 실제 변환 명령으로 만들었다. 전후
비교가 끝난 뒤 해당 임시 파일과 디렉터리를 모두 제거했으며 저장소에는 추가하지 않았다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 `unicode_deception_contract` focused nextest | 각각 14/14 통과 |
| `cli_catalog_contract` | 이동 전 17/17, 이동 후 18/18 통과 |
| 사람용·JSON·format·filter·암호·오류 출력 hash equivalence | 21/21 일치 |
| release-test 전체 nextest | 7,748/7,748 통과, 3 slow, 38 skipped, 161.376초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| Rust test suite manifest | 통과, 750 sources / 3,699 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| test policy 자체 계약 | integration 16/16, unit tier 12/12 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo가 최신 원격에서 이름이 인접한 두 verifier package의 lockfile 순서만
재정렬했으나 #5511과 무관한 파생 변화이므로 추적 변경에서 복원했다. parser·탐지 알고리즘·
조판 로직은 바꾸지 않고 adapter 위치만 옮겼으므로 시각 검증과 WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. 다음 절편 관문

다음 보안 조회 후보는 `inspect watermark`와 전용 scan helper다. 현재 이름에 직접 대응하는
독립 integration contract가 검색되지 않으므로 이동 전에 capabilities·MCP 및 탐지축별 양성·음성,
중첩 위치, 오류·암호·무변경 경로가 다른 계약에 충분히 고정됐는지 먼저 조사한다. 보호가 부족하면
characterization test를 먼저 추가하고 handler 이동과 같은 변경 단위에 섞지 않는다.

`inspect injection`은 별도의 대형 계약과 live MCP tool registry 의존을 가지므로 watermark와
묶지 않는다. 다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전
수행하지 않는다.
