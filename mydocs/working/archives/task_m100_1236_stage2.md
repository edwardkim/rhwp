# Stage 2 완료 보고서 — Task #1236: 미주 줄간격 수정 (조건 게이트)

- **이슈**: #1236 (M100)
- **브랜치**: `feature/issue-1236-endnote-line-spacing`
- **단계**: Stage 2 / 3
- **작성일**: 2026-06-02

## 원인 확정

`src/renderer/layout/paragraph_layout.rs` 의 다줄 미주 문단 줄 배치
(`endnote_line_vpos_base.is_some()`, L1374 게이트 `end > start_line+1`):

```rust
let trailing = if line_idx + 1 < end { line_spacing_px } else { 0.0 };  // 마지막 줄 줄간격 누락
y + line_height + trailing + ...
```

- **다줄 미주 문단의 마지막 줄**에서 trailing 줄간격(~6px)을 0 으로 떨어뜨려, 다음 문단이
  `line_height`(14px)만 내려가 좁아짐. 단일줄·일반 문단은 else 분기에서 `line_height +
  line_spacing`(20px)을 항상 더해 정상 → **다줄 미주 문단 경계에서만 간헐적 좁음**.
- 무조건 줄간격 추가 시: 문19 수정되나 issue_1139(문제-사이 7mm 간격 2배)·issue_1189
  (페이지네이션 연쇄) 4건 회귀 — 미주 문단 높이가 페이지네이션과 결합(메모리 #1022 경고).

## 수정 — 조건 게이트

마지막 줄 trailing 줄간격을 **"다음 문단이 같은 미주(문제) 내 연속 문단"일 때만** 적용:

- `layout.rs`: `endnote_para_has_same_endnote_successor(para_index)` 추가 —
  `endnote_para_sources` 의 `(section_index, para_index, control_index)` 비교로 다음 미주
  문단이 같은 미주인지 판별.
- `paragraph_layout.rs`: 마지막 줄 게이트
  `if line_idx + 1 < end || self.endnote_para_has_same_endnote_successor(para_index)`.

→ 같은 풀이 내 연속(문19 "하면"→"f'(x)") → 줄간격 적용. 문제 경계(미주 마지막 문단,
between-notes margin 적용) → 0 유지 → 중복 가산·회귀 없음.

## 검증

| 항목 | 결과 |
|------|------|
| 문19 "하면→f'(x)" 간격 | 14.1 → **20.1px** (PDF 정합) |
| 시각 | `output/poc/task1236/mun19_gate_fix.png` (수정전 narrow → 수정후 균일 = PDF) |
| 회귀 테스트(issue_1139/1189) | 34 passed, 0 failed |
| 골든 스냅샷 | 8 passed |
| **전체 `cargo test`** | **1933 passed, 0 failed** |
| 페이지 수 | 21 불변 |

## 다음 단계

Stage 3: 나머지 지적 페이지(10·11·14쪽 문8·문11·문22·문24) 시각 확인 + 다문서(3-09·3-10월)
회귀 시각 점검 + 최종 보고서.
