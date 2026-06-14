# PR #1376 검토 - TAC 표 host line spacing 폴백

- PR: https://github.com/edwardkim/rhwp/pull/1376
- 제목: `fix(layout): apply TAC table host-line spacing when control index differs from line-seg index`
- 작성일: 2026-06-12
- 작성자: `mrshinds` (`wpresign`)
- 작성자 상태: first-time contributor
- base: `devel`
- head: `mrshinds:fix/tac-host-line-spacing`
- 처리 상태: merge 보류

## 1. 요약 판단

현재 상태로는 **merge 불가**로 판단한다.

PR #1376은 TAC 표가 비가시 컨트롤 뒤에 있을 때 `control_index`와 `lineSegArray` 인덱스가
어긋나 `layout_table_item`의 host line spacing 적용이 누락되는 문제를 해결하려는 PR이다.
재현용 `samples/tac-host-spacing.hwpx`와 회귀 테스트를 추가한 점은 좋다.

다만 PR의 `para.line_segs.get(control_index).or_else(|| para.line_segs.last())` 폴백이 기존
`issue_521` 회귀 테스트에서 TAC 표 하단 간격을 PDF 기대값보다 크게 만든다. GitHub Actions와
로컬 `cargo test --lib`가 모두 같은 테스트에서 실패했다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| PR 상태 | open |
| draft | false |
| base | `devel` |
| head | `mrshinds:fix/tac-host-line-spacing` |
| author association | `FIRST_TIME_CONTRIBUTOR` |
| maintainer can modify | true |
| mergeable | true |
| mergeStateStatus | `BLOCKED` |
| 연결 이슈 | 없음 |

## 3. 이슈 확인

PR 본문과 GitHub metadata의 `closingIssuesReferences`에는 연결 이슈가 없다.

검색 결과, #1376과 직접 대응되는 열린 이슈는 확인하지 못했다.

| 후보 | 상태 | 판단 |
|---|---|---|
| #1352 `HWPX 표 셀 TAC 이미지/텍스트 세로 정렬 한컴 정합` | open | 표 셀 내부 TAC 이미지/텍스트 세로 정렬 이슈라 #1376의 본문 흐름 spacing 문제와 별개 |
| #770 `shortcut.hwp 페이지 2~7 헤더 TAC 1x1 표 후속 spacing 누락` | closed | TAC 표 후속 spacing 누락이라는 닫힌 유사 이력 |

워크플로우상 연관 이슈 번호가 없는 PR은 코멘트로 이슈 연결 또는 신규 이슈 작성을 정중히 요청해야 한다.

## 4. 변경 범위

| 파일 | 내용 |
|---|---|
| `src/renderer/layout.rs` | `layout_table_item` 경로에서 `control_index`로 `line_segs`를 찾지 못하면 `last()`로 폴백 |
| `src/renderer/layout/integration_tests.rs` | `test_tac_host_line_spacing_with_preceding_invisible_controls` 추가 |
| `samples/tac-host-spacing.hwpx` | TAC host line spacing 재현용 synthetic fixture 추가 |

주요 변경 지점:

- `src/renderer/layout.rs:5112`
  - `tac_seg_applied` 이후 host line spacing을 적용할 때 `para.line_segs.get(control_index)` 실패 시
    `para.line_segs.last()`를 사용한다.
- `src/renderer/layout/integration_tests.rs:2151`
  - 새 fixture의 다음 문단 baseline이 약 161.3px인지 확인한다.

## 5. 검증 결과

### 5.1 GitHub Actions

| 체크 | 결과 |
|---|---|
| Build & Test | 실패 |
| CodeQL | 통과 |
| Analyze (javascript-typescript) | 통과 |
| Analyze (python) | 통과 |
| Analyze (rust) | 통과 |
| Canvas visual diff | 통과 |
| WASM Build | skipped |

Build & Test 실패:

- 실패 테스트: `renderer::layout::integration_tests::tests::test_521_tac_table_outer_margin_bottom_p2`
- 실패 위치: `src/renderer/layout/integration_tests.rs:1607`
- 실패 값: 박스 bottom `531.68` -> `①` y `556.53`, gap `24.85`
- 기대 값: PDF 기준 gap `20.00 ±2px`
- CI 로그: https://github.com/edwardkim/rhwp/actions/runs/27415506755/job/81033135957

### 5.2 로컬 검증

| 명령 | 결과 |
|---|---|
| `git diff --check HEAD` | 통과 |
| `cargo test --test issue_1139_inline_picture_duplicate issue_1139_page9_endnote_table_does_not_overlap_header` | 통과 |
| `cargo test --test svg_snapshot` | 통과, 8 passed |
| `cargo test --lib test_tac_host_line_spacing_with_preceding_invisible_controls` | 통과 |
| `cargo build --lib` | 통과 |
| `cargo test --lib` | 실패, 1724 passed / 1 failed / 6 ignored |

로컬 실패도 GitHub Actions와 동일하게
`renderer::layout::integration_tests::tests::test_521_tac_table_outer_margin_bottom_p2`이다.

## 6. 실패 원인 판단

PR #1376의 방향은 타당하지만, 현재 구현은 `control_index`가 line segment 인덱스와 맞지 않는 모든
TAC 표에 대해 마지막 line segment를 폴백으로 적용한다.

그 결과 다음 두 케이스를 구분하지 못한다.

- 실제로 비가시 컨트롤 때문에 host line spacing이 누락된 케이스
- 기존 table outer margin bottom 또는 TAC after-spacing 경로로 이미 기대 간격이 맞는 케이스

`test_521_tac_table_outer_margin_bottom_p2`에서는 기존 기대 gap이 `20.00 ±2px`인데, PR 적용 후
gap이 `24.85px`로 커졌다. 따라서 현재 폴백은 기존 PDF 정합 회귀를 만든다.

## 7. 추가 리스크

- 새 테스트의 설명 주석이 영어로 작성되어 있다. rhwp 프로젝트 규칙상 코드 주석은 가능한 한
  한국어로 정리하는 것이 좋다.
- PR 본문에 관련 이슈 번호가 없다. first-time contributor PR이므로 정중하게 이슈 연결을 요청한다.

## 8. 권고

현재는 merge하지 않고 재작업 요청이 필요하다.

권장 코멘트 방향:

1. 재현 fixture와 분석을 제공해 준 점에 감사한다.
2. 현재 CI와 로컬 `cargo test --lib`가 같은 기존 회귀 테스트에서 실패해 바로 merge할 수 없다고 설명한다.
3. `line_segs.last()` 폴백을 더 좁혀 기존 `issue_521`의 PDF gap 기대값을 유지하도록 요청한다.
4. 관련 이슈 번호를 PR 본문에 연결해 달라고 요청한다.
5. first-time contributor이므로 톤은 아주 정중하게 유지하고, 영어 본문 뒤에 한국어 요약을 병기한다.
