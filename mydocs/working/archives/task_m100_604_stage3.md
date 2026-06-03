# Task #604 Stage 3 — HWP3 파서 cs/sw 인코딩 정정 (Issue #604 결함 본질 정정)

## 본 단계 목표

`hwp3-sample5.hwp` page 4 의 pi=75 첫 3 줄 cs=0/sw=0 결함 본질 정정. wrap text 문단의
모든 줄이 anchor 그림의 wrap zone 에 정합 배치되도록 HWP3 파서의 cs/sw 인코딩 로직 수정.

## 결함 본질 진단

### Before Stage 3

```
--- 문단 0.74 (anchor) --- "￼"
  ls[0]: cs=35460, sw=15564  ← 그림 anchor

--- 문단 0.75 (wrap text) ---
  ls[0]: cs=0,     sw=0       ← ❌ 그림 좌측 (x=56.7) 에 텍스트
  ls[1]: cs=0,     sw=0       ← ❌
  ls[2]: cs=0,     sw=0       ← ❌
  ls[3]: cs=35460, sw=15564   ← ✓
  ls[4~]:          cs=35460, sw=15564
```

### 본질 분석 (`src/parser/hwp3/mod.rs:1399-1407`)

기존 가드:
```rust
if pic_wrap_zone.is_some() || (linfo.pgy >= pgy_start && linfo.pgy < pgy_end) {
    Some((cs, sw))
} else {
    None
}
```

pi=74 anchor 그림: `vert_rel_to=Paper`, `vertical_offset=5720HU(20.2mm)`, `height=26788HU`.
- pgy_start = 5720/4 = 1430 HU
- pgy_end = 1430 + 6697 = 8127 HU

pi=75 의 첫 3 줄 `linfo.pgy` 가 1430 HU **미만** (paper top 근처에서 시작) → `pgy >= pgy_start`
가드 실패 → cs/sw=0 설정. 4번째 줄부터 pgy >= 1430 진입 → cs/sw 적용.

본질: **wrap text 문단의 첫 줄이 anchor 의 pgy_start 보다 위쪽에서 시작하는 케이스**
가 정상. 한컴뷰어는 본 케이스에서도 모든 줄을 그림 우측에 정합 배치.

## 정정 영역 (옵션 3a)

`pgy_start` 가드 제거 — `pgy_end` 만 검사하는 단방향 가드:

```rust
let line_cs_sw = current_zone.and_then(|(cs, sw, _pgy_start, pgy_end)| {
    if pic_wrap_zone.is_some() || linfo.pgy < pgy_end {
        Some((cs, sw))
    } else {
        None
    }
});
```

본질:
- `pic_wrap_zone.is_some()`: anchor 문단 자체 — pgy 무관 적용 (기존 보존)
- `linfo.pgy < pgy_end`: 그림 아래로 흘러간 줄만 wrap zone 외 판정 (cs=0 정상)
- 그림 위쪽에서 시작한 줄도 wrap zone 의 일부 → cs/sw 적용

## 검증 결과

### After Stage 3

```
--- 문단 0.75 (wrap text) ---
  ls[0]: cs=35460, sw=15564  ← ✓ (Stage 3 정정)
  ls[1]: cs=35460, sw=15564  ← ✓ (Stage 3 정정)
  ls[2]: cs=35460, sw=15564  ← ✓ (Stage 3 정정)
  ls[3~]: cs=35460, sw=15564  ← ✓ (이전부터 정합)
```

### 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build` | ✅ |
| `cargo test --lib` | ✅ **1130 passed** / 0 failed / 2 ignored |
| `cargo test --test issue_546` (Task #546) | ✅ 1 passed (exam_science 4페이지) |
| `cargo test --test issue_554` (HWP3 변환본) | ✅ 12 passed |
| `cargo test` (통합 31) | ✅ 모두 통과 |

### 회귀 검증

| 영역 | 결과 |
|------|------|
| `exam_science.hwp` | ✅ 4페이지 / 단 0 items=37 (Task #546 정합) |
| `hwp3-sample.hwp` | ✅ 16페이지 (회귀 0) |
| `hwp3-sample5.hwp` | ✅ 64페이지 (회귀 0) |

### 시각 판정 자료

`output/svg/task604_after/hwp3-sample5/hwp3-sample5_{004,008,016,022,027}.svg`:

| 페이지 | x>380 분포 (상위 3개) | Stage 3 정합 |
|------|---------------------|------------|
| 4 | (725, 27), **(529, 23)**, (713, 19) | ✅ x=529 23개 (PR #589: 20개, +3 = pi=75 첫 3 줄) |
| 8 | (725, 23), (713, 13), **(384, 13)** | ✅ x=384 13개 (Pattern A) |
| 16 | (725, 15), (713, 13), (701, 8) | ✅ |
| 22 | **(407, 23)**, (725, 21), (419, 18) | ✅ x=407 23개 |
| 27 | (725, 18), (713, 15), (701, 14) | ✅ |

## LOC 합계

| 파일 | 변경 |
|------|-----|
| `src/parser/hwp3/mod.rs` | -2 / +14 (가드 정정 + Task #604 Stage 3 본질 주석) |
| **소스 합계** | **-2 / +14 (+12 LOC)** |

## 본 단계의 본질적 가치

1. **Issue #604 결함 본질 정정**: pi=75 첫 3 줄 cs/sw=0 → 정합 인코딩
2. **시각 정합**: hwp3-sample5.hwp page 4 텍스트가 그림 우측에 정합 배치 (한컴뷰어 정합)
3. **HWP3 파서의 LineSeg 인코딩 표준 정합**: `mydocs/tech/document_ir_lineseg_standard.md`
   §"HWP3 파서" 책임 정합 — wrap zone 영역의 모든 줄 cs/sw 정확히 인코딩

## 작업지시자 승인 요청

본 Stage 3 (HWP3 파서 cs/sw 인코딩 정정) 완료 보고. 다음 단계 (Stage 4: 광범위 회귀 검증
+ 시각 판정 ★) 진입 승인 요청.

## 참조

- 수행계획서: `mydocs/plans/task_m100_604.md`
- 구현계획서: `mydocs/plans/task_m100_604_impl.md`
- LineSeg 표준: `mydocs/tech/document_ir_lineseg_standard.md`
- Stage 1 보고서: `mydocs/working/task_m100_604_stage1.md`
- Stage 2 보고서: `mydocs/working/task_m100_604_stage2.md`
- Stage 2b 보고서: `mydocs/working/task_m100_604_stage2b.md`
- Issue: #604
