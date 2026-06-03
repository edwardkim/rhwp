# 최종 결과보고서 — Task M100 #1218

**이슈**: [#1218](https://github.com/edwardkim/rhwp/issues/1218) HWP5 wrap=Square 인라인 표 단락 세로 측정 부족 — 답안/표 행 겹침
**마일스톤**: v1.0.0 (M100)
**브랜치**: `local/task1218` (← `devel`)
**완료일**: 2026-06-01

---

## 1. 문제

`samples/3-09월_교육_통합_2023.hwp` 4쪽 문26: 답안 ① 0.7262 가 문제 끝줄("표준정규분포표를…구한 것은?")과 겹침. (부수: z-표 행 압축.)

## 2. 근본 원인 (계측으로 확정)

`RHWP_DEBUG_TAC_CURSOR` 계측:
```
Table pi=258 ci=5  y 822.4→895.9 (dy=73.6 = 표 높이)   ← 커서 전진
PartialPara pi=258 y 895.9→895.9 (dy=0.0)              ← 호스트 본문 전진 0
FullPara pi=259 ①  y 895.9→...                          ← 표 하단에서 시작 → 겹침
```
- 렌더 y 는 pagination current_height 가 아니라 파일 vpos + HeightCursor 로 계산.
- `wrap=Square` 표는 커서를 **표 높이만큼만** 전진, 호스트 본문(`layout_wrap_around_paras`)은 dy=0.
- 본문(90px) > 표(73.6px) → 본문 하단이 표 하단보다 아래인데 ①이 표 하단에서 시작 → 겹침.

## 3. 수정 (Stage 2)

`src/renderer/layout.rs` 어울림 호스트 Table item: `layout_wrap_around_paras` 후
커서를 `host_text_bottom = table_y_before + 본문높이` 로 전진(표보다 길 때만).
- 본문 ≤ 표 인 기존 다수 케이스는 불변(가드). 19줄 추가, layout.rs 단일 hunk.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 계측 (수정 후) | ① pi=259 895.9 → **912.5** (본문 아래 분리) |
| 4쪽 문26 시각 | ①~⑤ 각 줄 분리, 한글 2022 PDF 정합 |
| `cargo test --release` | **1896 passed / 0 failed** (wrap_around `issue_546`, `svg_snapshot` 포함 회귀 0) |
| rustfmt | clean |

## 5. 미해결 / 분리 (Stage 3)

z-표 행 압축("1.0"/"1.1" 겹침, z-열↔P-열 정렬 어긋남)은 **표 셀 내부 세로정렬**(`table_layout.rs` 셀 콘텐츠 높이 + valign 중앙정렬, 셀 stored `lh=825<폰트`) 문제로 **다른 서브시스템·고위험**. 모든 표 렌더에 영향 → **별도 이슈로 분리 권장** (상세: `working/task_m100_1218_stage3.md`).

## 6. 결론

#1218 의 **주 증상(① 답안↔문제 겹침)은 Stage 2 로 해소**. z-표 행 압축은 별도 이슈.

## 7. 산출물

- 소스: `src/renderer/layout.rs`
- 계획: `mydocs/plans/task_m100_1218{,_impl}.md`
- 단계: `mydocs/working/task_m100_1218_stage{1,1b,2,3}.md`
- 최종: 본 문서
