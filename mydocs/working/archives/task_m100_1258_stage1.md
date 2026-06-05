# Stage 1 완료 보고서 — Task #1258 재현 가드 확립

- **이슈**: edwardkim/rhwp#1258
- **단계**: Stage 1 / 5 (재현 가드)
- **코드 변경**: 없음 (baseline 캡처만)

## 목적

A 정규화는 **동작 무변경 리팩터**이므로, 변경 전 현 동작을 수치로 고정해 무변경 증명 기준선을 만든다.

## 1. 테스트 baseline (green 확인)

| 스위트 | 결과 |
|--------|------|
| `cargo test --lib height_cursor` | **34 passed** (min-gap 3 + single-line 3 + after_tall_line + backtrack 군 포함) |
| `issue_1082_endnote_multicolumn_drift` | **4 passed** (3-09/3-11 실샘플 overflow) |
| `issue_505` | **46 passed** |
| `issue_1139_inline_picture_duplicate` | **9 passed** (미주사이20 참조 포함) |
| 전체 `cargo test` | **2004 passed, 0 failed, 22 ignored** (exit 0 — base clean) |

## 2. 렌더 y baseline (debug-overlay, px) — 핵심 회귀 가드

option B(render 측 정규화)는 렌더 y 를 건드리므로, 변경 후 아래 pi→y 가 **동일해야** 무회귀.
캡처: `output/poc/1258_base/*.svg` (`--debug-overlay`).

### 2.1 문22 — 3-11월_실전_통합_2022.hwp page 14 (#1246 다줄 prev)

| pi | y(px) | 비고 |
|----|-------|------|
| 631 | 427.8 | 문21 content 끝 직전 |
| **632** | **484.3** | **문22 제목 (above-gap 7mm 적용 결과)** |
| 633 | 502.3 | |
| 634 | 536.0 | |

(전체 pi 626~645 스냅샷은 baseline SVG 보존)

### 2.2 미주사이20 — 3-09월_교육_통합_2024-미주사이20.hwp page 10 (#1256/#1261 단일줄 prev)

| pi | y(px) | 비고 |
|----|-------|------|
| 549 | 312.9 | 문9 끝 |
| **550** | **402.9** | **문10 제목** |
| **557** | **618.6** | **문11 제목** |
| **567** | **995.7** | **문12 제목 (overflow 0 유지 대상)** |

(전체 pi 540~569 스냅샷은 baseline SVG 보존)

## 3. 재현 절차 (변경 후 비교)

```
cargo run --bin rhwp -- export-svg "samples/3-11월_실전_통합_2022.hwp" -p 13 --debug-overlay -o output/poc/1258_after/
cargo run --bin rhwp -- export-svg "samples/3-09월_교육_통합_2024-미주사이20.hwp" -p 9 --debug-overlay -o output/poc/1258_after/
# pi→y 가 §2 와 동일하면 무회귀
```

## 4. 다음 단계 (Stage 2)

- render layout 에서 **다줄 prev 마지막 줄 trailing 이 y 누적에서 빠지는 정확한 지점** 특정 (spike)
- 구현안 B/A/C 택1 + 근거

## 승인 요청

Stage 1 baseline 승인 후 Stage 2(spike) 착수.
