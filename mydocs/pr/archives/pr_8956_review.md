# PR #8956 리뷰 — 표 조회 ByPath 경로의 마지막 단계 계약

## 최종 판정

**승인 — 코드 검토에서 수정이 필요한 결함은 발견하지 않았으며, 집중 회귀 4건과 필수 로컬 lint·workspace 빌드 묶음이 모두 통과했다.**

2026-11-12 KST에 동일 code head에서 남은 로컬 검증을 완료하여 보류를 해제했다.
GitHub에서 확인한 결과와 reviewer가 직접 실행한 결과는 아래에서 구분한다.
작업지시자는 2026-11-12에 승인 리뷰 게시와 조건 충족 시 병합 진행을 승인했다.
문서 반영 뒤 최신 head의 CI와 mergeability를 다시 확인한 후 승인된 원격 조치를 수행한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#8956](https://github.com/edwardkim/rhwp/pull/8956) |
| 작성자·base | `gitprogam` / `devel` |
| 관련 이슈 | [#8880](https://github.com/edwardkim/rhwp/issues/8880), 원 PR 본문 `Closes #8880` |
| 원 head·검증 code candidate | `2acf7fecfc644a2fc2870c0ef31fb65872657a20` |
| PR base | `4c876654d86d071d342fc78706c51cfff6ddbde4` |
| 병합 대조 devel | `80b1c354c29b7a02fb2b3c14a274da7f5b653ada` |
| Reviewer | `postmelee`, 지정 후 API로 재확인 |
| 규모 | 1 commit, 6 files, +275 / -10 |
| 작성 시점 상태 | 2026-11-12 20:51 KST 조회: open, draft=false, mergeable=true, head 불변 |
| Contributor 수정 허용 | REST API의 `maintainer_can_modify=true` 확인 |
| 검토 경로 | collaborator 외부 PR, 원 source head를 보존한 별도 검토 worktree |

기본 작업공간의 기존 미커밋 문서 3개를 보존하기 위해 작업지시자의 승인을 받아 격리했다.
리베이스·체리픽·source 보정은 하지 않았다. 원 기여 commit의 author와 메시지도 변경하지 않았다.

## 변경과 검토 범위

- `src/document_core/queries/cursor_nav.rs`: 공통 `parse_path_entries`에 마지막 단계가
  표인지 전달한다. `parse_cell_path`는 모든 단계의 세 필드를 계속 요구하고,
  `parse_table_path`만 마지막 `cellIndex`·`cellParaIndex` 생략을 허용한다.
- `src/document_core/helpers.rs`: `json_usize_opt`가 필드 부재와 주어진 값의 파싱 실패를 구분한다.
- `src/document_core/queries/cursor_rect.rs`: 표 차원·표 셀 bbox 조회 두 함수만 새 파서를
  사용한다. 두 native 함수를 공개해 integration test에서 제품 진입점을 호출한다.
- `src/wasm_api.rs` 문서 주석과 [소비자 가이드](../../manual/consumer_edit_api_guide.md)에
  경로 형식, 필드 의미, 중첩 표 예시를 추가한다.
- [회귀 소스](../../../tests/cases/issue_8880_bypath_table_path.rs)는 기존 자동 수집 경로에 등록된다.

소비 경로는 `parse_table_path → resolve_table_by_path → 표 조회`다.
`resolve_table_by_path`는 마지막 단계에서 `controlIndex`만으로 표를 선택하며,
bbox의 `check_table_match`도 마지막 단계의 `controlIndex`만 비교한다.
따라서 생략 필드의 기본값 0은 표 선택·bbox 좌표 계산을 바꾸지 않는다.
중간 단계는 셀·셀 문단을 실제로 통과하므로 기존 필수 필드 규칙을 유지한다.

검토한 최신 devel과 PR base 사이에서 위 helpers·cursor_nav·cursor_rect·wasm_api 파일의
추가 변경은 없었다. 병합 충돌이 없다는 사실과 실제 merge tree의 실행 검증은 구분한다.
현재 merge tree 자체를 빌드한 결과는 없다.

## 검증 입력과 결과

로컬 실행 대상은 모두 원 code head `2acf7fecfc644a2fc2870c0ef31fb65872657a20`이다.
macOS arm64, Rust 1.93.1에서 공용 review target과 `release-test` profile을 사용했다.

| 검사 | 실제 결과 |
| --- | --- |
| `git diff --check 4c876654d86d071d342fc78706c51cfff6ddbde4...HEAD` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `node scripts/rust-test-suite-manifest.mjs --prepare` | 통과, 기존 generated suite에 자동 배정 |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref 4c876654d86d071d342fc78706c51cfff6ddbde4` | 통과 |
| `node scripts/run-rust-test.mjs issue_8880_bypath_table_path -- --cargo-profile release-test --target-dir "$CARGO_TARGET_DIR"` | 4/4 통과, 같은 binary의 범위 밖 253건 제외 |
| `git merge-tree --write-tree 80b1c354c29b7a02fb2b3c14a274da7f5b653ada 2acf7fecfc644a2fc2870c0ef31fb65872657a20` | 종료 코드 0, tree `45044fb87d7f85e2eee24ef116b037c77f9e70bc` |

집중 회귀는 `env -u RHWP_FONT_PATH sandbox-exec -f <격리 프로필>` 아래에서 실행했다.
개인·공유 설치 글꼴 읽기와 개인 Font Registry 조회를 차단했다.
같은 프로필의 `system_profiler SPFontsDataType -json`에서 노출된 글꼴 경로 237개가 모두
`/System/Library/` 아래임을 확인했다. 사용자의 글꼴 설치·등록 상태는 변경하지 않았다.

Nextest run ID는 `b7733d39-f664-4dd5-afe7-eac79b9c83a3`다. 최종 실행은
빌드 3분 47초 뒤 테스트 0.017초에 종료 코드 0으로 끝났다.

| 실제 실행한 테스트 | 보호한 계약 |
| --- | --- |
| `table_path_last_step_needs_only_control_index` | 마지막 필드 생략·전체 지정·무시되는 범위 밖 셀 값의 표 차원 조회 |
| `table_path_still_rejects_invalid_last_step` | controlIndex 누락, 문자열 셀 값, 표가 아닌 컨트롤 거부 |
| `nested_table_path_needs_full_outer_steps` | 중첩 표의 마지막 필드 생략 허용, 앞 단계 필수 필드 유지 |
| `cell_bboxes_accept_control_only_last_step` | 생략 경로와 전체 경로의 bbox 결과 동일, 2×3 표의 6칸 반환 |

임시 실행 기록은 검토 worktree의 `output/pr-review/pr8956/`에 두었다.
전체 checkout 시도는 디스크 부족으로 실패했고 Git이 불완전 worktree를 제거했다.
T7에서 희소 checkout으로 다시 준비했으며, 최초 컴파일은 기존 fixture·내장 문서 누락으로
실패했다. 필요한 추적 파일을 추가한 뒤 위 최종 실행이 통과했다. 이 준비 실패를 PR 코드
결함으로 분류하지 않았고 실패 실행을 통과로 집계하지 않았다.

### 추가 로컬 lint·workspace 빌드 결과

2026-11-12 KST에 Rust 1.93.1, 같은 검토 head와 공용 target에서 다음 명령을 순차 실행했다.
`CARGO_BUILD_JOBS=6`을 사용했고 source·test·Cargo.lock 변경은 없었다.

| 명령 | 최종 결과 | 실행 기록 |
| --- | --- | --- |
| `cargo clippy --locked -- -D warnings` | 종료 코드 0, 54.6초 | `clippy-native.log` |
| `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 종료 코드 0, 56.1초 | `clippy-wasm.log` |
| `cargo build --locked --workspace` | 종료 코드 0, 86.0초 | `workspace-build.log` |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 종료 코드 0, Cargo 출력 15.63초 | `clippy-workspace-final.log` |

실행 기록은 `output/pr-review/pr8956/logs/` 아래 임시 로그다. 각 명령의 target은
`CARGO_TARGET_DIR`로 공용 review target을 지정했다. 전체 target Clippy의 앞선 두 실행은
희소 checkout에 없던 기존 글꼴·Studio 소스·검토 fixture의 `include_bytes!`·`include_str!`
입력 누락으로 종료 코드 101이었다. 추적된 정적 include 입력을 일괄 점검하고 필요한
checkout 범위를 보완한 뒤 최종 실행이 통과했다. 실패 실행은 `clippy-workspace.log`와
`clippy-workspace-retry.log`로 구분하며 PR 코드 결함이나 통과 결과로 집계하지 않았다.
전체 target Clippy는 컴파일·lint 검사이며 제외된 회귀 테스트를 새로 실행한 결과는 아니다.

### CI에서 확인한 범위와 로컬 미실행 항목

[CI run 43038606683](https://github.com/edwardkim/rhwp/actions/runs/43038606683)의
head가 검토 code candidate와 같음을 확인했다. Lint의 Format·Clippy·WASM32 Clippy·workspace 검사,
Frontend package gates의 fresh WASM 빌드, Native Skia tests, Archive A–E 실행과
Build & Test 집계가 실제 success였다. 독립 WASM Build job은 skipped였으며 이를 실행 성공으로
세지 않았다. fresh WASM 빌드는 Frontend package gates의 실행 결과다.

CodeQL `43038606644`, Proptest roundtrip `43038607327`, Adapter inter-diff `43038606669`,
Render Diff `43038606474`도 같은 code head에서 success였다.

- 로컬 전체 release-test와 Native Skia 광범위 재실행은
  [동일 code head Full CI 재사용](../../manual/pr_review_workflow.md#322-녹색-github-code-head의-중복-로컬-전체-회귀-생략)
  조건으로 생략했다. reviewer가 code·test·fixture·workflow를 보정하지 않았고,
  고정한 최신 devel과의 merge simulation이 통과했다.
- [로컬 검증 4.3](../../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)의
  native Clippy, WASM32 Clippy, workspace build, workspace all-target Clippy를 추가로 직접
  실행해 모두 통과했다. 앞선 보류 사유였던 로컬 lint 미실행은 해소됐다.
- reviewer의 fresh WASM 빌드와 JS 재현은 미실행이다. 원 작성자는 WASM 재현을 보고했고
  CI fresh 빌드도 성공했지만, 이를 reviewer의 직접 JS 실행으로 표기하지 않는다.
- 원 작성자가 보고한 음성 대조 3 FAIL / 1 PASS는 reviewer가 재실행하지 않았다.
  로컬에서 직접 확인한 근거는 위 제품 API 집중 회귀 4건이다.

## 조판 원칙·검증 입력 커밋 확인

**조판 원칙: 비해당.** 경로 파싱 계약만 바꾸며 측정·배치·paint·pagination·저장 조판 정보를
수정하지 않는다. bbox 조회도 기존 좌표 계산과 매칭 규칙을 그대로 사용한다.
따라서 Visual Sweep, 전쪽 Native/fresh WASM TSV, handbook gate 및 독립 PDF 확대 판독은
이 변경의 적용 대상이 아니다. 시각 일치를 측정하거나 개선했다고 주장하지 않는다.

**검증 입력 커밋 확인: 비해당.** 대상 테스트는 빈 문서를 API로 만들고 표를 삽입하여
직접 소비한다. 실행 입력으로 외부 HWP/HWPX/PDF를 읽지 않았다. 희소 checkout 보완 때
가져온 다른 회귀의 include_bytes fixture는 같은 binary의 컴파일 의존성이며,
해당 테스트 253건을 실행하거나 이 PR의 수용 증거로 사용하지 않았다.

## 발견 사항과 후속 처리

새로 보고할 코드 결함은 없다. 필수 로컬 lint·workspace 빌드는 모두 통과했다.
reviewer 직접 WASM 실행의 한계는 위와 같으며 CI 빌드·작성자 보고와 구분한다.
전체 JSON 문법 엄격성 개선이나 공통 `렌더링 오류:` 접두어 변경은 이 PR의 해결 범위가 아니다.
이슈의 핵심 요구인 표 경로의 불필요한 마지막 필드 생략과 API 설명 보완은 코드·테스트에서 확인했다.

작업지시자는 정식 검토 문서 작성에 이어 승인 리뷰와 조건 충족 시 병합 진행을 승인했다.
이 문서 commit 시점에는 contributor source push·GitHub review·merge·issue close가 아직 완료되지 않았다. 문서 반영 계획과 원격 게이트는
[사전 판단 보고서](pr_8956_report.md)에 기록한다.
