---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6056 review - 셀 안 표 검색·치환 확장 (#2792)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6056](https://github.com/edwardkim/rhwp/pull/6056) |
| 작성자 | [@CO0Ki3](https://github.com/CO0Ki3) |
| 원 head | `5e5ede9a94602514cf3b485938e598dd52fa99cb` |
| 통합 적용 commit | `e948cb618` |
| GitHub 상태 | non-draft, `mergeStateStatus=BLOCKED`, CI 16 success/4 skip/2 failure |
| review 상태 | `CHANGES_REQUESTED` |
| 통합 판정 | **메인터너 보정 포함 수용 권고** |

## 검토 요약

`search_all`이 셀 문단 내부 `Control::Table`을 버려 중첩 표 텍스트를 찾거나 치환하지 못하던
핵심 진단과 `cellPath` 기반 by_path 재사용 방향은 타당하다. 깊이 1 결과는 기존 `cellContext`를
보존하고, 깊이 2 이상만 `cellPath`로 싣는 하위호환 전략도 맞다.

원 head는 기존 계약 테스트를 갱신하지 않아 Archive C가 실패했고, `grep`과 실제 치환 경로의 범위·순서가
갈라져 `edit replace-text --dry-run`과 실제 실행이 다른 결과를 낼 수 있었다. 글상자 안 표는 `search_all`
이 찾지만 `grep`이 놓치는 새 불일치도 남아 있었다. 따라서 원 PR 그대로는 보류가 맞고, 통합 후보에서
메인터너 보정으로 처리했다.

## 메인터너 보정

- `tests/cases/issue_2792_search_nested_tables.rs`의 낡은 “중첩 전용 토큰은 치환하지 않는다” 계약을
  최신 계약인 `count == 1`로 갱신했다.
- `edit replace-text --dry-run`과 `changedPages` 산출 전 매치 수집을 `grep` 대신 실제 치환과 같은
  `search_all_text_native(..., include_cells=true)` 결과로 맞췄다.
- `grep` 주석을 최신 계약으로 수정하고, 글상자 안 표 텍스트도 direct search 범위에 포함했다. flat
  `cell` 주소로 글상자 내부 표의 전체 경로를 표현할 수 없으므로 match에는 `textbox` 호스트를 싣는다.
- Studio `SearchHit`/`SearchResult` 타입에 `cellPath`와 `equationControl`을 반영했다.
- 글상자 안 표 텍스트를 `grep`이 찾는 회귀 테스트를 추가했다.

## 로컬 검증

- `cargo test --profile release-test --target-dir target/pr-review --test regression_suite_005 issue_2792_search_nested_tables -- --nocapture`: 10 pass
- `cargo test --profile release-test --target-dir target/pr-review --test regression_suite_011 replace_occurrence_contract -- --nocapture`: 4 pass
- `cargo check --profile release-test --target-dir target/pr-review --tests`: 통과
- `cargo fmt --check`: 통과
- `npm --prefix rhwp-studio run build`: `tsc && vite build` 통과

## 권고

메인터너 보정 후 원 PR의 Archive C 실패와 dry-run/실행 불일치 위험을 통합 후보에서 해소했다.
#2792의 rect·줄정보 path core 축은 원 PR 본문처럼 별도 잔여 범위이므로, 이 통합은 검색·치환 범위
확장으로 수용하고 tracker는 후속 판단에 맡긴다.
