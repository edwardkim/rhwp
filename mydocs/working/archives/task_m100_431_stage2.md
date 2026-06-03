# Task #431 Stage 2 — 정정 적용

## 정정 본질

`compute_cell_line_ranges` 의 `content_limit` 비교 단위 mismatch 정합화.

- `line_end_pos = cum + line_h` = **절대 좌표** (셀 시작부터의 누적 px)
- `content_limit` = **상대 길이** (현재 페이지에서 표시할 px)
- 정정: `abs_limit = content_offset + content_limit` 으로 절대 좌표 변환 후 비교

## 변경 영역 (단일 파일 — 3개소)

`src/renderer/layout/table_layout.rs::compute_cell_line_ranges`:

```rust
// [Task #431] 신규: abs_limit 변수 도입
let abs_limit = if has_limit { content_offset + content_limit } else { 0.0 };

// atomic 분기 (라인 ~2129):
let exceeds_limit = has_limit && para_end_pos > abs_limit && !bigger_than_page;

// 일반 line 분기 (라인 ~2168):
if has_limit && line_end_pos > abs_limit {
    break;
}
```

`bigger_than_page` 가드 (Task #362 의 정정 의도) 는 `para_h > content_limit` (상대 길이 비교) 그대로 유지 — 의도 보존.

## 정정 작동 확인

### SVG 출력 (synam-001 페이지 14, 15, 16)

| 페이지 | 정정 전 | 정정 후 |
|--------|---------|---------|
| 14 | 237,476 bytes / 859 text | **351,134 / 1,271 text** ✅ |
| **15** | **3,230 / 6 text (빈 페이지)** | **397,303 / 1,438 text** ✅ **빈 페이지 해소** |
| 16 | 335,623 / 1,210 text | 335,623 / 1,210 text (변화 없음 — `content_limit=0` 케이스) |

페이지 14: 출력량 증가 — 이전 `content_limit=965.4` 로 잘못 cap 되어 일부 paras 가 누락된 정황. 정정으로 `abs_limit=315.1+965.4=1280.5` 까지 표시.

페이지 15: 빈 페이지 → 정상 출력 — 본 결함의 본질 정정.

페이지 16: 무관 (`content_limit=0` 이라 has_limit=false → 분기 미적용).

### dump-pages 정황 (변화 없음)

dump-pages 는 typeset 단계의 PartialTable fragment 정의를 보여줌 — 본 정정은 layout 단계의 단위 mismatch 정합화. typeset 결과는 변화 없음.

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1080 passed** ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |

## 다음 단계

- **Stage 3**: 광범위 회귀 검증 — kps-ai 88→79 페이지 + Task #362 정정 영역 (p56/p67/p68-70/p72-73) 보존 확인
- **Stage 4**: 작업지시자 시각 검증 (synam-001 + kps-ai)
- **Stage 5**: 최종 보고서 + 머지 + close
