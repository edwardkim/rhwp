# 최종 결과보고서 — #1025: 페이지보다 큰 단일 표 셀 내부 분할 (intra-cell line split)

- 이슈: edwardkim/rhwp #1025 (M100) / 브랜치 `local/task1025` (base edf62865)
- 작성일: 2026-05-20
- 선행: #993(분할 표 cut 모델), #1022(측정 정합)
- 권위: `pdf/2. 인공지능(AI) … 제안요청서-2022.pdf` (한컴 2022)

## 1. 문제

페이지보다 큰 **단일 표 셀**(요구사항 명세 표 PMR-007 의 세부내용 셀, 25문단 ≈ 1024px)이
rowspan 라벨 셀(상세설명, rs=2)이 걸친 행에 있어, `advance_row_cut` 이 `row_span==1` 셀만
다루는 #993 컷 모델에서 **원자 처리 → 분할 불가 → 본문 143px 초과·용지 밖 잘림**.

## 2. 해결 — 행블록(row-block) 컷

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 진단 + 한컴 PDF 정답 확정 | 1f097e01 |
| 2 | 행블록 컷 설계 | 97915cd8 |
| 3(1/2) | `advance_row_block_cut` 함수 + parity 테스트 | eadca088 |
| 3(2/2) | 통합 시도 경험적 발견(typeset만으론 불가) | 90281837 |
| 3(2/2)+4 | typeset walk + 렌더러 블록-셀 컷 정합 | fd074de2 |
| 5 | 잔여 overflow 해소(가드/page-larger 판별) | 82d71f17 |

핵심:
- **`advance_row_block_cut`**: rowspan 블록 `[b_start,b_end)` 셀을 `(row,col)` 순서로
  순회, 거대 `row_span==1` 셀을 줄 단위 분할. rs>1 라벨 셀은 첫 조각 소비·연속 공란.
- **`PartialTable.is_block_split`** 플래그: page-larger 블록 분할만 블록-셀 컷 해석,
  일반 분할(form-002 등)은 per-row 유지 → 무회귀.
- **렌더러**(`table_partial.rs`): is_block_split 시 rowspan 행 포함, 블록-셀 인덱스로
  높이·줄범위 산출(`rowspan_block_range`/`block_cut_index`).
- **Stage 5 정정**: 연속분 행-스킵 가드를 블록 컷에서 스킵(컷 보존), mid-page 분할은
  진짜 page-larger 일 때만(아니면 deferred).

## 3. 결과 (AI 184p, 한컴 2022 PDF 정합)

| 항목 | baseline | 결과 |
|------|----------|------|
| LAYOUT_OVERFLOW | 18 | **11** |
| pi=272 세부 셀 (854.9px) | 잘림 | **해소** |
| pi=324 PMR-007 (143.9px) | 잘림 | **2-프래그먼트 분할 (한컴 p63→p64 정합)** |
| svg_snapshot(공개) | 5 pass/3 debt | 5 pass/3 debt (form-002 포함 무회귀) |
| lib / clippy | — | 1310 pass / 0 |
| 광범위(exam/aift/k-water/biz) | — | 무회귀 |

**한컴 PDF 시각 재판정(pi=324)**: 거대 셀 문단 경계 분할 + 헤더 반복 + rs=2 라벨 셀
연속 공란 — 한컴 2022 PDF p63→p64 와 **정확히 일치** 확인.

## 4. 잔여·후속
- pi=323(29.1px) 등 소형 잔여 — baseline 동일, page-larger 핵심(pi=272/324) 해소.
- 중첩표 보유 셀(atom 원자 유지), 표 외 page-larger 문단(pi=567) — 비범위.
- WASM 재빌드 — 릴리즈 시점.

## 5. 산출물
- 계획: `plans/task_m100_1025.md`, `_impl.md`
- 단계보고: `working/task_m100_1025_stage1/2/3/5.md`
- 코드: `table_layout.rs`(advance_row_block_cut+테스트), `typeset.rs`(walk),
  `table_partial.rs`(렌더러), `pagination.rs`(is_block_split).
