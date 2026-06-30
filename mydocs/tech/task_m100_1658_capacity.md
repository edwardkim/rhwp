# 조사 — #1658 별표4 Δ+3 per-page 용량 정합 (clean lever 없음)

- 일자: 2026-06-29 / 대상: 산업통상부 별표4(80×15 표, 한글 25 / rhwp 28쪽, Δ+3).
- 도구: RHWP_TABLE_DRIFT(TABLE_SPLIT_AVAIL), detect_table_clipping, RHWP_UNIT_DEBUG.

## 1. 계측 결과
- **avail_for_rows = 1009.1px = 전체 body** (이 문서 pagination_tolerance = 0).
  → **tolerance 차감 문제 아님.** 용량은 한글과 동일한 body 전체.
- 페이지당 fragment 는 셀 내용 **vpos reset(=한글 페이지 경계)으로 경계**되며 ~34~35유닛/쪽으로
  한글과 정렬(full page 는 일치).
- 잔여 over 는 **행 전환 tail fragment**: 예) row31 cell page8 = 유닛 3..6(3유닛). 이전 페이지가
  rows24~30 + row31 앞부분으로 채워져 tail 에 row31 을 3유닛만 담음(한글은 reset 6 까지 6유닛).
  → 다음 페이지에 3유닛 orphan(Δ+1). 유사 transition 누적 = Δ+3.

## 2. 근인
- **선행 행(소행들)의 누적 높이가 한글보다 미세하게 커서 tail 가용이 부족** → row 시작 cell 이
  reset 까지 못 닿고 capacity-break → orphan. 다요인 px 누적(행높이 측정 미세차).
- tolerance 도, reset 분류도, 단일 cut 버그도 아님 — **measurement-fidelity(행높이 정합) 영역.**

## 3. clean lever 부재 (모두 배제)
| lever | 결과 |
|------|------|
| tolerance 차감 제거 | 별표4 avail 이미 full body(tolerance 0) → 무효 |
| reset-snap(다음 reset 까지 overflow 흡수) | 32px 로 대형 게이트 442→440 회귀(입증). 별표4 deficit 3유닛(~86px)≫32px → 못 닿음 |
| tiny_fragment_waste ≤3 확장 | #1488(가시 문단 reset 3유닛 후 보존) 단위테스트 위반 |

## 4. detect_table_clipping 발견 (별도 버그)
- 별표4 1페이지 **기존 클리핑 23.5px**(upstream 동일). 행분할 수정 무관. 렌더 측 fragment 높이
  과대(≈0.8유닛)로, Δ+3(3유닛 deficit)과는 별개 규모. 별도 render-fidelity 수정 대상.

## 5. 결론
- 별표4 Δ+3 은 **선행 행높이 누적 정합(measurement fidelity)** 문제로, page-count·clipping 게이트로
  안전하게 좁힐 clean lever 가 없다(3종 배제). 한글 행높이와의 미세 정합은 깊은 별도 작업.
- 현 안전 ceiling(별표1 일치, 별표4 Δ+8→Δ+3, 무회귀, 클리핑 불변) 유지.
- 후속: (a) 행높이 측정 정합(한글 대비 행 단위 diff 도구 필요), (b) 별표4 23.5px 클리핑 render 수정.
  둘 다 detect_table_clipping + render_page_gate 양 게이트로 검증.

## 6. 행높이 diff 도구 — 한글 레퍼런스 차단 (COM)
별표4의 어느 행이 한글보다 과대 측정되는지 규명하려면 **한글 per-row 행높이 레퍼런스**가 필요.
COM(pyhwpx) 경로 탐색 결과 차단:
- `hwp.CellShape` → 셀 진입/선택 후에도 **None**(ParameterSet 미획득).
- `hwp.TableLowerCell()` → 첫 셀에서 즉시 falsy(행 하향 이동 불가).
- `hwp.get_row_height()` → 0.
→ 경량 COM 으로 한글 행높이/행위치 추출 불가(이 환경 API 한계).

**viable 경로(무거움)**: `hwp.FileSaveAs(PDF)` 로 한글 권위 PDF 생성 후 PDF 표선 검출(PyMuPDF/cv)로
행높이 측정. 별도 도구·의존성 필요 → 본 라운드 범위 밖.

## 7. 상태
- 소스 무변경(조사만). PR #1670(행분할 수정 + 오라클 + 클리핑 검출기) 유지.
- **별표4 Δ+3 capacity 정합은 한글 행높이 레퍼런스 차단으로 본 라운드 보류.** viable 경로 = PDF 기반
  측정(무거움) 또는 COM API 추가 연구. 현 ceiling(별표1 일치/별표4 Δ3/무회귀) 유지.
