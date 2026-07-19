---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1658/README.md
last_verified: 2026-07-16
---

# 통합 진단 — #1658 별표4 클리핑 ≡ capacity 결손 (cut↔render↔한글 줄높이 fidelity)

- 일자: 2026-06-30 / 대상: 산업통상부 별표4 / 도구: detect_table_clipping, export-svg SVG 분석, RHWP_TABLE_DRIFT.

## 1. 클리핑 위치 (detect_table_clipping → SVG 정밀)
- 페이지 2: 마지막 표 행 전체가 baseline **y=1118.2** 에 렌더, body_bottom **1094.7** → **over 23.5px**.
- 행 pitch = 28.8px(baseline 간격 일정). 마지막 행이 **1행 초과 포함**되어 본문 밖(하단 여백)으로 그려짐 → body-clip 에 잘림(데이터 손실).

## 2. 근인 분해 (≈23.5px)
| 성분 | 크기 | 설명 |
|------|------|------|
| cut↔render 유닛높이 drift | ~0.4px/유닛 × ~28유닛 ≈ **11px** | `advance_row_cut` 셀유닛(≈28.4px) < 렌더 줄 pitch(28.8px). cut 이 과소측정 → 페이지네이션이 1행 초과 포함 |
| 페이지네이션 body ↔ render body | **~10px** | pagination avail_for_rows=1009.1 vs SVG body height=1019.1 |
| 합 | ≈21~23px | 관측 23.5px 와 정합 |

## 3. 핵심 통찰 — 클리핑과 capacity 는 같은 뿌리·상충 방향
- **클리핑**(render 가 cut 보다 큼) 과 **capacity 결손**(rhwp 가 한글보다 적게 적재) 은
  모두 **cut↔render↔한글 3자 간 sub-px 줄높이 fidelity 차이**의 다른 발현이다.
- **방향 상충**:
  - 클리핑 수정 = cut 을 render 만큼 크게 측정 → 더 일찍 끊음 → **페이지수↑(Δ 악화)**.
  - capacity 수정 = 더 채움(cut 작게/avail 크게) → **클리핑↑**.
  - 한글은 셋 중 가장 많이 적재(가장 작은 유효 줄높이 또는 tolerance) → cut/render 양쪽과 다름.
- ∴ 단일 조정으로 클리핑·capacity·한글정합을 동시 만족 불가. **3자 줄높이 정합(cell_units ↔
  layout_partial_table ↔ 한글)** 이 본질 과제.

## 4. 결론
- 별표4 23.5px 클리핑은 **국소 render 버그가 아니라** cut↔render 줄높이 drift(≈0.4px/유닛) + body
  영역 10px 불일치의 누적 → capacity 결손과 **동일 뿌리**.
- 안전한 단독 수정 불가(클리핑↔capacity 상충, 한글 3자 정합 필요). page-count·clipping 게이트만으론
  방향을 못 정한다 → **한글 권위 줄높이 레퍼런스(PDF 측정 등)** 가 선행되어야 3자 정합 가능.
- 현 ceiling(별표1 일치 / 별표4 Δ8→Δ3 / 무회귀 / 클리핑 23.5px 불변=기존) 유지.

## 5. 권고 (후속 라운드)
1. **한글 줄높이 레퍼런스 인프라**(한글 PDF 표선/텍스트 baseline 측정, PyMuPDF) — 3자 정합의 기준.
2. 그 위에서 cell_units(cut) ↔ layout_partial_table(render) 줄높이 통일 + 한글 baseline 정합.
3. 모든 변경은 render_page_gate(페이지수) + detect_table_clipping(클리핑) 양 게이트 무회귀 + 한글 baseline diff 로 검증.
