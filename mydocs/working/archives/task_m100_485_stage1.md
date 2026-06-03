# Stage 1 보고서 — Task #485 본질 정밀 측정

**작성일**: 2026-05-07
**브랜치**: `local/task485`
**대상**: `samples/synam-001.hwp` 페이지 15·20·21 (PartialTable 분할 셀 마지막 줄 클립)

---

## 1. 측정 방법

`src/renderer/layout/table_layout.rs:2304` 의 `compute_cell_line_ranges` break 조건 직전에 임시 trace 삽입 (env `RHWP_T485_TRACE=1` 게이팅). 각 line 마다 `pi, li, line_h, cum, line_end_pos, abs_limit, diff(=abs_limit - line_end_pos)` 출력.

페이지 15·20·21 의 `export-svg` 실행 후 BREAK / NEAR (gap < 5px) / OK 분류 분석. 측정 종료 후 trace 코드는 revert (현재 소스에 남아있지 않음).

---

## 2. 측정 결과

### 2.1 페이지 15 (PartialTable pi=140 ci=0 rows=6..7)

| 위치 | pi | li | line_h | cum | line_end | abs_limit | diff |
|------|----|----|--------|-----|----------|-----------|------|
| 마지막 OK | 60 | 1 | 23.467 | 2205.867 | **2229.333** | 2245.973 | +16.640 |
| BREAK | 61 | 0 | 23.467 | 2229.333 | 2252.800 | 2245.973 | **-6.827** |
| BREAK | 62..83 | 0 | 23.467 | 2229.333 | 2252.800 | 2245.973 | -6.827 |
| **NEAR (통과)** | **84** | **0** | **14.667** | **2229.333** | **2244.000** | **2245.973** | **+1.973** |

`abs_limit = 1280.6 + 965.4 = 2246.0` (페이지 본문 가용 범위에 해당)

**관찰**:
- pi=60 li=1 까지 정상 적재 후, pi=61..83 (23개 단락) 의 첫 줄이 모두 동일하게 `line_end=2252.8 > abs_limit=2245.973` 으로 BREAK
- pi=84 는 셀의 **마지막 단락** 으로 `is_cell_last_line=true` → `line_h = h (line_spacing 제외) = 14.667px` (다른 줄들의 23.467 보다 작음)
- 작아진 line_h 가 abs_limit 안에 fit (1.973px 여유) → **통과하여 렌더링됨**

### 2.2 페이지 20 (PartialTable pi=163 ci=0 rows=0..1)

| 위치 | pi | li | line_h | cum | line_end | abs_limit | diff |
|------|----|----|--------|-----|----------|-----------|------|
| 마지막 OK | 67 | 0 | 20.000 | 1260.000 | **1280.000** | 1294.453 | +14.453 |
| BREAK | 68..168 | 0 | 20.000 | 1280.000 | 1300.000 | 1294.453 | -5.547 |
| **NEAR (통과)** | **169** | **0** | **13.333** | **1280.000** | **1293.333** | **1294.453** | **+1.120** |

p15 와 동일한 패턴. pi=169 (셀의 마지막 단락, line_h=13.333) 가 통과.

### 2.3 페이지 21 (PartialTable pi=163 ci=0 rows=0..1)

| 위치 | pi | li | line_h | cum | line_end | abs_limit | diff |
|------|----|----|--------|-----|----------|-----------|------|
| **NEAR (마지막 OK)** | **108** | **0** | **20.000** | **2260.000** | **2280.000** | **2280.947** | **+0.947** |
| BREAK | 109..168 | 0 | 20.000 | 2280.000 | 2300.000 | 2280.947 | -19.053 |
| BREAK | 169 | 0 | 13.333 | 2280.000 | 2293.333 | 2280.947 | -12.387 |

p15·p20 와 다름:
- pi=108 의 정상 line (20.000, NOT cell-last) 가 abs_limit 와 0.947px 차이로 가까스로 통과
- pi=169 (cell-last, 13.333) 도 line_end=2293.333 > 2280.947 로 break (slip 발생 안 함)

---

## 3. 본질 — 두 개의 분리된 버그

본 측정으로 **단일 epsilon 마진으로는 해결 안 됨** 을 확인. 두 개의 본질적 버그가 결합:

### 3.1 Bug-1: out-of-order rendering (p15·p20)

`compute_cell_line_ranges` 의 outer 루프 (`for pi in composed_paras`) 가 inner 루프 break 후 **계속 진행** 하여 다음 단락의 첫 줄을 다시 평가:

```rust
for (pi, ...) in composed_paras.iter().enumerate() {
    for (li, line) in comp.lines.iter().enumerate() {
        let line_end_pos = cum + line_h;
        if has_limit && line_end_pos > abs_limit {
            break;  // ← inner loop 만 빠짐 — cum 갱신 안 됨
        }
        cum = line_end_pos;
    }
    result.push(...);  // ← 계속 진행
}
```

`cum` 이 BREAK 시점에 갱신되지 않으므로 다음 단락도 같은 `cum` 에서 시작. 일반적으로 line_h 가 동일하면 모두 BREAK 되어 문제없으나, **셀의 마지막 단락은 `is_cell_last_line=true` 분기로 `line_spacing` 이 제외되어 line_h 가 작아짐** (p15: 23.467→14.667, p20: 20.0→13.333). 이 작은 line_h 가 abs_limit 안에 fit 하여 **통과**.

결과: 셀 마지막 단락(pi=84/169) 이 페이지 N 에서 렌더링되고, pi=61..83/68..168 (시각적으로 그 앞에 있어야 할 단락들) 은 페이지 N+1 로 밀림 → **시각 순서 역전 + 셀 본문 영역 끝 부근에 작은 글자가 남아 클립**.

→ **시각적 클립의 진원**: 통과한 작은 line 의 baseline 이 abs_limit 와 ~1~2px 차이로 cell-clip-rect bottom 에 거의 닿음. baseline 아래 descender 영역(~3px) 이 cell-clip 에 의해 잘림.

### 3.2 Bug-2: boundary epsilon (p21)

p21 에는 cell-last-line slip 이 발생하지 않음 (Bug-1 영향 없음). 대신 일반 line (pi=108 li=0, line_h=20) 이 `abs_limit - 0.947` 위치까지 fit 하여 통과 — **abs_limit 자체가 cell-clip-rect bottom 과 거의 일치** 하므로 line 의 descender 가 clip rect 밖으로 빠져 클립.

`line_end_pos > abs_limit` 의 `>` 가 boundary 처리를 통과시키며, descender 여유분이 없음.

### 3.3 cell-clip-rect 와 abs_limit 정합 (참고)

p21 SVG defs:
```
<clipPath id="cell-clip-6"><rect y="75.573" height="988.373"/></clipPath>  → bottom=1063.947
<clipPath id="body-clip-3"><rect y="75.573" height="994.273"/></clipPath>  → bottom=1069.847
```

마지막 visible text 의 baseline y=1067.847 (cell-clip 보다 3.9px 아래) → descender 영역 (baseline+~3px) 이 cell-clip 에 의해 잘림. body-clip 은 1069.847 까지 허용하므로 body 단위로는 통과. **cell-clip-rect 가 abs_limit 보다 약간 좁음** 도 보조 원인.

---

## 4. 정정 후보 비교

수행계획서 §5 후보 재정의:

### 후보 B' (Bug-1 정정) — 우선

`compute_cell_line_ranges` 의 inner break 를 outer break 로 확장. `limit_reached` 플래그 도입:

```rust
let mut limit_reached = false;
'outer: for (pi, ...) in composed_paras.iter().enumerate() {
    if limit_reached {
        // 이미 abs_limit 도달 — 이 단락은 렌더 안 됨, line range = (line_count, line_count)
        result.push((line_count, line_count));
        continue;
    }
    ...
    for (li, line) in ... {
        ...
        if has_limit && line_end_pos > abs_limit {
            limit_reached = true;
            break;  // inner 만 빠지되, 다음 단락에서 위 가드로 차단
        }
        ...
    }
    result.push((para_start, para_end));
}
```

이로 시각 순서 역전 해소. p15·p20 의 cell-last-line slip 제거.

### 후보 C' (Bug-2 정정) — 차순위

break 조건에 epsilon 마진 추가 — cell-clip-rect 와 abs_limit 의 미세 어긋남(+ descender 여유) 흡수:

```rust
// epsilon = line_h 의 일정 비율 또는 고정 px
const SPLIT_LIMIT_EPSILON: f64 = 2.0;  // 또는 line_h * 0.1
if has_limit && line_end_pos > abs_limit - SPLIT_LIMIT_EPSILON { break; }
```

p21 의 NEAR (gap=0.947) 케이스를 break 시킴.

#### epsilon 값 비교 (측정 데이터 기반)

| epsilon | p15 pi=84 (gap=1.973) | p20 pi=169 (gap=1.120) | p21 pi=108 (gap=0.947) | 회귀 위험 |
|---------|----------------------|-----------------------|-----------------------|----------|
| 0.5px (고정) | 통과 | 통과 | 통과 | 거의 없음 |
| 1.0px (고정) | 통과 | 통과 | 통과 | 거의 없음 |
| **2.0px (고정)** | **break** | **break** | **break** | 낮음 (line_h 의 ~10%) |
| 3.0px (고정) | break | break | break | 낮음~중 |
| line_h × 0.05 | 통과 (0.73) | 통과 (0.67) | 통과 (1.0) | — |
| line_h × 0.1 | 통과 (1.47) | break (1.33) | break (2.0) | 낮음 |
| line_h × 0.15 | break (2.2) | break (2.0) | break (3.0) | 낮음~중 |

**권장**: 후보 C' 는 후보 B' 적용 후 잔여 케이스 (p21) 만 처리. 후보 B' 만으로 p15·p20 해소되지만 p21 은 별도 — 그래서 C' 는 보완 단계로 진행. epsilon 값은 **고정 2.0px** (line_h 약 10%, descender 여유분과 정합) 를 1차 권장.

또는 line_h 비례 (`line_h * 0.1`) 가 폰트 크기 변화에 더 강건 — 단점은 line_h 작은 line (cell-last) 에서 너무 작아져 효과 부족. **결론: 고정 2.0px** 채택 권장.

### 후보 A (typeset split_end_limit epsilon) — 보류

`engine.rs` 4곳의 산정에 epsilon 차감 — 후보 B'·C' 만으로 해소되면 불필요. 회귀 발생 시 검토.

### 후보 D (vpos correction drop) — 보류

선행 collapse 회귀 사례 (`typeset_layout_drift_analysis.md`) 로 신중. 후보 B'·C' 부족 시에만.

---

## 5. 정정 단계 재구성 제안

수행계획서/구현 계획서 §단계 구성 갱신:

| 단계 | 내용 | 변경 |
|------|------|------|
| Stage 2a | 후보 B' (out-of-order 정정) — outer break + limit_reached 플래그 | **신규 (최우선)** |
| Stage 2b | 후보 C' (epsilon 마진) — break 조건에 2.0px 마진 | 기존 Stage 2 의 정정 본 |
| Stage 3 | 회귀 검증 (kps-ai.hwp, synam-001 외 페이지) | 동일 |
| Stage 4 | 최종 보고 | 동일 |

Stage 2 를 a/b 로 분리하여 각각 검증. 2a 만으로 p15·p20 해소 + 회귀 없으면 2b 는 p21 전용 정정.

---

## 6. 검증 데이터 보존

- trace 출력 원본: `/tmp/t485/p15.log`, `/tmp/t485/p20.log`, `/tmp/t485/p21.log` (임시)
- abs_limit 산출 정합: typeset 의 `split_end_limit = avail_content` 출력 = layout 의 `abs_limit - content_offset` 와 일치 (±0.1px)

---

## 7. 작업지시자 승인 요청

1. **본질 진단의 변경**: 단일 epsilon → "out-of-order (Bug-1) + boundary epsilon (Bug-2)" 두 본질로 분리. 동의?
2. **Stage 2 분할**: 2a (Bug-1 정정) + 2b (Bug-2 정정). 동의?
3. **epsilon 권장값**: 고정 2.0px. 또는 `line_h * 0.1` 선호?
4. **후보 A (typeset 측 산정 정정) 보류**: 후보 B'·C' 부족 시에만 진입. 동의?

승인 후 Stage 2a (Bug-1 — out-of-order 정정) 시작.
