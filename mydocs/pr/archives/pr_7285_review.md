# PR #7285 리뷰 — 빈 누름틀 캐럿 정렬 앵커 복원

## 접수 정보

2026-09-20에 확인한 code candidate의 참고값이다. 이 리뷰 문서를 포함한 최신 head의
상태와 CI는 merge 직전에 다시 확인한다.

| 항목 | 값 |
| --- | --- |
| PR | [#7285](https://github.com/edwardkim/rhwp/pull/7285) |
| 작성자·경로 | `jangster77`, collaborator self-review |
| 관련 이슈 | [#6766](https://github.com/edwardkim/rhwp/issues/6766) |
| base | `devel` / `517df04ba110dc108fbbb203e72229c51819b591` |
| 검토한 code candidate | `ba0e97a7cdcef2c445f03640c77734a11bcaf6f7` |
| 최초 제출 규모 | 4 files, +142 / -0 |
| GitHub 상태 | code candidate에서 Open, non-draft, MERGEABLE / CLEAN. 아래 CI 성공은 이 문서 trailing commit 전의 참고값이며 final head는 다시 확인 필요 |
| reviewer | self-review이므로 지정하지 않음 |

라우팅: `collaborator_self_merge` + `intake_and_review` + `local_validation`.
읽은 정본: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_self_merge.md`, `intake_and_review.md`, `local_validation.md`.

## 변경과 코드 검토

빈 ClickHere가 있는 문단에서 `get_cursor_rect_native`의 빈 문단 빠른 경로가 페이지 tree
탐색을 건너뛰면, renderer가 이미 만든 zero-width 정렬 앵커 대신 TextLine의 본문 좌단을
반환했다. 수정 전 공개 API 재현에서 left/center/right x는 모두 `113.4`였다.

`src/document_core/queries/cursor_rect.rs`는 zero-length `FieldType::ClickHere`만
`para_page_scan_can_hit`에 포함한다. 따라서 해당 필드의 정렬 앵커는 찾되, 일반 빈 문단과
다른 Field는 기존 #4126 빠른 경로를 유지한다. sample명·좌표 상수·clamp로 결과를 맞추는
분기는 추가하지 않았다.

새 `tests/cases/issue_6766_clickhere_caret_alignment.rs`는 실제 Studio와 같은
`apply_para_format_native`·`insert_click_here_field_at` 경로로 세 문단을 만들고,
`left < center < right`를 검사한다. 수정 전에는 실패하고 수정 후 통과했다. 테스트 source는
정식 `tests/cases/`에 있으며 generated suite와 manifest는 커밋하지 않았다.

## 조판 원칙 준수 검토

| 검토 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 빈 zero-length ClickHere라는 IR 조건만 사용하며, 특정 문서·좌표·폰트 조건이 없다. |
| 측정·배치 일관성 | 충족 | renderer가 생성한 정렬 앵커를 cursor rectangle 조회가 직접 소비한다. TextLine 좌단 fallback은 앵커를 찾지 못할 때만 남는다. |
| 분할·이어받기 계약 | 비해당 | pagination, table fragment, reservation 경로를 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 비해당 | 줄 구성·높이·저장 LineSeg를 바꾸지 않고 기존 빈 문단 앵커의 조회 여부만 바꾼다. |
| 사례와 증거의 독립성 | 충족 | 수정 전 동일 public API에서 세 x가 같음을 확인했고, 수정 후 정렬 단조성 및 #4126 대조군을 실행했다. |
| 기준값 변경 | 비해당 | baseline, golden, 허용치를 바꾸지 않았다. |
| 주장과 검증 범위 | 충족 | focused, 기존 field, 전체 release-test, native/WASM lint 및 실제 WASM 번들 결과를 아래에 기록했다. |

## 검증과 입력 공급

| 검증 | 실제 결과 |
| --- | --- |
| #6766 새 회귀 | `regression_suite_004`: 1 passed / 198 skipped |
| 빈 문단 대조군 | `issue_4126_cursor_rect_empty_para_pages`: 1 passed |
| 기존 ClickHere | `issue_258_clickhere_form_mode`: 13 passed / 184 skipped |
| 전체 Rust 회귀 | `cargo nextest run --locked --cargo-profile release-test --no-fail-fast`: 10,098 passed / 50 skipped / 18 slow / 977.504초 |
| Rust quality gates | fmt, native clippy, WASM32 clippy, workspace build, workspace all-target clippy 통과 |
| suite 정책 | `rust-test-suite-manifest --prepare` 및 base `517df04b` 대비 `--check` 통과 |
| WASM | `rhwp-wasm-build` 통과. `pkg` build 산출물은 추적하지 않음 |
| GitHub code CI | [CI run 35507439658](https://github.com/edwardkim/rhwp/actions/runs/35507439658) 성공. Lint, 4개 archive build·test shard 및 Build & Test 성공; Rust CodeQL도 성공 |

1단계 뒤 test source를 `tests/cases/`로 옮긴 최종 code candidate에서는 suite check, 새 focused
회귀 및 모든 Rust quality gate를 `CARGO_BUILD_JOBS=8`로 다시 통과했다. source 내용과 제품
코드는 바뀌지 않아 전체 10,098개 실행은 1단계 결과를 재사용한다. 최신 final head의 GitHub CI가
전체 범위를 다시 확인해야 한다.

검증 입력은 코드가 빈 문서와 ClickHere를 programmatically 생성한다. HWP/HWPX/PDF 파일 또는
기준 PDF를 사용하지 않았으므로 검증 입력 커밋 확인은 **비해당**이다. renderer/layout/paint 또는
인쇄 조판 출력을 바꾸지 않는 caret rectangle query 변경이므로 PDF Visual Sweep도 **비해당**이다.

## 최종 판정

- 판정: **승인**
- 근거: 수정 전 결함을 검출한 public-API 회귀, 수정 후 정렬 불변식, 빈 문단 대조군과 기존
  ClickHere 계약, 전체 release-test 및 Rust/WASM 품질 검증을 확인했다. code candidate의 GitHub
  CI도 성공했고, 추가 코드 보정이 필요한 문제는 발견하지 못했다.
- merge 전 조건: 이 review·오늘할일 trailing commit을 포함한 최신 PR head의 required CI 통과,
  mergeable/CLEAN 재확인, 작업지시자의 merge 승인.
- 이 문서는 self-review 기록이며 GitHub APPROVE, issue close, comment 또는 merge를 실행하지 않는다.
  소형 단일 PR이므로 별도 `review_impl`은 만들지 않는다.
