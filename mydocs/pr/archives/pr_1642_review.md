# PR #1642 리뷰 기록 — HWPX roundtrip IR-invisible 페이지네이션 변동 수정

- 작성일: 2026-06-29
- PR: https://github.com/edwardkim/rhwp/pull/1642
- 작성자: @planet6897
- base: `edwardkim/rhwp:devel`
- head: `planet6897/rhwp:task/1636-1637-pagination-fidelity`
- 검토 경로: collaborator-mediated 외부 PR 원본 head 문서 동반(Route A)

## 1. PR 메타 확인

| 항목 | 내용 |
|------|------|
| PR 유형 | 외부 contributor PR |
| maintainer modify | `true` |
| draft | 문서 작성 직전 기준 `false` |
| mergeable | 문서 작성 직전 기준 `MERGEABLE` |
| label | `hwpx`, `serialization`, `roundtrip`, `layout` |
| assignee | @planet6897 |
| review request | @postmelee |

`draft`, `mergeable`, `head SHA`, GitHub Actions 상태는 변동 가능하므로 approval/merge 직전에 다시 확인한다.

## 2. 변경 범위

PR의 코드 변경은 HWPX roundtrip 후 IR diff가 0인데도 페이지네이션이 변하는 문제를 줄이기 위한
직렬화 보존과 게이트 보강이다.

핵심 변경:

- `src/serializer/hwpx/section.rs`
  - secPr `hp:visibility`를 템플릿 고정값 대신 `SectionDef` IR 값으로 치환해
    `hideFirstEmptyLine` 등 visibility 계열 값을 보존한다.
- `src/serializer/hwpx/table.rs`
  - table `hp:pos@flowWithText` 하드코딩을 제거하고 `c.flow_with_text`를 방출한다.
- `src/serializer/hwpx/roundtrip.rs`
  - section visibility와 object `flowWithText` 차이를 `diff_documents` 게이트에 편입한다.
- `tools/verify_pi_page_roundtrip.py`
  - 같은 PI가 roundtrip 전후 다른 페이지에 배치되는지 확인하는 검증 도구를 추가한다.
- `tests/visual_roundtrip_baseline.rs`
  - 시각 roundtrip baseline 설명과 gate를 보강한다.
- `mydocs/plans/task_m100_1636.md`
- `mydocs/report/task_m100_1637_report.md`
- `mydocs/troubleshootings/hwpx_visibility_and_flow_with_text_serialization.md`

리뷰 중 확인한 문서 불일치:

- `mydocs/report/task_m100_1637_report.md`는 원인 B를 표 `flowWithText` 드롭으로 규명·수정 완료했다고 기록한다.
- 반면 `mydocs/troubleshootings/hwpx_visibility_and_flow_with_text_serialization.md`는 초기 조사 상태인 "미규명/추가 조사" 표현이 남아 있었다.
- 이 불일치는 코드 동작 문제는 아니며, PR head에 별도 문서 보정 커밋으로 정리한다.

## 3. 원인과 수정 타당성

원인 A는 secPr visibility의 `hideFirstEmptyLine`이 HWPX 직렬화 과정에서 `1 -> 0`으로 드롭되는 문제다.
파서는 값을 읽고 렌더러는 레이아웃에 반영하지만, 직렬화기는 템플릿 기본값을 방출했고 기존 roundtrip
게이트는 해당 필드를 비교하지 않았다. PR은 visibility 치환 방출과 section visibility diff를 추가해
이 누락을 봉인한다.

원인 B는 table `hp:pos@flowWithText`가 `0 -> 1`로 드롭되는 문제다. 원본 treatAsChar 표의
`flow_with_text=false`가 저장 후 true처럼 방출되면서 표 partial-split 임계가 바뀌고 페이지네이션이
변했다. PR은 table 직렬화가 IR 값을 사용하도록 바꾸고, object `flowWithText` 비교를 roundtrip
게이트에 추가한다.

코드 검토 결과, 위 두 수정은 기존 HWPX 직렬화의 속성 보존 누락을 좁게 보정하는 형태이며,
roundtrip 게이트가 같은 계열의 재발을 잡도록 확장되어 있다.

## 4. 로컬 검증

최신 PR head 기준 별도 worktree에서 다음 검증을 수행했다.

| 검증 | 결과 |
|------|------|
| `cargo test task1637 --lib` | pass, 4 tests |
| `cargo fmt --all --check` | pass |
| `python3 -m py_compile tools/verify_pi_page_roundtrip.py` | pass |
| `cargo test --test hwpx_roundtrip_baseline baseline_large_samples_roundtrip -- --nocapture` | pass |
| `cargo test --test hwpx_roundtrip_baseline baseline_all_samples_roundtrip -- --nocapture` | pass |
| `cargo test --test visual_roundtrip_baseline visual_baseline_all_samples -- --nocapture` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass |

## 5. 수동 시각 검증 산출물

외부 재현 파일 접근이 불가능한 상황을 대비해 비교 HTML을 별도 생성했다.

- 산출물: `/private/tmp/rhwp-pr1642-review/output/pr1642_visual/compare.html`
- 비교 컬럼: Original / Before PR roundtrip / PR roundtrip
- 확인 대상: Page 1, Page 3

자동 비교 요약:

- PR 전 roundtrip: `STRUCT_MISMATCH`, page 3 max displacement `622.00px`
- PR head roundtrip: `PASS`, page count `27 -> 27`, max displacement `0.00px`

육안으로는 Page 1 하단의 얇은 선 제거가 가장 보기 쉬운 차이며, Page 3 차이는 주로 구조/배치 안정성
측면에서 자동 diff가 드러내는 변화다.

## 6. 추가 검토 결과

PR 자체의 blocking 문제는 발견하지 못했다. 다만 본 PR 범위 밖 follow-up 후보는 두 가지다.

- #1654: HWPX -> HWP 변환 경로에서 `hideFirstEmptyLine` 관련 flags 동기화 검증
- #1655: HWPX 수식 `flowWithText` roundtrip 보존

두 항목은 현재 PR의 수정 범위를 막는 문제는 아니다. 각각 별도 이슈로 등록하고 #1637의 sub-issue로 연결했다.

## 7. 처리 계획

1. 문서 불일치 보정을 별도 커밋으로 PR head에 추가한다.
2. 본 리뷰 기록과 사전 판단 보고서를 별도 커밋으로 PR head에 추가한다.
3. PR comment로 로컬 검증, 수동 시각 산출물, follow-up 분리 내용을 공유한다.
4. GitHub Actions가 최신 head 기준 통과하거나, 문서 전용 후속 커밋 fast-pass 조건을 만족하는지 확인한다.
5. approval review와 merge는 작업지시자 승인 및 최신 상태 재확인 후 별도 단계로 수행한다.

## 8. 결론

코드 변경은 문제 원인 A/B를 모두 직접 겨냥하고, 회귀 게이트를 함께 보강하고 있다. 로컬 검증과 수동
시각 산출물 기준으로는 merge 수용 가능하다.

단, 이 문서는 사전 리뷰 기록이다. 최종 approval/merge 전에는 최신 PR head SHA, GitHub Actions,
mergeable 상태, PR diff에 review 문서가 포함되었는지 다시 확인해야 한다.
