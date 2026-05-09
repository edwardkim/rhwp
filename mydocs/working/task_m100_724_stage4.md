# Task #724 Stage 4 단계별 보고서 — HWP3 파서 결함 본질 진단

## 진단 결과 요약

HWP3 native (`samples/hwp3-sample5.hwp`) 페이지 16 paragraph 443 ls[2~6] cs=0/sw=0 결함은 **HWP3 파서 결함** 확정 (composer/paragraph_layout 결함 아님).

본 환경 HWP3 파서 후처리에서 wrap zone 영역 끝 line 의 `seg.segment_width = 0` 강제 설정 → 한컴 IR 정합 위반 (한컴 HWP5 변환본 sw=51024).

## 1. HWP3 native vs HWP5 변환본 IR 비교 (paragraph 443)

| ls index | HWP5 변환본 (한컴 정합) | HWP3 native (본 환경) |
|----------|------------------------|----------------------|
| ls[0/1] | cs=22800/sw=28224 (wrap zone) | cs=21096/sw=29928 (wrap zone) |
| ls[2~] | cs=0/**sw=51024** (col_area 전체) | cs=0/**sw=0** (이상) |

**HWP5 변환본 (한컴 변환) 의 sw=51024 = body_w (col_area 전체 폭)** 가 정상 IR 인코딩.
본 환경 HWP3 파서가 sw=0 으로 인코딩 → composer/paragraph_layout 의 sw=0 fallback 분기 결함으로 좁은 폭 분산 layout.

## 2. HWP3 파서 결함 위치 — `src/parser/hwp3/mod.rs:1705~1710`

```rust
} else if wrap_zone_end_vpos > 0 && acc_section_vpos >= wrap_zone_end_vpos {
    // wrap zone 영역 끝 — cs/sw=0/full 전환
    if seg.column_start > 0 {
        seg.column_start = 0;
        seg.segment_width = 0;  // ← 결함: 한컴 정합은 sw=col_area_width (51024)
    }
}
```

본 환경 HWP3 파서 후처리에서 wrap zone 끝 line 의 cs/sw 를 `(0, 0)` 으로 설정.
한컴 HWP5 변환본 IR 정합은 `(0, col_area_width)` (= body_w = 51024 HU = col_area 전체 폭).

## 3. 본질 정정 방향 (Stage 5)

### 정정 영역

`src/parser/hwp3/mod.rs:1705~1715` 후처리 분기:

```rust
} else if wrap_zone_end_vpos > 0 && acc_section_vpos >= wrap_zone_end_vpos {
    // [Task #724] sw=column_width_hu (col_area 전체 폭) 한컴 IR 정합.
    if seg.column_start > 0 || seg.segment_width == 0 {
        seg.column_start = 0;
        seg.segment_width = column_width_hu;
    }
}
```

추가 정정: line 1424~1430 의 `line_cs_sw` 분기에 `current_zone == None` 시 `(0, column_width_hu)` fallback (안전 가드).

### 회귀 위험 좁힘

- HWP3 파서 한정 정정 (composer/layout 무영향)
- IR 인코딩이 한컴 HWP5 변환본 정합화
- 광범위 sweep 209 fixture 페이지 수 차이 0 검증 필요

### CLAUDE.md 정합

> "HWP3 파서 규칙: src/parser/hwp3/ 내부에서 HWP3 바이너리를 읽어 Document IR로 변환하여 반환한다. HWP3 전용 로직은 반드시 src/parser/hwp3/ 안에서만 구현한다."

본 정정은 HWP3 파서 영역 안에서 IR 정합화 — 본질 룰 정합.

## 4. Stage 5 진행 승인 요청

본 진단 결과 + Stage 5 정정 방향 (HWP3 파서 sw=column_width_hu 정합) 승인 요청.
