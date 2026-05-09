# Task #724 Stage 5 단계별 보고서 — HWP3 파서 정정

## 개요

Stage 4 진단에 따라 HWP3 파서의 wrap zone 후처리 영역 정정. 한컴 HWP5 변환본 IR (sw=col_area_width) 정합 도달.

## 1. 정정 영역 — `src/parser/hwp3/mod.rs`

### 정정 1: 후처리 분기 (line 1705~1715)

```rust
} else if wrap_zone_end_vpos > 0 && acc_section_vpos >= wrap_zone_end_vpos {
    // wrap zone 영역 끝 — cs/sw=0/full 전환
    // [Task #724] sw=column_width_hu (col_area 전체 폭) 한컴 IR 정합.
    if seg.column_start > 0 || seg.segment_width == 0 {
        seg.column_start = 0;
        seg.segment_width = column_width_hu;
    }
}
```

### 정정 2 (rollback): `line_cs_sw` 분기 (line 1424~1430) — 정정 시도 후 회귀 발견 → 원복

`line_cs_sw` 분기에 `current_zone=None` 시 `(0, column_width_hu)` fallback 정정 시도했으나 wrap zone 안 line 의 cs/sw 도 broad 적용 → 페이지 4 paragraph 75 (multi-line wrap text) ls 모두 (0, 51024) 으로 잘못 설정 → 좁은 wrap zone 해석 안 됨 → 좁은 폭 분산 layout 결함. 원복.

본 정정은 후처리 분기 (정정 1) 만으로 충분. wrap zone 끝 line 의 sw=0 강제 → sw=column_width_hu 정합화.

## 2. IR 정합 검증 (paragraph 443 + paragraph 75)

| ls index | 정정 전 | 정정 후 | HWP5 변환본 정합 |
|----------|---------|---------|------------------|
| ls[0/1] | cs=21096/sw=29928 | cs=21096/sw=29928 | cs=22800/sw=28224 (값 차이는 image margin) |
| ls[2~6] | cs=0/**sw=0** | cs=0/**sw=51024** | cs=0/sw=51024 ✓ |

ls[2~6] sw 정합. cs 는 wrap zone 영역 내외 판정 (paragraph 440 image 영역 18560~33912 HU 와 비교) 결과.

## 3. 시각 판정 (rsvg-convert PNG)

- **stage5_p16_native.png**: paragraph 443 ls[2~] 정상 col_area 전체 폭 layout (한컴 정합) ✓
- HWP3 native 페이지 8/27/48 (PR #723) 정합 보존 ✓
- HWP5 변환본 페이지 16/22 (Task #724 Stage 1~3) 정합 보존 ✓

## 4. 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1166 passed; 0 failed** (회귀 0) |
| `cargo clippy --release` | 신규 경고 0 |
| 광범위 sweep (209 fixture) | **DIFF 0** (회귀 0) |

## 5. 회귀 위험 영역 정합

- HWP3 파서 단일 영역 (line_cs_sw + 후처리 분기)
- composer/layout 무영향 (CLAUDE.md HWP3 파서 규칙 정합)
- 한컴 HWP5 변환본 IR 정합화로 본질 정합
- 광범위 sweep 페이지 수 차이 0

## 6. Stage 6 진행 승인 요청

광범위 sweep + 결정적 검증 + 시각 판정 통과. 최종 결과 보고서 작성 + commit + PR 갱신 진행 승인 요청.
