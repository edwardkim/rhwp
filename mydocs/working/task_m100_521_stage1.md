# Task #521 Stage 1: 정밀 진단 + 가설 검증 — 완료 보고서

## 핵심 발견 — 2개의 독립 본질

최초 BehindText 가설 → 틀림 (engine.rs:1069-1084 의도적 제외, IR line_height 가 ctrl[1] 영역 포함).

정밀 분석 결과 **2개의 독립적 본질** 식별:

### 본질 1: `tac=true` 표 다음 paragraph 의 host_line_spacing 누락

**`src/renderer/typeset.rs:1230-1237`**:

```rust
let host_line_spacing = if !is_tac {
    para.line_segs.last()
        .filter(|seg| seg.line_spacing > 0)
        .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
        .unwrap_or(0.0)
} else {
    0.0
};
```

`tac=true` 표 (인라인 표) 의 host paragraph 는 `line_spacing` 누락. pi=104 의 `ls[0] line_spacing=344 HU` 가 `host_spacing.after` 에 포함되지 않음 → 다음 문단(pi=105) 이 344 HU = 4.6 px 위로 올라감.

**IR 정합 검증**:
- pi=104 col-vpos 끝 = 2254 + 22207 = 24461
- pi=105 col-vpos 시작 = 24805
- 실측 gap = 344 HU = pi=104 ls[0] line_spacing 정확히 일치

→ IR 자체에 `tac=true` 표의 line_spacing 이 다음 문단 시작에 반영됨을 증명.

### 본질 2: pi=103 (제목) `spacing_after` /2.0 처리 의심

**`src/renderer/style_resolver.rs:658-659`**:
```rust
spacing_before: hwpunit_to_px(ps.spacing_before, dpi) / 2.0,
spacing_after:  hwpunit_to_px(ps.spacing_after,  dpi) / 2.0,
```

주석: *"pyhwpx 확인 결과 동일하게 2배 스케일 저장. 실제 렌더링 시 2로 나누어야 올바른 값이 된다."*

**IR 정합 검증으로 가설 반박**:

pi=103 [PS] `spacing_after=1000`. pi=103 ls[0] vpos=0, 가정 line_height ≈ 1350 HU. pi=103 line bottom ≈ 1350 HU.

| 가정 | pi=104 예상 col-vpos |
|------|---------------------|
| spacing_after = 500 HU (/2) | 1850 HU |
| spacing_after = 1000 HU (full) | 2350 HU |
| **IR 실제** | **2254 HU** |

IR 실제값 (2254) 이 **full 값 가정 (2350) 에 훨씬 근접** (96 HU 차이) — `/2` 가정 (1850) 과는 404 HU 차이.

→ 적어도 pi=103 의 `spacing_after` 는 IR 자체에서 full 값으로 처리됨.

## 결합 효과

1. 본질 2 누락분: ~404 HU = 5.4 px (pi=103 sa 절반만 적용)
2. 본질 1 누락분: 344 HU = 4.6 px (pi=104 ls 미반영)

**총 ~10 px 위로 올라감** — 측정된 ① 위치 차이 (~13 px = 3.5mm) 의 대부분 설명.

## Stage 2 위험 평가 (수정안 별)

### 수정안 A: 본질 1 만 (tac=true 표의 host_line_spacing 포함)

**`typeset.rs:1230-1237`** 수정 — `is_tac` 분기 제거.

- 영향 범위: `tac=true` 인라인 표 다음 paragraph (예제 sample 9건+)
- 위험: **중간** — 다른 인라인 표 케이스 영향
- 검증: SVG snapshot 6 + svg_regression_diff.sh 7 샘플

### 수정안 B: 본질 2 만 (spacing_before/after `/2.0` 제거)

**`style_resolver.rs:658-659`** 수정.

- 영향 범위: 모든 `spacing_before/after != 0` 인 ParaShape (광범위)
- 위험: **매우 큼** — 모든 샘플 layout 변동 가능
- 검증: 전체 회귀 + 작업지시자 시각 검증 다수

### 수정안 C: A + B 동시

- 위험: **매우 큼**
- 두 변경의 상호작용 효과 평가 어려움

### 수정안 D: 본질 1 만 (수정안 A) 진행, 본질 2 는 별도 task 보류

- 본 task 의 범위를 본질 1 로 제한
- 본질 2 는 별도 task 로 광범위 분포 조사 + 단계적 검증 필요
- 본 케이스 (exam_eng p18) 시각 결함 일부 (~4.6 px = 1.2mm) 만 해결, 나머지 5.4 px = 1.4mm 은 별도 task 로 이월

## 권장: 수정안 D

**근거**:
1. 본질 1 (host_line_spacing) 은 수정 위치/조건이 명확하고 위험 제한적
2. 본질 2 (`/2.0`) 는 광범위 영향, "pyhwpx 확인" 주석이 있어 다른 케이스에서는 정합일 가능성 — 무작정 제거 위험
3. 본질 1 만 수정해도 시각 개선 효과 명확 (① 가 4.6 px 아래로 내려가 PDF 와 가까워짐)
4. 본질 2 는 별도 광범위 검증 task 로 분리

## Stage 2 진행 결정 대기

작업지시자 선택:

- **A)** 수정안 A (본질 1 만, 권장)
- **B)** 수정안 B (본질 2 광범위 변경)
- **C)** 수정안 C (둘 다)
- **D)** 수정안 D (수정안 A 진행 + 본질 2 별도 task 등록)
- **E)** 보류 / 별도 layout 리팩터링 흡수
