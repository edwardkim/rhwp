# Task #1001 Stage 5-A — 추가 진단 + Fix 후보 결정 의사

이슈: [#1001](https://github.com/edwardkim/rhwp/issues/1001)
Stage 4: [`task_m100_1001_stage4.md`](task_m100_1001_stage4.md)

## 1. 추가 진단 결과

### 1-1. HWPX 파서의 HwpUnitChar 처리 — 이미 구현됨

`src/parser/hwpx/header.rs:776-916` 에 HwpUnitChar case 처리 코드 존재:
- `<hp:switch>` `<hp:case namespace="HwpUnitChar">` 우선 처리
- "HwpUnitChar 값은 실제 HWPUNIT(1× 스케일)이므로 HWP 바이너리와 동일한 2× 스케일로 변환"
- 즉 HwpUnitChar (1×) × 2 → HWPUNIT (2×) — rhwp 내부 단위로 변환

**Implication**: rhwp 가 HWPX 의 hp:case HwpUnitChar 값 (852) 을 1704 로 변환. HWP5 binary 의 1704 와 결국 동일. 즉 HWPX 와 HWP5 binary 의 파싱 결과 단위 일치.

### 1-2. HWP3 파서의 spacing 변환 (비교 baseline)

`src/parser/hwp3/mod.rs:195`:
```rust
ps.spacing_before = (hwp3_ps.margin_top as i32) * 4;
```

HWP3 raw `margin_top` × 4 → rhwp 내부 spacing_before.

HWP3 sample16 pi=70: raw margin_top=213 → spacing_before=852 → **한컴 정합 ✓ baseline**.

### 1-3. 가설 검증 — 한컴 viewer 의 단위 해석

HWP3 sample16 pi=70 spacing_before=852 → 한컴/rhwp 시각 정합 (X mm)
HWP5 sample16-hwp5 pi=70 spacing_before=1704 → 한컴 X mm / rhwp 2X mm (회귀)

가설 A 확정 강한 증거:
- HWPX 의 hp:switch HwpUnitChar case 가 852, default 가 1704 → 같은 시각 spacing 으로 의도
- 한컴 viewer 가 HWP5 변환본의 단위를 HwpUnitChar 로 해석하여 1704 → X mm 표시
- rhwp 는 일반 HWPUNIT 으로 해석하여 1704 → 2X mm 표시

**Root cause 가설 A 확정**: 한컴 변환본의 HWP5 binary 가 단위 HwpUnitChar (= HWPUNIT × 2) 로 저장됨. 한컴 viewer 는 변환본 식별 후 단위 보정.

### 1-4. 변환본 식별 신호 후보

| 후보 | 신뢰도 | 검출 가능성 |
|------|--------|-----------|
| 같은 디렉토리의 짝 .hwpx 파일 존재 + hp:switch HwpUnitChar | 높음 | 가능하나 file system 의존 |
| HWP5 file_header flags | 미확인 | 추가 binary 분석 필요 |
| DocInfo / Properties 의 generator metadata | 미확인 | 추가 binary 분석 필요 |
| 일관된 2배 패턴 (전체 ParaShape 통계) | 보통 | 휴리스틱, 일반 HWP5 misid risk |
| 짝 .hwpx 의 hp:switch 존재 → HWP5 binary 도 변환본 결정 | 가장 정확 | file system 의존 + 부재 시 fallback |

### 1-5. Sample 비교 — 변환본 vs 일반 HWP5 패턴

| 파일 | 종류 | pi=70 spacing_before | 비고 |
|------|------|---------------------|------|
| hwp3-sample16-hwp5.hwp | 변환본 | 1704 (2배) | HWP3 sample16 의 변환 |
| hwp3-sample16.hwp | HWP3 원본 | 852 | baseline |
| exam_kor.hwp | 일반 HWP5 | 0 | 패턴 다름 |
| biz_plan.hwp | 일반 HWP5 | 0 | 패턴 다름 |

일반 HWP5 sample 들의 pi=70 spacing 이 0 이라 직접 비교 어려움. 다른 paragraph 들의 패턴 sweep 필요.

## 2. Stage 5 Fix 후보 — 의사 결정 필요

### 옵션 F1: 변환본 식별 + 1/2 보정 (가설 A 기반 fix)

**구현**:
1. HWP5 binary 파서에서 변환본 식별 신호 검출
2. 변환본일 때 ParaShape spacing_before / margin_left/right/etc 를 1/2 보정
3. 단위 변환 후 typeset/layout 단계에서 일관 사용

**Risk**:
- 식별 신호 정확도가 결정적 — 일반 HWP5 misidentification 시 회귀
- HWP3 / HWP5 / HWPX 의 시각 정합이 변환본 식별과 단위 단위 의존
- 가설 A 의 한컴 viewer 내부 메커니즘 추정 — 검증 어려움

**예상 작업량**: 1-2주, 정밀 진단 + 식별 신호 + sweep + 시각 검증

### 옵션 F2: 후속 분리

격차 A (완료) 만 본 Task #1001 에서 처리하고:
- 격차 B (Shape control / paragraph border 렌더링 단순화) — 별도 issue
- 격차 C (HWP5 변환본 단위 해석) — 별도 issue
- 본 Task #1001 close → 격차 A 결과로 closed

**Pros**:
- Stage 3 의 격차 A fix 효과 확정
- 격차 B/C 의 가설 검증 + 위험 작업을 분리하여 안전성 확보
- 사용자가 향후 fix 진행 시 검증된 fix 단위로 진행 가능

**Cons**:
- 사용자 보고된 시각 격차 (변환본 vs 한컴) 가 본 Task 에서 미해결
- 추가 작업이 분산

### 옵션 F3: 진단 + Stage 5 빠른 시도 (proof of concept)

격차 C 의 단위 가설 (1/2 보정) 을 빠르게 시도:
- 환경변수 (`RHWP_HWP5_CONVERT_VARIANT_HALF`) 로 옵트인 1/2 보정 활성화
- 변환본 식별 신호 미구현 — 사용자 명시적 활성화
- 시각 비교로 가설 검증
- 가설 적합 확인 시 식별 신호 + 정식 fix 는 후속 issue 분리

**Pros**:
- 가설 검증 빠름
- Task #1001 본 fix (격차 A) + 격차 C 의 proof of concept

**Cons**:
- 환경변수 의존 fix 는 production 사용 어려움
- 정식 fix 는 별도 작업

## 3. 작업지시자 의사 결정 요청

격차 B/C 의 정식 fix 가 변환본 식별 신호 결정 + 가설 검증 + sweep 검증으로 큰 작업.

| 옵션 | 권고 |
|------|------|
| F1: 정식 fix (대규모, 1-2주) | 사용자 요청에 따라 진행, 회귀 risk 인지 후 |
| F2: 후속 분리 + Task #1001 close (격차 A 만 처리) | 안전 |
| F3: 환경변수 PoC + 정식 fix 분리 | 검증 우선 |

## 4. 잔존 의문 (Stage 5-B 진입 전)

- HWP5 binary 의 한컴 변환본 식별 metadata — 위치 / 형식
- HWPX `hp:switch` 처리가 HWP5 binary 와 결과 일치하므로, HWP5 binary 의 단위 해석이 실제로 다른지 (가설 검증)
- 시험지 / aift / biz_plan 등 일반 HWP5 의 paragraph 별 spacing 패턴 sweep
