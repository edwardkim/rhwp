# Task #518 최종 보고서 — Layout 리팩터링 Phase 2

**이슈**: #518
**브랜치**: `local/task518`
**Phase**: 2 (line_break_char_idx 다중화)

## 1. 작업 내용

`layout_inline_table_paragraph` 줄 나눔 위치 결정 정확도 향상.

### 변경 핵심
1. `Option<usize>` → `Vec<usize>` 로 ls[1..] 모두 사용
2. ctrl_gap 기반 알고리즘 → 직접 char_offsets 룩업 (간결 + 정확)

### 이전 알고리즘의 결함

paragraph 전체 controls 합을 ts 에서 빼는 over-subtraction 으로, controls 가 있는 paragraph 에서 saturating 0 → 항상 None 반환 → dynamic right_margin reflow 로 fallback. 하지만 dynamic reflow 도 inline_x 가 right_margin 을 초과하지 않으면 wrap 안 함 → 본문이 column 폭 너머로 늘어짐.

#496 (exam_science p2 pi=61) 케이스가 정확히 이 패턴 (10 controls + 56 char text).

## 2. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | line_break_char_indices Vec 도입 + wrap 로직 + LAYOUT_BREAK_INDICES 디버그 로깅 |

## 3. 검증

### 3-1. #496 재현 케이스

| | Before | After |
|---|--------|-------|
| 본문 라인 수 | 1 (column 폭 초과) | 3 (정상 wrap) |
| ls[1] break (char 5) | 무시 | **적용** |
| ls[2] break (char 28) | 무시 | **적용** |

### 3-2. 광범위 회귀

7 샘플 170 페이지: **same=167 / diff=3** (exam_science p2/p3/p4)
- exam_kor / exam_eng / exam_math / synam-001 / aift / 2010-01-06 byte 동일
- exam_science 변경 = 정확도 정정

### 3-3. 단위 + 통합 테스트

- `cargo test --release --lib`: **1103 passed**

## 4. 영향 범위

| 케이스 | 영향 |
|--------|------|
| Single line_seg paragraph | 변화 없음 (`Vec.is_empty()` → dynamic reflow) |
| Multi line_seg, controls 없음 | 동일 break (이전 algorithm 결과 일치) |
| Multi line_seg + controls (#496 케이스) | 정정 (이전 saturating 0 버그 해결) |

## 5. 잔여 결함 (별도 Phase)

#496 는 복합 본질 (A/B/C):
- (B) ls[2..] break 미사용 — **본 task 에서 해결 ✓**
- (A) 본문 baseline 수직 정렬 정책 — Phase 3
- (C) 인라인 vs 블록 정책 — Phase 4

#496 이슈 본문 "줄들이 거의 같은 y 에 그려져" 는 (B) 가 주원인이었음. 본 fix 로 본문이 정상 wrap → 시각적으로 글자 안 보임 문제 해소.

baseline 수직 정렬 (본문 y=1195 가 표 row 0/row 1 사이 끼임) 은 별도 Phase 3 에서 처리.

## 6. 요약

- ls[1..] 다중 break 사용 ✓
- ctrl_gap over-subtraction 버그 정정 ✓
- 회귀: 7 샘플 167/170 동일 (3건은 정확도 정정) ✓
- 단위 1103 통과 ✓
- #496 본질 (B) 해결 (본질 A/C 는 별도 Phase)
