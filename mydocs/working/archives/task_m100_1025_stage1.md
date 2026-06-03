# Stage 1 완료보고서 — #1025: page-larger 셀 진단 + 한컴 PDF 정답

- 타스크: #1025 / 브랜치 `local/task1025` (base: `local/devel` = edf62865)
- 작성일: 2026-05-20
- 단계: Stage 1 — 진단 + 한컴 2022 PDF 정답 확정 (코드 무수정)
- 검증 샘플: `samples/2. 인공지능(AI) … 제안요청서.hwpx` (184p, 비공개)
  / `pdf/2. …-2022.pdf` (한컴 2022 권위, 179p)

## 1. 재현 (page-larger 표 셀 overflow)

`LAYOUT_OVERFLOW` (edf62865 빌드):

| pi | type | overflow | 비고 |
|----|------|----------|------|
| 272 | PartialTable | **854.9px** | 거대 셀 |
| 324 | PartialTable | **143.9px** | PMR-007 표, 잘림 |
| 323 | PartialTable | 29.1px | 인접 표 |
| 567 | PartialParagraph | 856.7px | **표 아닌 page-larger 문단 — 비범위(§6)** |

## 2. 코드 원인 확정

`typeset.rs:2539~2560`:
```rust
let rowspan_touched[r] = table.cells.iter().any(|c| c.row_span>1 && c.row<=r<c.row+c.row_span);
let cut_row_h[r] = if rowspan_touched[r] { mt.row_heights[r] }  // 원자·분할 불가
                   else { row_cut_content_height(...) };          // 줄 단위 분할 가능
```

`advance_row_cut`(table_layout.rs)는 `row_span==1` 셀만 순회(908/911) → rowspan 셀이 걸친
행은 컷 모델 밖 → `mt.row_heights[r]` 원자 폴백.

**pi=324 (PMR-007 7×3)**: `cell[6] r=3 rs=2`(상세설명, rows 3-4 걸침) → row 4 가
`rowspan_touched`. row 4 의 거대 세부내용 셀 `cell[10]`(25문단, h=76816HU≈**1024px**)이
원자 처리 → 페이지(941px)보다 커도 분할 못 함 → 통째 배치 → 143px 초과·잘림. 진단 확정.

## 3. 한컴 2022 PDF 정답 (PMR-007, p63→p64)

거대 셀이 한컴에서 **2페이지에 걸쳐 분할**됨. 분할 규칙:

1. **거대 내용 셀은 문단/줄 경계에서 분할** — p63 "…완료보고서 각 3부)" 까지, p64 "※ 사업자는
   과업이 완료되기…" 부터 연속. (mid-line 아님, 문단 경계.)
2. **헤더 행(요구사항번호 | PMR-007)은 연속 페이지에 반복** (기존 repeat_header 동작).
3. **rs=2 라벨 셀(상세설명) + 세부내용 라벨 셀은 첫 조각에만 표시**, 연속 페이지에선
   해당 좌측 칸이 **빈 칸**(라벨 미반복, 내용 셀만 이어짐).
4. 표 종료 후 산출정보·관련요구사항 행이 정상 이어지고 다음 표(PMR-008) 시작.

→ **정답 모델**: page-larger 행(rowspan 셀 포함)을 셀 내부 문단/줄 범위로 분할,
헤더 반복, rs>1 라벨 셀은 첫 조각 렌더 후 연속 조각은 공란.

## 4. Stage 2 입력 (설계 과제)

- `advance_row_cut`/`row_cut_content_height`/`cell_line_ranges_from_cut` 를 rowspan 셀
  포함하도록 확장 — rowspan 셀의 잔여 진행(셀별 누적 컷)을 행 경계 넘어 추적.
- `rowspan_touched` 원자 처리 → "셀 1개라도 page-larger 면 줄 단위 분할 허용" 전환.
- 렌더러(table_partial.rs): 연속 조각의 rs>1 라벨 셀 공란 렌더(정답 #3).
- 내부표 보유 셀(pi=323 cell 내부표)은 분할 시 원자 유지 여부 Stage 2 결정.

## 5. 비범위 재확인
- pi=567 류 표 외 page-larger 문단/그림. WASM 재빌드.
