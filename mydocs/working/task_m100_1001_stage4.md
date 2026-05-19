# Task #1001 Stage 4 — 격차 B/C 정밀 진단 보고서

이슈: [#1001](https://github.com/edwardkim/rhwp/issues/1001)
Stage 1-3: [`stage1`](task_m100_1001_stage1.md), [`stage2`](task_m100_1001_stage2.md), [`stage3`](task_m100_1001_stage3.md)

## 1. 진단 대상 / 자료

- `samples/hwp3-sample16-hwp5.hwp` (HWP5 binary 변환본) — rhwp-studio 의 실제 입력
- `samples/hwp3-sample16-hwp5.hwpx` (HWPX 변환본) — 한컴 변환 시 함께 생성된 XML 산출물 (rhwp 참조용)
- `samples/hwp3-sample16.hwp` (HWP3 원본) — 한컴 정합 baseline
- `rhwp dump-pages -p 2` (사업개요 페이지)
- `rhwp dump -s 0 -p 70` (paragraph 상세)
- HWPX header.xml 직접 unzip + grep

## 2. 격차 C 진단 — Paragraph spacing drift (Root cause 확정)

### 2-1. ParaShape 직접 비교 (pi=70 "1. 추진목적")

| 항목 | HWP3 원본 | HWP5 변환본 | 비율 |
|------|----------|------------|------|
| text | "═══■═══1.추진목적═══■═══" | "1. 추진목적" | - |
| ps_id | 90 | 46 | 다름 |
| spacing_before | **852** | **1704** | **2x** |
| margins.left | **2000** | **4000** | **2x** |
| border_fill_id | 0 | 1 | - |

### 2-2. HWPX `<hp:switch>` 의 분기 구조

`paraPr id="46"` (HWP5 변환본의 pi=70 ParaShape):

```xml
<hp:switch>
  <hp:case hp:required-namespace="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar">
    <hh:margin>
      <hc:left value="2000" unit="HWPUNIT"/>    ← HWP3 정합 값
      <hc:prev value="852" unit="HWPUNIT"/>     ← HWP3 정합 값
    </hh:margin>
  </hp:case>
  <hp:default>
    <hh:margin>
      <hc:left value="4000" unit="HWPUNIT"/>    ← 2배 값
      <hc:prev value="1704" unit="HWPUNIT"/>    ← 2배 값
    </hh:margin>
  </hp:default>
</hp:switch>
```

### 2-3. Root cause

한컴 변환 시:
- HWPX 에는 `<hp:switch>` 분기 구조로 두 값 모두 저장
- `<hp:case namespace="HwpUnitChar">`: HWP3 정합 값 (한컴 2016+ HwpUnitChar 지원 viewer)
- `<hp:default>`: 2배 값 (HwpUnitChar 미지원 구 viewer / 다른 viewer)
- HWP5 binary 에는 단일 값만 저장 가능 → 2배 (default) 만 저장

**rhwp 동작**:
- HWPX 파서: `hp:switch` 무시, `hp:default` 만 읽음 → 2배 값 사용
- HWP5 binary 파서: 단일 값 (2배) 그대로 사용

**한컴 viewer 동작**:
- HWPX 열 때: `HwpUnitChar` namespace 지원 → `hp:case` 사용 → HWP3 정합 ✓
- HWP5 binary 열 때: 어떤 보정 로직으로 정합 표시 (확인 필요)

### 2-4. 잔존 의문 — HWP5 binary 단일 값 처리

HWP5 binary 의 ParaShape spacing_before=1704 가 한컴 viewer 에서 HWP3 의 852 와 동일 spacing 으로 표시되는 메커니즘:
- 가설 A: 한컴 viewer 가 HWP5 binary 의 단위를 HwpUnitChar 로 해석 (HWPUNIT 의 1/2)
- 가설 B: 한컴 변환본 식별 metadata 가 binary header 어딘가에 있고 viewer 가 이를 인지 후 1/2 보정
- 가설 C: HWP5 binary 의 spacing 이 절대값 (mm 등) 으로 다르게 인코딩
- → Stage 5 fix 후보 평가 시 가설 검증 필요

## 3. 격차 B 진단 — Styling 단순화

### 3-1. text "■ ... ■" 출처

- **HWP3 원본** pi=70 text: `"════════════════════■ 1.추진목적 ■══════════..."` (text 에 직접 장식 포함)
- **HWP5 변환본** pi=70 text: `"1. 추진목적"` (장식 strip 됨)

한컴이 변환 시 paragraph text 에서 장식 문자를 strip 함. **rhwp 의 SVG export 출력에도 "■" 없음** (`/tmp/diag_b/hwp3-sample16-hwp5_003.svg` 확인). 즉 native rhwp 는 정합.

사용자 화면의 "■" 는 image 1 (HWP3 원본) 에서 보인 것이며, image 2 (HWP5 변환본 rhwp-studio) 에서 보인 것은 다른 시각 요소 (아래 3-2 참조).

### 3-2. 회색 띠 / 점선 박스 / 그라데이션 출처

SVG `/tmp/diag_b/hwp3-sample16-hwp5_003.svg` 에서:

```xml
<rect x="103.36" y="163.78" width="611.7" height="130.18" 
      rx="6.5" ry="6.5" 
      fill="url(#grad1)" 
      stroke="#000000" stroke-width="1" stroke-dasharray="2 2"/>
```

- 위치 (y=163~294) = `1. 추진목적` (y=147 baseline) 바로 아래
- pi=71 의 Shape control (vpos=7464, h=130.2, wrap=TopAndBottom, tac=true)
- 그라데이션 fill (`#grad1`) + 점선 외곽 (`dasharray="2 2"`) + rounded corner (rx=6.5)

이게 사용자 image 2 (rhwp-studio) 의 **본문 점선 박스** 출처. HWP3 원본 (image 1) 도 같은 도형을 가지지만 다른 wrap 으로 (InFrontOfText vs TopAndBottom) 가 변환본에서 다름.

### 3-3. dump-pages 비교 (pi=71)

| 항목 | HWP3 원본 | HWP5 변환본 |
|------|----------|------------|
| Shape control wrap | InFrontOfText | TopAndBottom |
| vpos | 5760 | 7464 |
| h | 130.2 | 130.2 |
| tac | true | true |

`wrap=TopAndBottom` 으로 변경되어 paragraph 흐름과 충돌. 한컴이 어떤 방식으로 이를 그리는지 (또는 안 그리는지) 추가 확인 필요.

### 3-4. 외곽선 박스 차이 (상단)

- 한컴 (image 3): 페이지 상단에 외곽선 박스, "Ⅰ. 사업개요" 포함
- rhwp HWP5 변환본 (image 2): **유사하게** 외곽선 박스 그려짐 — Stage 3 fix 효과로 정합

격차 A fix 이후 외곽선 박스는 한컴 정합. 격차 B 의 잔존은 **내부 paragraph border / Shape control rendering** 차이.

### 3-5. 격차 B 의 본질

격차 B 는 **HWP3 → HWP5 변환 시 한컴이 적용하는 변환 규칙** 의 결과:
1. paragraph text 에서 장식 문자 strip
2. paragraph 의 `border_fill_id` 부여 (NONE 의 dummy 외곽선)
3. Shape control 의 wrap 변경 (InFrontOfText → TopAndBottom)
4. ParaShape spacing 2배 (격차 C 와 같은 원인)

한컴 viewer 는 변환본을 표시할 때 이러한 변환 결과를 다시 한컴 정합 형식으로 표시하는 추가 로직 보유. rhwp 는 변환본의 raw data 를 그대로 그림 → 격차 발생.

## 4. 변환본 식별 신호 진단

HWP5 변환본 vs 일반 HWP5 구분 신호:
- ParaShape spacing 의 2배 패턴 (격차 C 의 직접 증거)
- pgbf.attr=0x01 + bit1=0 + bit2=0 (격차 A 의 한컴 default)
- HWPX 의 `<hp:switch>` 존재 (한컴 변환 산출물 명시 신호, HWPX 만)
- File metadata (확인 필요): generator, creator, version 등

추가 조사 필요 — Stage 5 에서 정확도 측정 후 신호 선정.

## 5. Fix 방향 (Stage 5 후보)

### 5-1. 격차 C Fix 후보

**C1. HWPX `hp:switch` 처리 추가** (HWPX 전용)
- `hp:case namespace="HwpUnitChar"` 우선 처리
- HWP5 binary 무관 (다른 fix 필요)

**C2. HWP5 binary 변환본 식별 + spacing 1/2 보정**
- 변환본 식별 신호로 ParaShape spacing 자동 보정
- 회귀 risk: 일반 HWP5 misidentification

**C3. HWP5 binary 의 HwpUnitChar 단위 가설 검증** (가설 A 기반)
- 한컴 변환본의 spacing 단위가 HWPUNIT/2 라 가정
- ParaShape 파싱 시 단위 보정

### 5-2. 격차 B Fix 후보

**B1. Shape control wrap=TopAndBottom + tac=true + 그라데이션 fill 의 한컴 정합 처리**
- 한컴이 이를 어떻게 그리는지 정확 분석 필요
- 변환본 식별 + 한컴 mimicking

**B2. paragraph border_fill_id=NONE 모두인 경우 render skip 강화**
- 회귀 risk 낮음
- 효과: paragraph 외곽선 회귀 차단

**B3. 격차 B 잔존 일부는 후속 issue 분리**

### 5-3. Stage 5 진행 계획

1. 격차 C 가 시각 격차의 주된 원인 → C1+C2 또는 C3 우선 적용
2. 격차 B 는 격차 C fix 후 시각 차이 측정 → 잔존 시 B1+B2 적용 또는 분리

## 6. 추가 진단 필요 항목 (Stage 5 진입 전)

- HWP5 binary 의 한컴 변환본 식별 metadata (binary header / docinfo) 분석
- Shape control 의 fill / wrap 한컴 정합 처리 방식
- 시험지 / aift 등 일반 HWP5 의 ParaShape spacing 패턴 (변환본 식별 정확도 측정)

## 7. Stage 5 진입 결정

격차 C 의 root cause 가 명확 (`hp:switch`) → Stage 5 진입 권고.
HWP5 binary fix 는 가설 검증 필요 → Stage 5 1단계로 가설 A/B/C 검증 + 후보 선정.
