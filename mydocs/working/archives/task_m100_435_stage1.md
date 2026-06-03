# Task #435 Stage 1: 베이스라인 측정 + 진단 데이터 수집 — 완료보고서

> **이슈**: [#435](https://github.com/edwardkim/rhwp/issues/435)
> **브랜치**: `local/task435`
> **단계**: 1/5 (베이스라인 측정 + 진단)
> **완료일**: 2026-04-29
> **코드 변경**: 없음 (계측만)

---

## 통과 조건 체크

- [x] `output/debug/task435/exam_kor_baseline.csv` — 페이지별 단별 used/hwp_used/diff 24페이지 전체 수집
- [x] 회귀 대상 4 문서 페이지 수 기록
- [x] `pi=0.30`, `pi=1.25` typeset cur_h 진행 로그 캡처
- [x] reserve 산정 값 (col 1 시작 시점) 수치 확보 + 산정 경로 추적

---

## 산출물

| 파일 | 내용 |
|---|---|
| `output/debug/task435/exam_kor_baseline.csv` | exam_kor.hwp 24 페이지 × 단별 used / hwp_used / diff |
| `output/debug/task435/regression_baseline.txt` | 회귀 대상 5 문서 페이지 수 |
| `output/debug/task435/typeset_drift_baseline.txt` | RHWP_TYPESET_DRIFT 전체 출력 (748 lines) |
| `output/debug/task435/typeset_drift_key.txt` | 핵심 paragraph 추출 (col 1 시작, pi=0.30, pi=1.25) |

---

## 결과 요약

### exam_kor 베이스라인 (24 페이지)

| 페이지 | 단 0 used | 단 1 used | 단 0 diff | 단 1 diff | 비고 |
|---|---|---|---|---|---|
| 1 | 1056.1 | 1207.6 | -117.4 | **+54.5** | pi=0.30 split 시작 |
| 2 | 49.0 | — | — | — | **orphan (pi=0.30 lines 1-2)** |
| 3 | 1179.9 | 973.9 | -12.2 | -132.8 | |
| 4 | 957.8 | 995.3 | -250.1 | -212.1 | 누적 -462px |
| 5 | 1149.2 | 873.4 | -42.8 | -333.6 | 누적 -376px |
| 7 | 1036.3 | 965.9 | -169.8 | -229.3 | |
| 8 | 933.1 | 1087.2 | -272.5 | -119.5 | |
| 10 | 990.7 | 996.7 | -218.0 | -212.9 | |
| 12 | 730.4 | 927.1 | -268.5 | -276.7 | |
| 13 | 677.4 | 883.2 | -329.0 | -130.3 | |
| 14 | 1080.7 | 1193.5 | -117.4 | **+106.2** | pi=1.25 split 시작 |
| 15 | 104.2 | — | — | — | **orphan (pi=1.25 lines 1-2 + pi=26, 27)** |
| 16 | **1225.8** | 64.3 | overflow | -1124.7 | Square wrap 표 4개 |
| 19 | 833.2 | 946.8 | -373.1 | -50.5 | |
| 20 | 1209.1 | 370.5 | +109.1 | -803.0 | |
| 22 | 1082.6 | 1051.2 | -126.3 | -153.7 | |
| 23 | 1017.2 | 946.4 | -153.8 | -259.0 | |

### 회귀 대상 페이지 수 베이스라인

```
exam_kor.hwp:    24페이지 (목표: 20)
exam_eng.hwp:     8페이지 (유지)
k-water-rfp.hwp: 28페이지 (유지) ← 수행계획서의 27 보다 1 많음, 정정
hwpspec.hwp:    177페이지 (유지)
synam-001.hwp:   35페이지 (유지) ← #431 별도
```

---

## 핵심 진단: col 1 reserve 과대 산정 (#393 옵션 A)

### TYPESET_DRIFT 데이터

```
pi=12 col=1 ... first_vpos=7085 ... cur_h=306.1 avail=1201.3   (section 0, page 1)
pi=12 col=1 ... first_vpos=7085 ... cur_h=306.1 avail=1201.3   (section 1, page 14)
pi=30 col=1 ... fmt_total=78.1   ... cur_h=1178.5 avail=1201.3 → split
pi=25 col=1 ... fmt_total=49.0   ... cur_h=1169.0 avail=1201.3 → split
```

- `cur_h=306.1` = col 1 시작 reserve (양 섹션 동일)
- HWP 실제 col 1 시작점: vpos=7085 HU = **94.5 px**
- **과대 산정량: 306.1 - 94.5 = +211.6 px**

### pi=0.0 (section 0 첫 문단) 의 body-wide 표

```
[4] 표: 1행×1열, vert=Paper(10771=38.0mm), wrap=TopAndBottom
       size=66616×11057 HU (235.0×39.0mm)
       outer_margin: bottom=4.0mm (1131 HU)
```

본문 영역: `body_top` = margin_top 56mm = 211.7 px, `body_h` = 1211.3 px

### `compute_body_wide_top_reserve_for_para` 산정 경로 (`typeset.rs:2127-2172`)

```rust
let shape_y_offset = hwpunit_to_px(common.vertical_offset)  // 10771 HU = 143.6 px (Paper-rel)
if shape_y_offset > body_h / 3.0 { continue; }              // 143.6 < 403.8 → 통과
let shape_h = ...                                            // 11057 HU = 147.4 px
let outer_bottom = ...                                       // 1131 HU = 15.1 px
let bottom = shape_y_offset + shape_h + outer_bottom         // 143.6 + 147.4 + 15.1 = 306.1
```

### 버그

`shape_y_offset` 은 `vert_rel_to == VertRelTo::Paper` 일 때 **Paper(용지) 기준** 좌표. 즉 용지 상단부터 143.6 px (= 38.0mm). 본문 상단 211.7 px 보다 **위쪽** (header 영역).

표가 차지하는 본문 영역 침범 폭은 실제로는:
```
shape_bottom_paper = shape_y_offset + shape_h = 143.6 + 147.4 = 291.0 px
shape_invades_body = max(0, shape_bottom_paper - body_top) = max(0, 291.0 - 211.7) = 79.3 px
정확한 reserve = shape_invades_body + outer_bottom = 79.3 + 15.1 = 94.4 px
```

이 값 (94.4) 은 HWP 의 실제 col 1 시작점 (94.5) 과 거의 일치 (오차 0.1px).

코드 (`typeset.rs:2148-2155`) 는 Paper-rel 가드로 `shape_bottom_abs <= body_top` 인 경우만 skip 하지만, body 와 일부만 겹치는 경우 (`shape_top_abs < body_top < shape_bottom_abs`) 에 대해 **body_top 차감 없이** 그대로 reserve 에 더함. 이것이 +211.6 px 과대 산정의 직접 원인.

### 영향

- pi=0.30 (page 1 단 1): `cur_h=1178.5 + fmt=78.1 = 1256.6` → avail 1201.3 초과 55.3px → split
  - 정정 후 예상: `cur_h ≈ 966.9 + 78.1 = 1045.0` → 156.3px 여유로 fit, **page 2 orphan 해소**
- pi=1.25 (page 14 단 1): `cur_h=1169.0 + 49.0 = 1218.0` → avail 1201.3 초과 16.7px → split
  - 정정 후 예상: `cur_h ≈ 957.4 + 49.0 = 1006.4` → 194.9px 여유로 fit, **page 15 orphan 해소**

---

## Stage 2 입력

### 수정 대상

`src/renderer/typeset.rs:2127-2172` `compute_body_wide_top_reserve_for_para`

### 수정 방향 (가설)

`VertRelTo::Paper` 분기에서 `shape_top_abs < body_top < shape_bottom_abs` 인 경우 body 와 겹치는 부분만 reserve 에 반영:

```rust
let shape_y_offset = hwpunit_to_px(common.vertical_offset, dpi);
let shape_h = hwpunit_to_px(common.height, dpi);

// Paper-rel 좌표를 body-rel 로 변환
let effective_y = if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let shape_top_abs = shape_y_offset;
    let shape_bottom_abs = shape_top_abs + shape_h;
    if shape_bottom_abs <= body_top { continue; }
    // body 와 겹치는 시작 y (body 좌표계)
    (shape_top_abs - body_top).max(0.0)
} else {
    shape_y_offset
};

let effective_h = if matches!(common.vert_rel_to, VertRelTo::Paper) {
    // body 안에서 표가 차지하는 높이
    let shape_bottom_abs = shape_y_offset + shape_h;
    shape_bottom_abs - shape_top_abs.max(body_top)
} else {
    shape_h
};

if effective_y > body_h / 3.0 { continue; }
let bottom = effective_y + effective_h + outer_bottom;
```

### 회귀 우려 케이스

- **vert_rel_to=Para 또는 Body 인 body-wide 표/도형** (대부분의 일반 케이스): `shape_y_offset` 이 이미 body-rel 이므로 기존 동작 유지 필요. Paper 분기에서만 변환.
- **bbox 전체가 body 위쪽 (header) 만 점유** (`shape_bottom_abs <= body_top`): 기존 skip 로직 유지.
- **bbox 전체가 body 안 (header 영역 무관)** (`shape_top_abs >= body_top`): `effective_y > 0`, `effective_h = shape_h` — 기존과 동일.

---

## 위험 / 주의 사항

1. **Task #386 (Paper-rel 좌표계 가드)** 가 이미 부분적으로 처리 — 위 코드의 line 2148-2155 가 그 결과. 그러나 body 일부 침범 케이스는 누락. Stage 2 에서 정정.
2. **layout 측의 `calculate_body_wide_shape_reserved`** (`layout.rs:1178`) 도 동일 로직을 가져야 일관성 유지. Stage 2 에서 동시 정정 필요.
3. **임시 디버그 println 추가 안 함** — 기존 `RHWP_TYPESET_DRIFT` 로 충분히 추적 가능. Stage 2 진입 시 추가 디버그 추가 후 Stage 종료 전 제거.

---

## 다음 단계

**Stage 2 — #393 (1) col 1 reserve 정정** 진입. 수정 대상은 `compute_body_wide_top_reserve_for_para` 와 `calculate_body_wide_shape_reserved` 의 Paper-rel 분기.

승인 요청 후 진행.
