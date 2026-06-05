# PR #1153 검토 — TAC 표 intra-paragraph vpos-reset 가드

## 1. PR 메타

| 항목 | 값 |
|---|---|
| PR | #1153 |
| 제목 | 본문 — 호스트 문단 내 TAC 표 intra-paragraph vpos-reset 가드 |
| 작성자 | planet6897 |
| 연결 이슈 | closes #1152 |
| base <- head | `devel` <- `pr/task_m100_1152` |
| 상태 | open |
| mergeable | true |
| 변경 규모 | 9 files, +808 / -0 |
| GitHub CI | CodeQL success, CI success, Render Diff success |

## 2. 문제 요약

`samples/2022년 국립국어원 업무계획.hwp`의 32페이지 하단에서 `별첨 | 국립국어원 일반현황`
1x3 TAC 표가 한컴 PDF와 달리 32페이지에 배치되는 문제다.

한컴 기준 정답은 다음 흐름이다.

```text
page 32:
  12x5 본문 표 마지막 행까지만 출력

page 33:
  1x3 별첨 박스
  1 연혁 및 임무
  □ 연 혁
```

PR의 핵심 주장은 pi=586 호스트 문단이 `text_len=0, controls=2, line_segs=2`이고,
두 번째 TAC 표에 매핑되는 `line_seg[1].vertical_pos == 0`이 같은 문단 안의 페이지 reset 신호라는 것이다.

## 3. 변경 내용

코드 변경:

```text
src/renderer/typeset.rs
  - typeset_tac_table() 진입부에 intra-paragraph vpos-reset 가드 추가
```

테스트:

```text
tests/issue_1152_intra_para_vpos_reset.rs
  - page 32에 pi=586 ci=1이 없고
  - page 33에 pi=586 ci=1이 있는지 검증
```

문서:

```text
mydocs/plans/task_m100_1152.md
mydocs/plans/task_m100_1152_impl.md
mydocs/working/task_m100_1152_stage1.md
mydocs/working/task_m100_1152_stage2.md
mydocs/working/task_m100_1152_stage3.md
mydocs/working/task_m100_1152_stage4.md
mydocs/report/task_m100_1152_report.md
```

## 4. 코드 검토

PR의 가드는 다음 조건에서만 발동한다.

```text
current_items가 비어 있지 않음
ctrl_idx > 0
호스트 문단 text가 비어 있음
line_segs.len() == controls.len()
line_segs[ctrl_idx].vertical_pos == 0
```

이는 좁은 조건이다. 일반 문단, 텍스트가 있는 호스트 문단, line_seg와 control 매핑이 불명확한 문단,
첫 컨트롤은 제외된다. 따라서 정상 문서에 대한 false positive 위험은 낮다.

주의할 점은 현재 `devel`이 PR base보다 앞서 있다는 점이다. 특히 `typeset_tac_table()`에는 PR #1088
후속으로 TAC 표가 첫 줄인 경우 `fmt.line_heights[0]`를 fit 기준으로 쓰는 보정이 이미 들어와 있다.
PR #1153의 가드는 그보다 앞에서 강제 advance를 수행하므로, cherry-pick 후 현재 `devel` 기준 회귀
테스트가 필요하다.

## 5. 검증 계획

권장 검증:

```text
1. PR commit을 현재 local/devel 또는 전용 review branch에 cherry-pick
2. cargo check
3. cargo test --test issue_1152_intra_para_vpos_reset
4. cargo test --test issue_1145_table_row_fit
5. cargo test --test svg_snapshot
6. 필요 시 samples/2022년 국립국어원 업무계획.hwp page 32/33 SVG 확인
```

추가 확인이 필요한 이유:

```text
- PR #1153은 페이지네이션 핵심 경로인 typeset_tac_table()를 변경한다.
- 현재 devel에는 PR #1088, #1145 등 표/페이지 fit 관련 후속 보정이 이미 들어와 있다.
- 따라서 PR branch의 CI success만으로는 현재 devel 통합 안정성을 확정하기 어렵다.
```

## 6. 현재 devel 기준 검증 결과

검토 브랜치:

```text
local/pr1153-review
```

적용:

```text
origin/pr/1153 cherry-pick
충돌 없음
```

검증:

```text
cargo check
cargo test --test issue_1152_intra_para_vpos_reset
cargo test --test issue_1145 --test issue_1070_tac_table_post_text_overflow --test issue_1073_nested_table_split --test issue_1086
cargo test --test svg_snapshot
cargo fmt --all -- --check
cargo check --target wasm32-unknown-unknown --lib
```

결과:

```text
pass
```

세부:

```text
issue_1152_intra_para_vpos_reset: 1 passed
issue_1070_tac_table_post_text_overflow: 3 passed
issue_1073_nested_table_split: 3 passed
issue_1086: 4 passed
issue_1145: 1 passed
svg_snapshot: 8 passed
wasm32 lib check: pass
```

## 7. 권장안

**수용 권장.**

조건부 항목이 현재 `devel` 기준에서 충족됐다.

```text
1. 현재 devel 기준 cherry-pick 충돌 없음
2. 페이지네이션 회귀 테스트 통과
3. svg_snapshot 통과
4. wasm32 target check 통과
```

PR의 원인 분석은 타당하고, 가드 조건도 충분히 좁다. 현재 `devel` 기준 통합 검증에서도
페이지네이션 회귀가 발견되지 않았다.

## 8. 승인 요청

권장 처리:

```text
PR #1153 commit을 수용
검토 문서와 최종 보고서를 함께 커밋
승인 후 local/devel -> devel 절차로 반영
```
