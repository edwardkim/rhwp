---
kind: pr-review
status: active
issue: 4098
pr: 4754
---

# PR #4754 리뷰 - 레거시 OLE 차트 그리드 구조 파싱

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
visual_fixture_evidence.md, rework_and_exceptions.md,
review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_external_pr.md, intake_and_review.md, local_validation.md,
visual_fixture_evidence.md, rework_and_exceptions.md,
review_only_fast_pass.md, post_merge.md
current head: bb236611597f675808a63c1dff41ed7e7fbe004e
(문서 작성 직전의 녹색 code candidate)
```

| 항목 | 값 |
| --- | --- |
| PR | [#4754](https://github.com/edwardkim/rhwp/pull/4754) |
| 작성자 | `johndoekim` |
| 대상 / source | `devel` <- `task_m100_4098_ole_chart_grid` |
| 관련 이슈 | [#4098](https://github.com/edwardkim/rhwp/issues/4098) |
| code candidate | `bb2366115` - 메인터너 보정까지 포함한 후보 |
| 규모 | 문서 작성 전 12 files, +2,131 / -219, 3 commits |
| 작성 시점 참고 상태 | `MERGEABLE`, `CLEAN`; candidate의 CI·CodeQL 통과 |

외부 contributor source branch의 `maintainerCanModify=true`를 확인했다. 메인터너 보정은 원
contributor commit을 재작성하지 않고 동일 source 위의 별도 commit으로 추가했다. 최신
`upstream/devel@bea127738`과의 merge-tree 및 `git diff --check`도 통과했다.

## 변경 검토와 메인터너 보정

원 변경은 `VtDataGrid`의 명시 치수와 1-based 셀 인덱스로 값·라벨·방향을 구조적으로 읽는다.
정수 범위 휴리스틱, 카테고리-major 고정 가정, 숫자가 든 라벨의 계열 소실을 제거하고, OOXML 우선
경로와 기존 `#1251` 대조군 계약은 유지한다.

검토에서 다음 두 차단 결함을 확인해 `bb2366115`로 보정했다.

1. 손상 문서가 `rows`와 `cols`에 최대 `u16` 값을 선언하면 실제 입력 크기와 무관하게 대규모
   `Vec` capacity를 예약할 수 있었다. 이제 셀은 입력 바이트에서만 수집하고, 선언 치수와의 정합은
   마지막 구조 검증에서 실패 처리한다.
2. 공개 `legacy_grid_window`가 `VtDataGrid`의 `u16 version + u32 payload` 중 version을 건너뛰지
   않아 window 시작이 2바이트 빨랐다. scanner 프롤로그와 동일한 문법으로 계산하도록 정렬했다.

최대 치수 손상 스트림이 `NumberCellCountMismatch`로 안전하게 거부되는 회귀와, window가 선언 전체
뒤에서 시작하는 회귀를 `src/ole_chart/grid.rs` 단위 테스트로 고정했다. 상세 실행 순서와 commit
경계는 [implementation 기록](pr_4754_review_impl.md)에 남긴다.

## 렌더 및 시각 증적 판정

이 PR은 parser와 차트 IR 내용을 바꾸지만 `src/renderer`, layout, paint, HWP/HWPX fixture, golden,
기준 PDF를 수정하지 않는다. 레거시 `Contents`만 남은 입력에서 차트 IR이 복구되는 경로가 대상이며,
일반 차트의 OOXML 우선 렌더 경로는 그대로다. 따라서 새 PDF fidelity 수치나 renderer visual sweep을
merge 근거로 주장하지 않았다.

대신 `issue_4098_legacy_chart_grid` 통합 계약이 OOXML 정답지와 구조 스캐너의 값·방향을 비교한다.
생성용 `generate_legacy_only_variant`는 `output/`에 파일을 쓰는 수동 육안 확인용이라 ignore 상태이며,
자동 검증 통과로 기록하지 않았다.

## 완료한 검증

모든 Cargo 명령은 Linux `target/pr-review`를 순차 재사용했고 `CARGO_INCREMENTAL=0`은 지정하지
않았다.

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --target-dir target/pr-review --lib ole_chart::grid::tests -- --nocapture` | 15 passed |
| `cargo test --profile release-test --target-dir target/pr-review --test issue_4098_legacy_chart_grid -- --nocapture` | 9 passed, 1 ignored |
| `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 5,966 passed, 38 skipped, 7 slow |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `git diff --check` 및 최신 devel merge-tree | 통과 |
| code candidate GitHub CI | Build & Test, CodeQL, Lint, Rust shard 전체 통과; frontend/WASM/Native Skia skip은 preflight 분류에 따른 정상 상태 |

## 권고와 merge 조건

코드 후보의 차단 결함은 메인터너 보정과 회귀 테스트로 해소됐다. 이 문서와 오늘할일만 추가한
trailing docs-only head는 같은 PR source branch의 녹색 code candidate `bb2366115`를 재사용하는
fast-pass 대상이다.

**권고: merge.** 다만 merge 직전에는 이 trailing 문서 head의 preflight 및 Build & Test aggregate 성공,
최신 `MERGEABLE`/`CLEAN` 상태, 그리고 작업지시자의 merge 승인을 다시 확인한다. merge 뒤에는
[#4098](https://github.com/edwardkim/rhwp/issues/4098)의 종료 상태와 contributor PR 후속 comment를
공식 post-merge 절차에 따라 처리한다.
