# Task #672 Stage 2 단계별 보고서 — 본질 정정

## 1. 정정 위치 (Stage 1 진단 결과 반영)

**옵션 1 (2% 임계값) 적용** — TAC 표 비례 축소 임계값 강화로 작은 차이 (≤2%) 면제.

| 파일 | 변경 종류 | 영역 |
|------|----------|------|
| `src/renderer/height_measurer.rs:805-822` | 임계값 가드 추가 | 단일 분기 |

## 2. height_measurer.rs 정정 내용

```rust
// [Task #672] TAC 표 비례 축소 임계값 강화 — 작은 차이 (≤2%) 는 면제.
//
// 본질: 셀 콘텐츠 측정값과 common.height 의 미세한 불일치 (측정 오차
// 또는 line_height 보정 부산물) 시 비례 축소가 셀 콘텐츠 클립을 발생.
// 한컴 뷰어는 작은 차이를 비례 축소 안 함 (계획서.hwp 1.32% 차이 — 3 줄
// 정상 표시). 2% 이상 차이는 사용자 의도 영역 (의도적 압축) 으로 간주
// 하여 기존 동작 유지.
//
// 발동 영역 sweep 진단 (187 fixture): ≤2% 7 건 면제, ≥5% 11 건 그대로.
const TAC_SHRINK_THRESHOLD_RATIO: f64 = 0.02;
let shrink_threshold = (common_h * TAC_SHRINK_THRESHOLD_RATIO).max(1.0);
let table_height = if table.common.treat_as_char && common_h > 0.0
    && raw_table_height > common_h + shrink_threshold {
    let scale = common_h / raw_table_height;
    for h in &mut row_heights {
        *h *= scale;
    }
    common_h
} else {
    raw_table_height
};
```

### 면제 효과

본 정정으로 면제되는 영역:

```
계획서.hwp pi=0    | 1.32% (12.76 px) — 본 case
2010-01-06.hwp     | 1.97% (6.93 px)
hwp-img-001.hwp    | 1.22% (3.76 px)
kps-ai.hwp         | 1.88% (3.73 px)
hwp-3.0-HWPML.hwp  | 0.16% (1.09 px)
synam-001.hwp      | 0.38%, 0.94% (3.76, 2.99 px)
```

5% 이상 차이 (의도적 큰 압축) 는 기존 동작 그대로 — 회귀 위험 좁힘.

## 3. 결정적 검증

| 검증 영역 | 결과 |
|----------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 0 warnings |

## 4. 시각 판정 (작업지시자)

`samples/계획서.hwp` 1 페이지 표:

| 영역 | 결과 |
|------|------|
| 셀 [21] "목적" 3줄 SVG 그려짐 | ✅ |
| 셀 [21] 마지막 줄 PNG 시각 표시 | ❌ — Issue #674 영역 |
| 다른 셀 회귀 0 | ✅ |

**본 task #672 본질 영역 (TAC 표 비례 축소 면제) 정정 완료** ✅.

## 5. 잔존 결함 — 별도 Issue 분리

본 task #672 정정 후에도 셀 [21] / [52] 의 마지막 줄/paragraph 가 시각적으로 보이지 않는 결함 잔존.

### 진단 결과

- height_measurer row_heights[5] = 67.76 (3줄 + pad, 정확) ✅
- layout cell_h = 67.76, recompose 3줄 분할 ✅
- paragraph_layout 줄 layout 정상 (줄 2 끝 y=443.37 = col_area end_y)
- SVG에 3 줄 baseline 모두 그려짐 (y=395.37/411.37/432.71)

### 잔존 결함의 본질

paragraph_layout 줄 layout 위치는 `col_area.y + line_idx * line_height` 로 결정 — `row_heights` 와 무관. 측정값을 늘려 cell BoundingBox 를 확장해도 줄 위치는 변하지 않음. 마지막 줄 baseline + glyph descender 가 cell 외곽 BoundingBox 안에 있는데도 PNG 변환 시 클립 발생.

### descent 여유 시도 결과

- `max_fs * 0.15` (2px): 효과 없음
- `max_fs * 0.5` (6.67px): 마지막 줄 일부 보임 (다음 행 영역 침범 — 시각 회귀)

→ row_heights 측정값 증가만으로 해결 불가. paragraph_layout 의 줄 위치 결정 로직 자체의 본질 영역.

### 별도 Issue 등록

[Issue #674](https://github.com/edwardkim/rhwp/issues/674) — "paragraph_layout 줄 위치 vs row_heights 정합 — line_segs 부재 paragraph 마지막 줄 시각 클립"

`feedback_hancom_compat_specific_over_general` + 회귀 위험 좁힘 영역 정합 — 본 task #672 본질 영역 (TAC 표 비례 축소) 과 다른 본질 (paragraph_layout 줄 위치) 으로 분리.

## 6. Stage 3 진행 영역

- 광범위 페이지네이션 회귀 sweep (187 fixture)
- 결정적 검증 (cargo test, clippy)
- 최종 보고서 작성

## 7. Stage 3 진행 승인 요청

본 Stage 2 결과 + 결정적 검증 + 잔존 결함 별도 Issue 분리 후 Stage 3 광범위 sweep 진행 승인 요청.
