# Stage 3 완료보고서 — #1046: Class A 수정 (다행 표 비분할 첫행 → 조건부 이월)

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 단계: 측정 통일(B) Stage 3 — 분류(→ `_stage3_classify.md`) 후 Class A 수정
- 작성일: 2026-05-21
- 대상: pi=290 (sec0, COR-003 요구사항 표, 7×3 tac)

## 1. 수정 내용 (1지점, typeset.rs 분할 진입부 가드)

Stage 2가 정합한 pre-loop 가드에 다행 표 이월 조건을 추가. 가드의 `first_row_force_splittable`
추정(`pad + min(line_h, 20)`)이 **분할 불가 첫 행**도 현재 페이지에 붙잡아, 루프가 통째 강제
배치 → 본문 초과(pi=290 행0 38.6px > 잔여 23.8px, 0.04px 차로 가드 미발동).

추가 조건 `multirow_clean_defer`:
```
!first_row_splittable           # 첫 행 내부 분할 불가
&& row_count > 1                 # 다행 표 — 행 경계로 깨끗이 이월 가능
&& first_block_end < row_count   # 첫 블록 뒤로 후속 행 존재
&& split_unit_h <= base_available - first_frag_overhead   # fresh 페이지엔 통째로 들어감
→ advance_column_or_new_page()
```

- **genuine page-larger** (>fresh capacity)·**1×1 단일 셀**(row_count==1, 행 경계 없어 셀
  내부 컷 필요, #874 aift)은 조건에서 제외 → 기존 force-split(렌더러 경계 컷) 유지.

## 2. #874 회귀 차단 (검증된 핵심)

초기 시도(`!first_row_splittable && fits_fresh_page`만)는 aift.hwp 74→**76p** 회귀
(issue_554 실패). aift의 다행 force-split 표까지 이월한 탓 — 이 아니라, 광범위 이월이
원인이었다. `row_count > 1 && first_block_end < row_count` 로 좁혀 **1×1 셀(#874)을 제외**
하니 aift 74p 유지. issue_147_aift_page3 골든도 통과.

## 3. 결과 (대상 샘플)

| 지표 | Stage 2 후 | Stage 3 후 |
|------|-----------|-----------|
| LAYOUT_OVERFLOW 총건 | 16 | **15** |
| in-scope (page-larger 제외) | 12 | **11** |
| 해소 | — | pi=290 (8.7px) |
| page-larger (323/567) | 2 | 2 (불변) |
| 신규 overflow | — | 0 |
| 총 페이지 수 | 185 | 185 (불변) |

pi=290 COR-003 표: page49→50 이월, **통째(635.4px) 배치, overflow 0**.

## 4. 회귀·정합 검증

- `cargo test --release`: **1516 passed / 0 failed** (골든 회귀 0).
- aift.hwp 74p 유지(issue_554), issue_147_aift_page3 통과 → **#874 무회귀**.
- 한컴 2022 PDF(idx 48): COR-003 표 헤더~산출정보 한 페이지 **통째** 배치 → 정합.

## 5. 잔여 (in-scope 11건)
- Class B(통째 표 7~12px): 266/308/354/357 · Class C(연속 ~2-3px): 218/600 ·
  Class D(문단 분할): 361/429/781/268/406. (`_stage3_classify.md` 참조.)
