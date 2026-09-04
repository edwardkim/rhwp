# #5511 Stage 2 열네 번째 수직 절편 — hidden-text 보안 조회 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `4dcef6727`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 경계 판단

열네 번째 이동 대상으로 `inspect hidden-text`를 선택했다. 이 명령은 문서를 읽어 기존 은닉
텍스트 탐지 결과를 사람용 또는 JSON 봉투로 내보내는 read-only query다. 약 110줄의 독립
handler이고 전용 helper가 없으며, 24개의 integration contract가 출력·형식·암호·오류 경로를
보호한다. 이동 전 cognitive complexity 25 초과 경고도 없어 물리 분리만으로 닫을 수 있는 최소
수직 절편이었다.

함께 조사한 `inspect unicode`는 약 260줄의 handler와 별도 scan helper를 가지며,
`load_document_core`와 중첩 Control 순회를 사용하고 14개의 독립 계약을 가진다. 두 명령을 한
절편에 묶으면 보안 조회라는 이름만 같을 뿐 서로 다른 traversal·출력 불변식을 동시에 바꾸게
된다. 따라서 이번에는 `hidden-text`만 `security_inspection` 경계로 옮기고 unicode는 다음 후보로
유보했다.

절편 시작·종료 시 `upstream/devel`은 `1a6ce79fd`로 동일했다. 활성 PR 중 `src/main.rs`,
`src/cli/queries/security_inspection.rs`, query module index, hidden-text·unicode·CLI catalog
계약과 이 보고서에 겹치는 변경은 없었다.

## 2. 구현 결과와 보호 불변식

- 새 `src/cli/queries/security_inspection.rs`가 `inspect_hidden_text` CLI adapter를 소유한다.
- `src/main.rs`의 inspect router는 새 query 모듈 API만 호출한다.
- 공개 함수 표식만 정규화한 기계 비교에서 이동 블록이 byte 단위로 일치했다.
- `cli_catalog_contract`가 handler 소유권, root 재유입 금지와 dispatch를 고정한다.
- 은닉 텍스트 판정, off-page 포함 정책, 임계값, provenance, format label, 암호 처리, exit code와
  stdout/stderr 분리를 바꾸지 않았다.

`security_inspection`이라는 이름은 이후 보안 조회 adapter가 공유할 수 있는 경계를 마련하지만,
이번 절편은 unicode나 watermark를 선제적으로 합치지 않는다. 각 명령의 helper·traversal·계약을
별도로 조사한 뒤 모듈 응집도와 1,200줄 상한을 동시에 만족할 때만 같은 경계를 사용한다.

## 3. 지표 변화

| 항목 | Stage 2 절편 13 | Stage 2 절편 14 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 39,613 | 39,502 | -111 |
| `src/cli/queries/security_inspection.rs` | 없음 | 118 | 신규, 모듈 상한 이하 |
| `main.rs` plain 최상위 함수 | 320 | 319 | handler 1개 이동 |
| 누적 이동 read-only handler | 17 | 18 | 1개 추가 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | 변화 없음 |
| security inspection 모듈 CC>25 함수 | 없음 | 0 | 새 경계 내 초과 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 63 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 72 | 72 | 변화 없음 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

이번 절편은 hidden-text CLI adapter만 책임 경계 안에 두었다. 탐지 알고리즘·문서 모델·parser·
renderer·service 경계에는 손대지 않았으므로 복잡도를 다른 파일로 숨기거나 동작 변경을 섞지
않았다.

## 4. 외부 동작 동등성

열세 번째 절편 통합 바이너리와 이동 후 바이너리에 대해 다음 열여섯 경로의 exit code와
stdout/stderr byte 수·SHA-256을 비교했다.

1. HML 사람용 결과
2. HML JSON 봉투
3. HWP5 JSON 봉투
4. HWPX JSON 봉투
5. HWP3 JSON 봉투
6. `--include-offpage`
7. `--threshold 0.5`
8. 존재하지 않는 파일
9. 필수 경로 누락
10. 위치 인자 초과
11. 알 수 없는 옵션
12. threshold 값 누락
13. 허용 범위를 넘는 threshold
14. 암호 문서에 비밀번호 없음
15. 암호 문서에 잘못된 비밀번호
16. 암호 문서에 올바른 비밀번호

열여섯 경로 모두 byte 단위로 일치했다. HML 사람용 결과 72 bytes의 SHA-256은
`bc71a7b28310075de5a9fe8f944734d820d970226f77c759654ef9edae92c0f4`, HML JSON
204 bytes는 `ae290d4b8e758e6c9ffd6e70abbe6c0d6be4bf986cf4b55279c53c4abf136891`, HWPX
JSON 203 bytes는 `e1b39501928be6fd10cedadfd4c35bb31ad49dd58cb121b544008f4fcc25a5e4`다.
올바른 비밀번호 경로 223 bytes의 hash는
`8fb827992e73d2b04c5f083ad8d0efef33b3ef0aadab9b1919e48f6708894b03`으로 이동 전후
일치했다. 오류 경로는 stdout이 비어 있으며 기존 exit 1/2와 stderr hash를 그대로 유지했다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 `hidden_text_contract` focused nextest | 각각 24/24 통과, 90 skipped |
| `cli_catalog_contract` | 이동 전 16/16, 이동 후 17/17 통과 |
| 사람용·JSON·format·option·암호·오류 출력 hash equivalence | 16/16 일치 |
| release-test 전체 nextest | 7,747/7,747 통과, 3 slow, 38 skipped, 161.247초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| Rust test suite manifest | 통과, 750 sources / 3,698 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| test policy 자체 계약 | integration 16/16, unit tier 12/12 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo가 최신 원격에서 이름이 인접한 두 verifier package의 lockfile 순서만
재정렬했으나 #5511과 무관한 파생 변화이므로 추적 변경에서 복원했다. 보안 탐지·parser·구조
추론·조판 로직은 바꾸지 않고 adapter 위치만 옮겼으므로 시각 검증과 WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. 다음 절편 관문

다음 후보는 `inspect unicode` handler와 전용 scan helper다. 이동 전에 14개 계약의 보호 범위,
중첩 Control traversal, `load_document_core` 의존성과 cognitive complexity를 다시 측정한다.
watermark 등 다른 보안 조회는 이름만으로 함께 묶지 않고 별도 조사 후 최소 경계를 선정한다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
