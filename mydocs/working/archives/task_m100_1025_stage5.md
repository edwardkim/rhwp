# Stage 5 완료보고서 — #1025: 검증 + 잔여 해소 + 골든 재판정

- 타스크: #1025 / 브랜치 `local/task1025`
- 작성일: 2026-05-20
- 단계: Stage 5 — 잔여 overflow 해소 + 회귀 검증 + 한컴 PDF 시각 재판정

## 1. 잔여 overflow 두 측정 버그 해소 (커밋 82d71f17)

Stage 3+4 통합 후 잔여(pi=272 813px, pi=324 101px, aift +1):

1. **연속분 행-스킵 가드 오판** (`typeset.rs:2671`): start_cut 이 블록 컷
   `[2,1,1,2,32]` 일 때 `advance_row_cut`(per-row)로 판정 → 블록 첫 행(row 3) 소진을
   "행 소진"으로 오판 → cursor 전진 + start_cut 소실 → 연속분이 거대 셀을 처음부터
   재렌더 → overflow. **`start_cut_is_block` 이면 가드 스킵**(블록 컷 보존).
2. **mid-page 과분할**: fresh 페이지엔 들어가는 블록(잔여 공간만 부족)을 분할 →
   잔여 overflow. **`block_h > base_available`(진짜 page-larger)일 때만 mid-page 분할**,
   아니면 통째 다음 페이지로 deferred(기존 동작). aift/134 회귀 해소.

## 2. 결과 (AI 184p)

| | baseline | Stage 5 |
|--|----------|---------|
| LAYOUT_OVERFLOW | 18 | **11** |
| pi=272 (854.9px) | overflow | **해소** |
| pi=324 (143.9px, 잘림) | overflow | **해소 (2-프래그먼트 분할)** |
| pi=323 | 29.1px | 29.1px (불변, 소형 잔여) |

## 3. 회귀 검증 (무회귀)

- svg_snapshot: 5 pass / 3 debt(267/617/677 사전 부채, form-002 포함 공개 골든 무회귀).
- lib 테스트 1310 pass / 0 fail. clippy 0.
- 광범위 sweep: exam_eng(10)/kor(19)/math(0)/biz_plan(1)/k-water(3)/**aift(6, 회귀 해소)** 무회귀.

## 4. 한컴 2022 PDF 시각 재판정 (PMR-007, pi=324)

rhwp 분할 결과(idx62 rows0..5 end_cut=[2,1,1,2,32] → idx63 rows3..7 start_cut=[2,1,1,2,32])
를 한컴 PDF p63→p64 와 대조 — **정확히 일치**:
1. 거대 세부내용 셀 **문단 경계 분할** ("완료보고서…" → "에 제출하여 심의를…").
2. **헤더 행(요구사항번호 | PMR-007) 반복**.
3. **rs=2 라벨 셀(상세설명/세부내용) 연속 페이지 공란** (Stage 1 정답 #3).

→ #1025 핵심 케이스 시각적 완전 정합 확인.

## 5. 잔여 (소형, baseline 동일)
- pi=323 (29.1px), para=322 류 소형 잔여 — baseline 과 동일하거나 미세. page-larger
  핵심(pi=272/324)은 해소. 추가 정밀화는 별도 후속.
- 중첩표 보유 셀(atom 원자 유지), 표 외 page-larger 문단(pi=567) — 비범위.
