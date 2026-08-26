---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6079 review - 바탕쪽 세로 제목 Y축 축소 재래핑 제외 (#5947)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6079](https://github.com/edwardkim/rhwp/pull/6079) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) |
| base | `devel` |
| 원 head | `bc1ec23fe9e95bd5898962e6f22989051f4e6977` |
| 규모 | +103 / -5, 4 files, 2 commits |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, Build & Test success (작성 시점 참고값) |
| 원 PR CI | [run 32873775951](https://github.com/edwardkim/rhwp/actions/runs/32873775951/job/97892278861) |
| 통합 적용 | `9a112af58`, `bccb5843b` |

## 관련 이슈와 변경 범위

[#5947](https://github.com/edwardkim/rhwp/issues/5947)은 행정업무운영편람 바탕쪽 세로 제목
"업무관리시스템"의 줄바꿈이 깨지는 결함이다. 저장은 1글자×7줄인데 가용 폭 기준으로 2글자 줄로
합쳐진다.

`src/renderer/layout/shape_layout.rs`의 `should_reflow_matrix_textbox_lines`에서
`compressed_group_child` 판정을 x축 축소로 좁힌다. Y축 전용 축소(`sy<1`, `sx≈1`)는 저장 줄의
가로폭을 바꾸지 않으므로 재래핑 근거가 되지 않는다.

## 렌더 영향과 시각 검증 판정

바탕쪽 글상자 조판 경로가 바뀌고 편람 특정 쪽의 줄바꿈 개선을 주장하므로 **직접 증적 필수**
조합이다. 저장소에 한컴 기준 PDF(`pdf/2025 행정업무운영 편람(최종)-*.pdf`)가 있어 통합 head
산출물로 대조 가능하다.

## 발견한 문제와 risk

구조적 결함은 찾지 못했다. 변경은 순수한 좁히기이며, 높이 초과 폴백
(`matrix_textbox_lines_overflow_height`)이 그대로 남아 Y축 축소로 줄이 상자를 넘는 형상은 계속
재래핑된다. `has_axis_scale`도 sx·sy 양쪽을 보므로 진입 조건 자체는 불변이다.

## 검증 근거 (통합 head `136a94677`)

- 이 PR의 회귀 `issue_5947_handbook_sidebar_linebreak`가 통합 head 전체 회귀에 포함돼 통과했다.
  이 시험은 저장 `line_segs` 7개(각 `segment_width=1440`)와 composed 줄이
  `["업","무","관","리","시","스","템"]`으로 유지되는지를 직접 단언한다.
- 시각 검증: `rhwp export-png "samples/2025 행정업무운영 편람(최종).hwpx" -p 136` 산출물에서 바탕쪽
  세로 제목이 한 글자씩 7줄로 표시되는 것을 직접 확인했다
  (`mydocs/pr/assets/pr_6079_handbook_p137_after.png`).

## 최종 권고

**수용.** 변경은 순수한 좁히기이고 높이 초과 폴백이 남아 있어 회귀 위험이 낮다.
