# Stage 3 진행보고서 — #1025: 블록 컷 함수 + 통합 시도 (경험적 발견)

- 타스크: #1025 / 브랜치 `local/task1025` (base edf62865)
- 작성일: 2026-05-20
- 단계: Stage 3 — 페이지네이터 블록 컷. (1/2) 함수 커밋 / (2/2) walk·렌더러 통합 시도.

## 1. Stage 3 (1/2) — 완료·커밋 (eadca088)

`advance_row_block_cut` / `row_block_cells` / `row_block_content_height`
(`table_layout.rs`) + 단위테스트 2:
- 단일행 block == `advance_row_cut` (회귀 0) ✓
- rowspan 블록 거대셀 줄 단위 분할 + 라벨 첫조각/연속공란 ✓

## 2. Stage 3 (2/2) — typeset walk 배선 시도 + 경험적 발견 (되돌림)

typeset walk(protected-block 분기)를 블록 컷 분할로 배선:
- 블록이 avail 초과 + 분할 가능 → `advance_row_block_cut` 으로 분할, `split_end_cut`
  (블록-셀 인덱스) 기록, `split_block_start` 로 연속분 커서를 블록 시작 행 복귀.

**빌드 + 게이트 결과 (typeset 변경만, 렌더러 미변경):**
- svg_snapshot: 5 pass / 3 debt — **공개 골든 신규 회귀 0**(267/617/677은 base 사전 부채).
- **그러나 AI 184p LAYOUT_OVERFLOW 폭발**: pi=272 854→1642+825, pi=323 29→899,
  pi=324 144→206+114, pi=321/322 신규.

**원인(확정)**: 페이지네이터는 블록을 분할하나, 렌더러(`table_partial.rs`)가 컷을
**per-row `row_span==1` 인덱스**로 해석(445~463) + rowspan 행을 컷에서 제외(122~129) →
블록-셀 인덱스 컷을 잘못 적용 → 잘못된 줄 범위 렌더 → 대규모 오버플로.

→ 합의(회귀 시 revert)대로 **typeset walk 변경 되돌림**. baseline 복원 확인
(pi=272 854.9/323 29.1/324 143.9). 기반(eadca088) 유지.

## 3. 확정된 다음 작업 (렌더러 per-row → block 재작성)

`table_partial.rs` 가 블록 컷을 정합 해석하도록 재작성 필요(고위험·focused):
1. **컷 인덱싱**: per-row(`row_span==1` col 순) → **블록-셀(row,col) 순**. 블록 범위는
   분할 행의 rowspan-확장(`rowspan_block_range`)으로 산출. 단일행=블록[r,r+1) 호환.
2. **rowspan 행 제외 해제**(122~129): 분할 블록의 rs>1 셀은 첫 조각 콘텐츠 / 연속 조각
   공란(한컴 정답 #3), 거대 row_span==1 셀은 블록 컷 줄 범위로 렌더.
3. **행 높이 모델**: per-row `row_heights[]` 가 블록-셀 컷과 불일치 → 블록 프래그먼트
   표시 높이를 `row_block_content_height` 로 산출해 정합.
4. PartialTable 가 블록 분할임을 렌더러가 식별(블록 범위 전달 or rowspan 재검출).

typeset walk 배선 코드는 본 보고서에 설계가 남아 있어 렌더러 재작성과 **함께** 재적용한다
(둘이 동시 착지해야 정합 — 본 시도가 분리 시 오버플로 폭발을 실증).

## 4. 권고
렌더러 per-row→block 재작성은 #993/#1022 분할 전반 영향 + 행높이 모델 변경이라 별도
focused 세션에서 svg_snapshot·overflow 게이트로 단계적 진행. 현재 기반(블록 컷 함수+테스트)
은 안전하게 커밋됨.
