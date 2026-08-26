---
kind: review
status: self-review-ci-pending
pr: 4742
issue: 3211
author: jangster77
base: devel
---

# PR #4742 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4742](https://github.com/edwardkim/rhwp/pull/4742) |
| 작성자 | `jangster77` (collaborator) |
| 관련 이슈 | [#3211](https://github.com/edwardkim/rhwp/issues/3211) (`Refs`, 자동 종료 아님) |
| head / base | `task_m100_3211` / `devel` |
| code candidate | `494e1317e6d49fa074736946dadea0bf004411d0` |
| 구현 전용 변경 | 2 files, +163 / -8 |
| 문서 작성 전 PR head | `502b0f8867d4757c0812df177e60195f56217d68` |
| 작성 시점 참고 상태 | `BLOCKED`; CI·CodeQL·Render Diff preflight queued, WASM Build skipped |
| 검토 방식 | 작업지시자 지시에 따른 `jangster77` collaborator 셀프 검토; external reviewer 요청 없음 |

### 적용 절차

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, local_validation.md,
visual_fixture_evidence.md, docs_and_git_workflow.md
```

최신 `upstream/devel` `d871bb8ce1c05e92543c4f5932c51e101280a301` 위에서 작업을 시작해
원본 `upstream`의 `task_m100_3211`으로 게시했다. PR 생성 뒤 이 review 기록과 오늘할일을
동일 source branch의 docs-only 후속 commit으로 추가한다. merge 직전에는 이 문서를 포함한
최신 head의 required check와 mergeability를 다시 확인해야 한다.

## 문제 분석과 변경 검토

Windows Hancom Office 2022에서 두 #3211 HWP 샘플을 단일 COM worker로 열었을 때, 둘 다
23쪽·468문단과 동일한 페이지 지문을 보였다. 저장 `PARA_LINE_SEG`를 제거한 비캐시 재조판은
수식·그림의 폭을 누락해 줄 수 일치를 139/165까지 낮췄다.

`FlowInlineControl`은 문자처럼 취급되는 picture, shape, table, equation, form의 원본 HWPUNIT
폭·높이와 문단 문자 위치를 수집한다. 토큰 폭과 character-level fallback 폭에 그 값을 반영하고,
각 제어문을 포함한 줄에만 최대 높이를 적용한다. 종전처럼 문단의 모든 인라인 높이를 첫 줄에
합치는 동작은 흐름 제어문이 있을 때 제거했다.

저장 LineSeg를 지운 비교 하니스는 실제 재조판 결과만 대조하도록 고쳤다. 두 Windows 한컴 샘플에
줄 수 160/165, 줄바꿈 130/165 하한을 추가해 폭 누락 회귀를 막는다. 남은 차이는 좁은 wrap-zone의
저장 `segment_width`와 한컴의 제어문 전후 경계 규칙이며, 넓은 허용오차나 저장 배열 재사용으로
감추지 않았다.

## 렌더 영향과 시각 근거

`src/renderer/composer`의 line-break 및 line box를 바꾸므로 시각 검증 보조 경로를 적용했다.
Windows 한컴 2022 페이지 지문을 독립 오라클로 수집했고, `issue_1139_inline_picture_duplicate`
85건은 한컴 PDF 기준의 미주·수식·그림 위치와 쪽수 계약을 통과했다.

이번 보정의 직접 실행 경로는 in-memory edit/reflow이며, 저장본을 그대로 여는 화면은 기존
LineSeg를 사용한다. 따라서 저장본 before/after OVL PNG를 새로 만들면 변경 경로를 검증하지 못해
증적으로 사용하지 않았다. #3211 비캐시 저장 LineSeg 대조와 PDF 기준 integration 회귀를 함께
판정 근거로 사용했다.

## 완료한 검증

모든 Cargo 명령은 Windows PowerShell에서 `target\\pr-review` 하나를 재사용해 순차 실행했으며,
`CARGO_INCREMENTAL`은 설정하지 않았다.

| 검증 | 결과 |
| --- | --- |
| Hancom Office 2022 page oracle, worker 1 | 두 HWP 모두 23쪽·468문단·동일 지문 |
| 비캐시 #3211 LineSeg regression | 두 샘플 각각 160/165 line count, 130/165 line break |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 5 passed |
| `cargo test --test issue_1139_inline_picture_duplicate` | 85 passed |
| `cargo test --lib line_breaking:: -- --nocapture` | 2 passed |
| 대상 파일 직접 `rustfmt.exe --check`, `git diff --check` | 통과 |
| 변경 Markdown link check | 2 documents, internal relative links clean |

`cargo fmt`는 이 Windows 긴 경로에서 OS error 206으로 help만 출력해 판정을 제공하지 못했다.
직접 rustfmt 검사를 대체 근거로 썼다. Hancom Office 2024는 HKCU override 뒤에도 COM major 12를
반환해 오라클이 안전 중단했고, 2024 결과는 사용하지 않았다. 기본 COM 등록은 즉시 복원했다.

## CI 보정

초기 최신 head `bd852c93335374d0cba4331abde47c026634a2a5`의 GitHub Actions Lint는
`clippy::obfuscated_if_else` 한 건으로 실패했다. `has_inline_control_in_range(...).then(...)
.unwrap_or_default()`를 동등한 명시적 `if/else`로 바꾼
`cfb631f48e265847cf9266eca54b31430f002f7b`은 동작·검증 하한을 바꾸지 않는다.

Windows PowerShell의 동일 `target\\pr-review`에서 다음을 완료했다.

- `cargo clippy -- -D warnings` — 통과.
- `cargo test --lib issue3211_uncached_endnote_body_preserves_inline_control_flow -- --nocapture`
  — 통과, 두 샘플 각각 160/165 line count 및 130/165 line break.
- 직접 `rustfmt.exe --check`, `git diff --check` — 통과.

이 보정 뒤의 새 GitHub Actions는 merge 전 조건으로 다시 확인한다.

## 기준선 갱신 뒤 회귀 보정

`devel`의 PR #4743 병합을 head에 반영한 `895508a7afece7f5de99dbcbfb16db7b27dc7eaa`에서
새 CI가 실행됐다. 이 실행은 `pr_2219_hml_middle_anchor::
formatting_table_middle_anchor_preserves_vertical_text_flow` 한 건만 실패했다. Windows의
동일 재현도 `tests/pr_2219_hml_middle_anchor.rs:40`에서 `distinct trailing efg text run`을
보고했다.

원인은 HML fixture의 `abc + 글자처럼 취급되는 표 + efg` 문단이었다. 표의 폭(41,956 HU)을
visible text에서 표가 빠진 위치의 다음 글자 폭에 더하면, renderer가 만드는 기존
`TextRun`/`Table` 경계가 사라진다. `b7fbd8b0218d2cf3bdbb0a1618a3fb01c4b2ce90`
(`fix(#3211): 인라인 표 재조판 경계 보존`)은 `flow_inline_controls()`에서 table만 제외했다.
표의 기존 cell-split·empty-paragraph 제어문 배치와 크기 계산은 바꾸지 않았고, #3211에서
Windows Hancom 저장 LineSeg와 대조한 수식·그림 계열의 폭·높이 보정은 유지한다.

Windows PowerShell에서 `target\pr-review` 하나를 순차 재사용해 아래를 다시 확인했다.

| 검증 | 결과 |
| --- | --- |
| `cargo test --target-dir target\pr-review --test pr_2219_hml_middle_anchor formatting_table_middle_anchor_preserves_vertical_text_flow -- --nocapture` | 1 passed |
| `cargo test --target-dir target\pr-review --lib issue3211_uncached_endnote_body_preserves_inline_control_flow -- --nocapture` | 두 샘플 모두 160/165 line count, 130/165 line break |
| `cargo test --target-dir target\pr-review --test issue_1082_endnote_multicolumn_drift` | 5 passed |
| `cargo test --target-dir target\pr-review --test issue_1139_inline_picture_duplicate` | 85 passed |
| `cargo clippy --target-dir target\pr-review -- -D warnings` | 통과 |
| 직접 `rustfmt.exe --check`, `git diff --check` | 통과 |

`CARGO_INCREMENTAL`은 이 검증에도 설정하지 않았다. 이 보정의 새 head CI가 통과하고
mergeability를 재확인하기 전에는 merge하지 않는다.

## 결론과 merge 조건

코드 검토와 로컬·Windows 검증에서 blocker는 발견하지 못했다. 단, 이 PR은 #3211 전체 정합성을
완료하지 않으므로 이슈를 닫지 않는다. 문서 작성 시점 CI는 대기 중이므로 **merge 보류**다.

최종 merge 조건은 최신 PR head의 GitHub Actions 통과, 최신 mergeability 확인, collaborator
셀프 검토, 그리고 작업지시자 승인이다. 조건 충족 뒤에도 남은 wrap-zone 차이는 #3211에서 계속
추적한다.
