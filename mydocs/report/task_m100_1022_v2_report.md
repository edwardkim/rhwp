# Task #1022 v2 — 최종 보고서: trailing-ls 보정 조건부화

## 결과

PR #1024 재구성 중 검출된 `issue_598` 회귀를 해소. `src/renderer/layout.rs` lazy_base
산출에서 #537 trailing-ls 보정을 **조건부**로 복원.

## 게이트 로직

```rust
let lazy_base_corrected = prev_vpos_end - (y_delta_hu + trailing_ls_hu);
let lazy_base = if lazy_base_corrected >= 0 { lazy_base_corrected }
                else { prev_vpos_end - y_delta_hu };
```

- 컬럼이 vpos≠0 에서 시작(상단 박스/도형 뒤 본문): 보정 적용 → IR 정합 복원.
- sequential 이 IR 을 정확히 추적(drift 0, lazy_base=0): 보정 시 음수 → 비보정 유지
  (표 페이지 over-correction 방지).

## PDF(한컴 2022) 검증

| 문서 | 지표 | PDF | 보정전(PR) | 게이트 |
|------|------|-----|-----------|--------|
| footnote-01 | "개념"→첫불릿 | 85px | 53px | 68→정합(마커 369.4) |
| 복학원서(677) | 본문 시작 band | 214 | 196 | **214 (정확 일치)** |

exam_kor(617)·form-002 골든 불변. 복학원서 골든은 PDF 정합 방향으로 갱신.

## 테스트

- `issue_598_footnote_marker_nav` 4/4 통과
- 전체 스위트 green (golden svg_snapshot 8/8)
- `cargo fmt --all --check` 통과
