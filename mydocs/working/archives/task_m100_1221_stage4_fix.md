# Stage 4 완료보고서 — Task M100 #1221 (해결)

**브랜치**: `local/task1221`
**결과**: **해결**. cell-scoped 최소 수정, 전체 회귀 0.

## 최종 근본 원인 (계측 확정)

z-표 값은 인라인 수식. 셀 수식-only 문단은 `paragraph_layout.rs` 의 빈-runs 줄 tac 렌더 블록(`comp_line.runs.is_empty()`)에서 그려진다. 이 블록은 tac 를 **줄 char 범위 `[char_start, next.char_start)`** 로 줄에 매핑한다.

그러나 수식-only 문단은 텍스트가 없어 **모든 LINE_SEG.text_start = 0** → 모든 composed 줄 `char_start = 0` (degenerate). 그 결과:
- line0: 범위 `[0, 0)` = 빈 범위 → 수식 0개
- line1: 범위 `[0, MAX)` → 모든 수식 흡수

→ p[0] 의 두 수식(1.0, 1.1)이 **둘 다 line1 에** 같은 y 로 렌더(가로 인접) → "1.01." 겹침, line0(행1) 비고.

## 수정

`paragraph_layout.rs` 빈-runs 줄 tac 블록:
- **모든 줄이 빈 runs(수식만) + 줄 char_start 비구분(degenerate) + 줄 개수 == tac 개수** 이면, tac→줄 매핑을 **순서(line_idx) 기준 1:1** 로 전환(`index_based_tac`).
- 그 외(일반 텍스트+수식, char_start 구분됨)는 기존 char-범위 로직 유지 → 동작 불변.

## 검증

| 항목 | 결과 |
|------|------|
| 4쪽 문26 z-표 | 1.0/1.1/1.2/1.3 각 행 분리, P-열과 정렬 (PDF 정합) |
| `cargo test --release` | **1896 passed / 0 failed** (svg_snapshot 다수 표/수식 회귀 0) |
| rustfmt | clean (paragraph_layout.rs 23+ 줄, 단일 블록) |

## 비고

수차례 잘못된 가설(line-height 클램프, 표준/분할 셀 경로) 배제 후, 빈-runs 줄 tac→줄 char 매핑의 degenerate char_start 가 실제 원인이었음. cell-scoped(빈-runs+degenerate 가드)라 회귀 위험 낮음.
