# PR #1149 검토 — HWPX TopAndBottom 빈 앵커 표 후행 line_spacing 보정

## 1. PR 메타

| 항목 | 값 |
|---|---|
| PR | #1149 |
| 제목 | Task #1147: HWPX TopAndBottom 빈 앵커 표 host_spacing + 후행 line_spacing 보정 |
| 작성자 | planet6897 |
| base ← head | `devel` ← `pr/task_m100_1147` |
| 상태 | OPEN |
| mergeable | MERGEABLE |
| 변경 규모 | 12 files, +887 / -2 |
| 연결 이슈 | closes #1147 |

## 2. 처리 방향

PR #1149는 PR #1148 이후 남은 렌더링 정합 보정이다.

다만 PR 브랜치의 base SHA가 현재 `devel`보다 오래되어, PR 브랜치를 그대로 병합하면
이미 반영된 #1148 문서/코드와 겹치고 `mydocs/pr/pr_1148_review.md` 삭제 같은 불필요한
diff가 함께 따라온다.

따라서 현재 `devel` 기준으로 다음 잔여 v2 변경만 선별 적용한다.

```text
src/document_core/queries/rendering.rs
src/renderer/layout.rs
src/renderer/typeset.rs
mydocs/plans/task_m100_1147_v2.md
mydocs/plans/task_m100_1147_v2_impl.md
mydocs/report/task_m100_1147_v2_report.md
mydocs/working/task_m100_1147_v2_stage1.md
mydocs/working/task_m100_1147_v2_stage2.md
```

## 3. 문제 요약

#1148에서는 HWPX 원본의 빈 앵커 `wrap=TopAndBottom` 비-TAC 표가 페이지네이션에서
`host_line_spacing`을 과대 가산하는 문제를 처리했다.

#1149는 그 후속으로, 렌더러(`LayoutEngine`)도 동일한 HWPX 빈 앵커 조건에서 표 아래 갭으로
앵커 문단의 마지막 `LINE_SEG.line_spacing`을 다시 더하지 않도록 맞춘다.

PR 본문 기준 재현 차이는 다음과 같다.

```text
typeset: host_line_spacing=0
layout: seg.line_spacing=1352 HU(약 18px)를 표 아래 갭으로 가산

결과:
  페이지네이션 used 높이와 실제 렌더 y_offset이 어긋나고,
  표 직후 문단이 권위 PDF/한컴 출력보다 아래로 밀린다.
```

## 4. 코드 변경 검토

핵심 변경:

```text
1. DocumentCore::find_page에서 source_format == Hwpx 여부를 LayoutEngine에 전달
2. LayoutEngine에 is_hwpx_source Cell<bool>와 set_hwpx_source() 추가
3. layout_table_item의 표 아래 간격 계산에서
   HWPX + 빈 앵커 + TopAndBottom 비-TAC 표이면 gap=0 처리
4. typeset_block_table의 ad hoc LayoutEngine에도 TypesetState.is_hwpx_source를 동기화
```

분기 조건은 기존 `is_current_empty_para_float`를 재사용한다.

```text
is_current_empty_para_float:
  - 현재 문단이 표 컨트롤을 포함
  - 컨트롤이 비-TAC
  - TextWrap::TopAndBottom
  - VertRelTo::Para
  - 가시 텍스트 없음
```

따라서 새 보정은 모든 HWPX 표에 적용되는 것이 아니라, PR #1147 계열의 빈 앵커
TopAndBottom 비-TAC 표로 범위가 제한된다.

## 5. 회귀 위험

주의할 점:

```text
1. LayoutEngine source flag가 누락되면 렌더러와 typeset의 시멘틱이 다시 어긋날 수 있다.
2. HWP/HWP3에는 적용되면 안 된다.
3. PR 브랜치 전체를 병합하면 stale base 때문에 문서 삭제/중복 diff가 생긴다.
```

현재 선별 적용 기준에서는 다음으로 격리된다.

```text
HWPX source: set_hwpx_source(true)
HWP/HWP3 source: 기본 false 유지
ad hoc LayoutEngine: TypesetState.is_hwpx_source 동기화
```

## 6. 검증

로컬 브랜치:

```text
local/pr1149-review
```

검증 결과:

```text
cargo fmt --all -- --check
  success

cargo test --lib
  1411 passed, 0 failed, 6 ignored

cargo test --test svg_snapshot
  8 passed

cargo test --tests
  success
```

`cargo test --tests`에서 페이지네이션/표 분할 계열 회귀 테스트도 함께 통과했다.

```text
issue_1070_tac_table_post_text_overflow
issue_1073_nested_table_split
issue_1079_picture_pushdown_vpos
issue_1082_endnote_multicolumn_drift
issue_1086
issue_1100_exam_social_hwpx_header
issue_1105
issue_1116
issue_1145
issue_546 / issue_554 / issue_643 / issue_703 / issue_775 / issue_986
svg_snapshot
```

GitHub PR CI:

```text
CI: SUCCESS
Render Diff: SUCCESS
CodeQL: SUCCESS
```

## 7. 권장안

**선별 수용 권장.**

PR #1149의 문제 정의와 잔여 v2 보정은 타당하다. 다만 PR 브랜치 전체 병합은 stale base
노이즈가 있으므로, 현재 `devel` 위에 잔여 변경만 반영하는 방식을 권장한다.

진행 순서:

```text
1. 선별 적용된 변경을 local/pr1149-review에서 커밋
2. 필요 시 WASM 빌드 후 메인테이너 시각 판정
3. local/devel에 병합
4. devel에 병합 후 테스트
5. origin/devel push
6. PR #1149에는 "선별 반영 완료" 코멘트 후 close
```

## 8. PR 코멘트 초안

```text
planet6897님, 후속 보정 PR 감사합니다.

#1148로 페이지네이션 쪽 host spacing 보정이 들어간 뒤, renderer/layout 쪽에서도 같은
HWPX 빈 앵커 TopAndBottom 비-TAC 표 조건에서 후행 line_spacing을 다시 가산하지 않아야
한다는 분석을 확인했습니다.

다만 이 PR 브랜치가 현재 devel보다 오래된 base에서 출발해 #1148 관련 문서/코드와 겹치는
diff가 포함되어 있었습니다. 그래서 PR 브랜치를 그대로 병합하지 않고, 현재 devel 위에
잔여 v2 변경만 선별 반영했습니다.

검증:
- cargo fmt --all -- --check
- cargo test --lib
- cargo test --test svg_snapshot
- cargo test --tests

모두 통과했습니다. 좋은 분석과 후속 정합 보정 감사합니다.
```
