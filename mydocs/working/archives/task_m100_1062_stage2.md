# Stage 2 보고서 — Task #1062: 구분 조건 설계 + 페이퍼 검증

- 브랜치: `local/task1062` (소스 무변경, 측정·논증만)

## 0. Stage 1 용어 정정

Stage 1 결론의 "vpos_h 누적"을 정밀화한다. 렌더러의 문단당 실제 전진(consumption)은
**vpos-delta = 다음 문단 first_vpos − 현재 first_vpos = vpos_h + trailing_ls** 이다.
(vpos_h = `last.vpos + last.lh − first.vpos`, 문단 자체 span. 여기에 trailing_ls 가 더해져 다음 문단 시작.)

- 단일 줄 빈 문단: vpos_h(12px) + ls(6px) = **18px** = total_height (sb=sa=0). 실측 일치.
- 다단(다중 줄): total_height 는 vpos-delta 보다 과대(다단 TYPESET_DRIFT diff +8~9px 중 ls 6px 초과분).

→ 누적 정답 = **vpos-delta (= vpos_h + trailing_ls)**, line_segs 부재 시 total_height fallback.
   fit 판정은 종전 height_for_fit (마지막 항목 trailing_ls 제외) 유지.

## 1. 안전성 보조정리 (페이퍼)

현재 다단 누적 = `height_for_fit` (= total − trailing_ls). 신규 = vpos-delta (= vpos_h + trailing_ls).
- vpos-delta ≥ height_for_fit (항상 trailing_ls 만큼 이상 큼).
- 따라서 누적이 **증가만** → 페이지 분할이 **현재와 같거나 더 일찍** → 페이지 수 **≥ 현재, 절대 감소 없음**.

**따름정리:** 현재 분할이 height-packing(높이 한계)으로 결정된 파일만 분할이 이동한다.
구조(섹션/단/표/명시적 쪽나눔)로 분할되는 파일은 height 여유가 있어 vpos-delta 로도 분할 위치 불변.

## 2. 핵심 논증 — 렌더러는 이미 vpos 기반 = 정답지

렌더러는 vpos→y 매핑으로 그린다. vpos-delta 누적 페이지네이터는 렌더러가 col_bottom 에 닿는
지점에서 정확히 분할하므로, **렌더러는 구조적으로 overflow 불가**가 된다.

- **비회귀 파일**: 현재 렌더러가 이미 fit(overflow≤3) + PDF 쪽수 일치 → 분할이 height-packing 이
  아님(packing 이면 렌더러가 넘쳤을 것). ∴ vpos-delta 로도 분할 불변 → **무회귀**.
- **대상 파일**: 현재 PDF보다 쪽수 부족 + 대량 overflow = height-packing 실패. ∴ vpos-delta 가
  분할을 추가(렌더러 실제 전진과 일치)해 **PDF 쪽수로 수렴**.

## 3. 예상 동작 표 (9 파일)

| 파일 | 현재 overflow | 우리/PDF 쪽 | 분할 결정 요인 | vpos-delta 적용 예상 |
|------|------|------|------|------|
| 3-09 2022 (대상) | 151 | 21/23 | height-packing 실패 | 분할 +2 → 23, overflow→0 |
| 3-09 2023 (대상) | 116 | 19/20 | 〃 | +1 → 20, overflow→0 |
| 3-10 2022 (대상) | 110 | 16/18 | 〃 | +2 → 18, overflow→0 |
| 3-11 2022 (대상) | 92 | 19/21 | 〃 | +2 → 21, overflow→0 |
| exam_eng (#391) | 2 | 8/8 | 구조(단/페이지) | 불변 8, 무회귀 |
| exam_kor (#1022) | 3 | 20/20 | 구조+표 | 불변 20, 무회귀 |
| k-water-rfp (#359) | 1 | (단단) | 구조 | 단단 분기 별도, 무회귀 |
| 복학원서 (#1049) | 1 | 1/1 | 단일 페이지 | 불변 1, 무회귀 |
| footnote-01 (#1049) | 0 | 6/6 | 구조 | 불변 6, 무회귀 |

(쪽수 증가 예상치는 overflow 누적량 ÷ 본문높이 기반 추정. Stage 4 에서 실측 확정.)

## 4. 모순 점검 결과

- 비회귀 파일이 height-packing 으로 분할되면서 동시에 PDF 일치+fit 인 모순 사례 → **없음**
  (height-packing 이면 렌더러 overflow 가 관측됐어야 하나 ≤3). 보조정리와 일치.
- vpos-delta < height_for_fit 가능성(분할 빨라져 과분할) → 없음(vpos-delta ≥ height_for_fit 항상).
- ∴ 페이퍼 모순 없음 → Stage 3 진행 가능.

## 5. Stage 3 구현 지침

- 위치: `src/renderer/typeset.rs` 다단 누적 분기 (1834-1841, place / 1908+ split / 1817+ atomic).
- 변경: `if st.col_count > 1 { fmt.height_for_fit }` → **vpos-delta 기반 값** (line_segs 있으면
  `vpos_h + trailing_ls`, 없으면 `fmt.total_height`). 단단(col_count==1)은 현행 total_height 유지(검토).
- fit 판정(`st.current_height + fmt.height_for_fit <= available`)은 **불변**.
- 단위테스트: 시험지 단일줄 문단 누적 = 18px 검증, exam_eng 누적 무변 검증.

## 6. 잔여 리스크 (Stage 4 실측 의존)

- 다단 split(부분 분할) 경로·atomic TAC 경로의 누적도 동일 통일 필요 — 누락 시 부분 회귀 가능.
- 쪽수 증가 추정의 정확성 — Stage 4 에서 대상 4종 PDF 쪽수·배치 정합으로 확정.
- 전 251 샘플 LAYOUT_OVERFLOW 합계(devel 1624) 악화 없음 확인.
