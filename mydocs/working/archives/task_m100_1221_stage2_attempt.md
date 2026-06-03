# Stage 2 시도(미채택) 보고 — Task M100 #1221

**브랜치**: `local/task1221`
**결과**: cell-scoped line-height 클램프 시도 → **z-표 미해결 → 되돌림(revert)**

## 시도

`paragraph_layout.rs` `layout_composed_paragraph` 의 셀 전용 경로(cell_ctx)에서, 인라인 수식이 있는 줄의 line_height 를 수식 높이로 클램프:
- v1: `eq.common.height` 기준 → **무효과**.
- v2: `max(eq.font_size, eq.common.height)` 기준 → **여전히 무효과**.

## 계측으로 드러난 사실 (가설 수정)

- z-수식의 `common.height = 825`(11px), `font_size = 900`(12px). 즉 **수식 높이 ≈ 줄높이** — line_height < eq_height 가 아니라서 클램프가 거의 안 걸림.
- **진짜 지배적 결함은 line-height 가 아님**: z-열이 P-열 대비 **한 행 아래로 밀림** (행1 z 비고, 행2 에 "1.0"+"1.1" 겹침). 줄높이 1px 차(11 vs 12)로는 한 행 밀림이 설명 안 됨.
- 즉 z-열 셀의 **콘텐츠 세로 위치(valign 중앙정렬 오프셋) 또는 다중-줄 문단(p[0]=1.0/1.1) recompose** 단계에서 한 행분 어긋남 발생. line-height 클램프로는 해결 불가.

## 결론 / 다음 단계 (전용 사이클)

- line-height 클램프 접근 **폐기**. 초점을 **셀 콘텐츠 세로 배치**로 이동:
  1. `calc_cell_paragraphs_content_height` (centering 용) 의 z-열 총높이 산출과 실제 render 줄 위치 정합 점검.
  2. 다중-줄 문단(p[0])의 `recompose_for_cell_width` 결과가 stored 2줄과 일치하는지, 첫 줄(1.0) 위치가 왜 한 행 밀리는지 계측.
  3. valign=Center 오프셋이 z-열/P-열 간 한 행 어긋나게 만드는지 확인.
- 소스 변경 없이 되돌림 완료. #1221 은 본 계측 결과를 토대로 전용 사이클에서 셀 세로배치 중심으로 재착수 권장.
