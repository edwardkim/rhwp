# Task #969 구현 계획서

- 이슈: [#969](https://github.com/edwardkim/rhwp/issues/969)
- 선행: [Stage 1 진단 보고서](../working/task_m100_969_stage1.md)
- 브랜치: `local/task969`

## 1. 진단 결과 (Stage 1 후속)

`RHWP_TYPESET_DRIFT` 진단으로 정확한 메커니즘 식별:

```
pi=395 (sample16-hwp5):
  HWPX: lh_sum=27.7  ls_sum=10.4  fmt_total=53.2   (ls=10.4 추가)
  HWP5: lh_sum=27.7  ls_sum=0.0   fmt_total=42.8

전체 문서 누적:
  HWPX total fmt_height: 25395.5 px
  HWP5 total fmt_height: 24482.8 px
  Diff:                   +912.7 px ≈ 0.94 페이지 (직접)
  실제 페이지 수 diff:    +10 페이지 (=72-62)
                          (잔여 ~9 페이지 = drift 누적 → 페이지 break 결정 연쇄 → 빈 공간 누적)
```

## 2. Root cause (정밀)

[src/renderer/composer.rs:390-419](src/renderer/composer.rs#L390-L419) — composer 가 preset `line_seg.line_spacing` 을 `ComposedLine.line_spacing` 으로 복사.

[src/renderer/typeset.rs:1190-1229](src/renderer/typeset.rs#L1190-L1229) — `format_paragraph` 의 composed branch:

```rust
let lh = if max_fs > 0.0 && raw_lh < max_fs {
    let computed = match ls_type {
        LineSpacingType::Percent => max_fs * ls_val / 100.0,  // = max_fs * 1.6 (160%)
        ...
    };
    computed.max(max_fs)
} else {
    raw_lh
};
(lh, hwpunit_to_px(line.line_spacing, self.dpi))   // ← line_spacing 그대로 사용
```

HWPX preset `lh=1299 HU (17.3 px)` + `ls=779 HU (10.4 px)` = **합 160% of font 13pt**.

composer 가 ComposedLine 에 lh=17.3, ls=10.4 복사 → format_paragraph 의 `raw_lh(17.3) < max_fs(17.3, FP 동등)` 판정 → `lh = max_fs * 1.6 = 27.7 px` 재계산 → **ls=10.4 별도 가산** → 27.7 + 10.4 = **38.1 px** (= 220% of font, double-count).

## 3. 후보 평가

| 후보 | 설명 | 위치 | Risk | 효과 |
|------|------|------|------|------|
| D1 | HWPX 파서가 lineSegArray emit 안 함 | `src/parser/hwpx/section.rs` | **높** (전체 HWPX layout 변화) | 큼 |
| D2 | format_paragraph 가 preset LineSeg 무시 | `typeset.rs` | **높** (HWP5 도 영향) | 큼 |
| D3 | HWPX 파서의 LineSeg 변환 보정 | `parser/hwpx/` | 중 (preset 의미 변경) | 중 |
| D4 | typeset 이 preset height 만 무시 | `typeset.rs` | 중 | 중 |
| D5 | composed 가 항상 작동하도록 보정 | (불요 — 이미 작동) | — | — |
| **D6** | **format_paragraph 재계산 시 line_spacing=0** | `typeset.rs:1217` | **낮** (정확히 bug site) | **타깃** |
| D7 | composer 가 line_spacing 복사 안 함 (ls_type=Percent) | `composer.rs:390/416/450` | 낮-중 (HWP3 정합 영향) | 중 |

## 4. 확정 후보: D6

**변경 위치**: [src/renderer/typeset.rs:1190-1217](src/renderer/typeset.rs#L1190-L1217)

**변경 내용**:
```rust
let lh = if max_fs > 0.0 && raw_lh < max_fs {
    let computed = match ls_type {
        LineSpacingType::Percent => max_fs * ls_val / 100.0,
        ...
    };
    computed.max(max_fs)
} else {
    raw_lh
};
// [Task #969] lh 재계산이 ls_type 기반 (Percent/SpaceOnly) 으로 발생하면
// preset 의 line_spacing 은 이미 lh 에 포함된 값의 일부이므로 0 으로 처리.
// HWPX (preset lh=font, ls=extra) ↔ HWP5 (preset 없음) 정합.
let effective_ls = if raw_lh < max_fs && max_fs > 0.0 {
    0.0
} else {
    hwpunit_to_px(line.line_spacing, self.dpi)
};
(lh, effective_ls)
```

## 5. 회귀 영향 분석

### 영향 받는 case
- `raw_lh < max_fs` 인 모든 composed line — 즉 preset lh 가 font_size 보다 작은 경우
- 대표적: HWPX 의 `<linesegarray>` 에서 `vertsize=font_size` 로 설정된 paragraph

### 영향 받지 않는 case
- `raw_lh >= max_fs` — preset 이 이미 160% lh 를 담은 경우 (HWP3 의 대부분 case)
- HWP5 의 preset 없음 case (ls 이미 0)
- composed 없는 case (별도 branch)

### 회귀 위험
- HWP3 변종 중 `vertsize=font_size` + 별도 `spacing` 인 paragraph 가 있으면 영향
- 169 샘플 회귀 측정 (Stage 4) 필수

## 6. 보조 fix (D7) — 만약 D6 만으로 부족 시

composer.rs 의 3 군데 (line 390/416/450) 에서 `line_spacing: line_seg.line_spacing` → 조건부 0 으로 변경.

Stage 3 에서 D6 적용 후 페이지 수 측정 → 62~64 페이지로 떨어지면 D6 만으로 충분. 그 외 추가 검토.

## 7. Stage 3 진행 절차

1. D6 패치 적용 (typeset.rs 1217 라인 부근)
2. `cargo build --release`
3. `./target/release/rhwp export-svg samples/hwp3-sample16-hwp5.hwpx -o /tmp/test/`
4. 페이지 수 확인 (목표: 72 → 62~64)
5. `RHWP_TYPESET_DRIFT=1` 으로 pi=395 fmt_total 확인 (목표: 53.2 → 42.8)
6. 단계별 보고서 작성
