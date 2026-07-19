---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-310/README.md
last_verified: 2026-07-16
---

# LINE_SEG vpos 패턴 분석 — 4개 샘플 비교

상위: Task #310 (Epic #309 1단계 산출)
도구: `rhwp dump-pages` (vpos 출력), `rhwp export-svg --debug-overlay` (vpos-reset 시각화)
조사일: 2026-04-25

## 핵심 결론

**HWP 파일이 LINE_SEG에 인코딩한 `vertical_pos == 0` 리셋(이하 vpos-reset)은 그 문서가 문단 중간에서 단/페이지를 강제 분리할 의도가 있을 때만 등장한다.** 자연 흐름 문서(exam_math/exam_eng)는 vpos-reset이 0개이며 우리 엔진과 페이지 수가 일치한다.

문제는 vpos-reset이 등장할 때 우리 엔진의 처리 방식이다:
- **PartialParagraph로 자연 분리되는 경우**: 우리 엔진이 단/페이지 가용 공간 부족으로 문단을 분리할 때 우연히 HWP 의도와 일치 → 문제 없음
- **FullParagraph인데 내부에 vpos-reset이 있는 경우**: HWP는 문단 중간을 끊었지만 우리 엔진은 통째로 한 단에 넣음 → 뒷페이지 누적 밀림 발생

이 두 케이스의 분포가 21_언어 +4쪽 과잉의 직접 원인이다.

## 4개 샘플 데이터 표

| 샘플 | PDF 쪽 | SVG 쪽 (현 엔진) | 차이 | vpos-reset 총 개수 | FullParagraph 내부 reset |
|------|--------|------------------|------|--------------------|--------------------------|
| 21_언어 | 15 | 19 | **+4** | 13 | **7** |
| exam_math | 20 | 20 | 0 | 0 | 0 |
| exam_kor | (미보유) | 25 | ? | 8 | 0 |
| exam_eng | (미보유) | 11 | 0 | 0 | 0 |

`FullParagraph 내부 reset`: vpos-reset이 있는데 우리 엔진은 그 문단을 분리하지 않고 단일 페이지의 단일 단에 통째로 배치한 케이스 수.

## 21_언어 vpos-reset 13개 위치 분석

| 페이지 | pi | line | 분류 | 문제? |
|--------|-----|------|------|-------|
| 3 | 50 | 9 | PartialParagraph (단 0/1 분리) | ✗ 일치 |
| 5 | 86 | 8 | PartialParagraph (단 0/1 분리) | ✗ 일치 |
| **7** | **117** | **1** | **FullParagraph** | **✓ 어긋남** |
| **9** | **149** | **6** | **FullParagraph** | **✓ 어긋남** |
| **11** | **181** | **8** | **FullParagraph** + 동반 Shape | **✓ 어긋남** |
| **13** | **211** | **15** | **FullParagraph** | **✓ 어긋남** |
| 15 | 238 | 5 | PartialParagraph (단 0/1 분리) + Shape | ✗ 일치 |
| **17** | **270** | **10** | **FullParagraph** | **✓ 어긋남** |
| **19** | **301** | **1** | **FullParagraph** | **✓ 어긋남** |

**7건의 FullParagraph 내부 vpos-reset = HWP가 문단 중간을 끊을 의도였으나 우리 엔진은 하나의 단/페이지에 통째 배치.** 가장 극단은 페이지 7 pi=117 line=1, 페이지 19 pi=301 line=1: HWP는 첫 줄 직후 페이지 분리 의도였으나 우리 엔진은 전부 한 단에 → 뒷페이지로 줄줄이 밀림.

### 각 어긋남의 vpos 변화

| pi | reset 직전 vpos | reset 직후 vpos | 의미 |
|-----|----------------|-----------------|------|
| 117 | 89700 | 10896 | 줄 1에서 단/페이지 끊고 새 시작 |
| 149 | 81265 | 5448 | 줄 6에서 끊음 |
| 181 | 77316 | 7264 | 줄 8에서 끊음 (Shape 동반) |
| 211 | 64220 | 1816 | 줄 15에서 끊음 |
| 270 | 73300 | 7264 | 줄 10에서 끊음 |
| 301 | 90345 | 36320 | 줄 1에서 끊음 |

직전 vpos는 ~64000~90000 HWPUNIT(약 22~31cm) — body 영역 하단 근처. 즉 HWP는 "이 줄 추가하면 단 하단 넘어가니 다음 단으로" 라는 자연스러운 단 끊기를 LINE_SEG에 박아두었다. 우리 엔진은 이를 무시하고 자체 높이 계산으로 "들어간다"고 판단.

## exam_math: 왜 회귀 없이 일치하는가

vpos-reset 0개. 즉 한컴 오피스도 자연 흐름으로 배치하므로 우리 엔진의 자체 높이 계산 결과가 그대로 PDF와 일치할 수 있다. **이 샘플은 vpos 정보가 페이지네이션에 영향을 주지 않는다** — 가설 4(Column break 비활성화)가 exam_math에서 회귀를 일으킨 이유는 별개의 매커니즘(0x08 플래그)이지 vpos-reset 매커니즘이 아니다.

## exam_kor: 일치 패턴

vpos-reset 8개, 모두 PartialParagraph로 분리됨 → 우리 엔진이 우연히 같은 위치에서 단/페이지 끊음. PDF 미보유로 정확한 일치 확인은 불가하나, FullParagraph 내부 reset이 0건이므로 어긋남이 누적될 매커니즘은 없다.

## exam_eng: 자연 흐름

vpos-reset 0개. exam_math와 동일.

## 도출되는 설계 원칙

1. **vpos-reset이 0인 문서**는 현 엔진과 동일한 자연 흐름 페이지네이션이 정답이다. 새 엔진이 이를 깨면 안 된다.
2. **vpos-reset이 PartialParagraph로 분리되는 경우**는 현 엔진이 우연히 맞춤. 새 엔진은 이 일치를 유지해야 한다.
3. **vpos-reset이 FullParagraph 내부에 있는 경우 (21_언어 7건)**가 진짜 문제. 새 엔진은 이 경우 문단을 분리해서 reset 줄을 새 단/페이지의 첫 줄로 배치해야 한다.

## 권장하는 2단계 작업 (Epic #309)

### Sub-issue #2 후보 (제안)

**제목**: `페이지네이션에서 LINE_SEG vpos-reset을 단/페이지 경계로 강제`

**범위**:
- 페이지네이션 엔진(`src/renderer/pagination/engine.rs`)에서 문단 추가 시 LINE_SEG의 vpos-reset 존재 여부 확인
- vpos-reset이 있는 문단은 reset 줄 직전까지를 현재 단에, reset 줄부터를 다음 단/페이지에 배치 (PartialParagraph 분리 강제)
- 옵션 플래그 `--respect-vpos-reset` 도입(기본 off → on 단계적 전환)

**검증**:
- 21_언어: 19쪽 → 15쪽 (PDF와 일치)
- exam_math: 20쪽 유지 (vpos-reset 0개 → 동작 무변화)
- exam_kor: 25쪽 유지 (FullParagraph 내부 reset 0개 → 동작 무변화)
- exam_eng: 11쪽 유지 (vpos-reset 0개 → 동작 무변화)

**4개 샘플 동시 회귀 0** 가능성이 매우 높다 (FullParagraph 내부 reset이 21_언어에만 있으므로).

### Sub-issue #3 후보 (선택)

**제목**: `페이지네이션 엔진 vpos 우선 모드 (전체 재설계)`

분석 결과 1차원적 해결책(Sub-issue #2)이 4개 샘플을 모두 만족시킬 가능성이 높으므로, 전면 재설계는 다른 회귀 발견 시 후속 검토.

## 측정 도구 명세

본 분석은 다음 도구로 자동화 가능:

```bash
# 샘플별 SVG 페이지 수 + vpos-reset 개수
rhwp dump-pages SAMPLE.hwp | grep -c "^=== 페이지"
rhwp dump-pages SAMPLE.hwp | grep -c "vpos-reset"

# FullParagraph 내부 reset 추출
rhwp dump-pages SAMPLE.hwp | grep -B0 "vpos-reset" | grep "FullParagraph"

# 시각 검증
rhwp export-svg SAMPLE.hwp --debug-overlay -o /tmp/
```

회귀 검증 자동화에 본 도구를 사용한다.

## 부록 A: 가설 검증 결과 (2026-04-25 추가, Task #311)

본 보고서가 권장한 "Sub-issue #2: vpos-reset을 단/페이지 경계로 강제" 가설을 실제 코드(`paginate_with_forced_breaks`) + 실험 플래그(`--respect-vpos-reset`)로 검증한 결과.

### 검증 결과 — 가설 부정

| 샘플 | OFF (기존) | ON (실험) | 평가 |
|------|------------|-----------|------|
| 21_언어 | 19쪽 | **20쪽 (+1)** | ❌ 가설 부정 |
| exam_math | 20쪽 ✓ | 20쪽 ✓ | 무변화 |
| exam_kor | 25쪽 | 25쪽 | 무변화 |
| exam_eng | 11쪽 ✓ | 11쪽 ✓ | 무변화 |

vpos-reset 강제 분리만으로는 21_언어 페이지 수 감소가 일어나지 않으며, 오히려 1쪽 증가.

### 페이지 7 비교 사례 (21_언어, ON 모드)

```
OFF 단 0: pi=115(3..8) + pi=116 + pi=117(전체) ... + pi=127  [13개 항목]
ON  단 0: pi=115(3..8) + pi=116 + pi=117(0..1)              [3개]
ON  단 1: pi=117(1..8) + pi=118 ~ pi=133                    [17개]
       → pi=134 가 다음 페이지로 overflow
```

### 재해석된 진짜 원인

우리 엔진의 column 가용 공간 계산이 HWP의 실제 사용 공간보다 **관대함**:
- HWP: `pi=117 line 0` 까지만 단에 넣고 line 1을 다음 단으로 (HWP 단 사용 공간 ≈ vpos=89700 HWPUNIT)
- 우리: `pi=117 전체(line 0+1, 약 29.4px)`를 한 단에 채움

vpos-reset 강제 분리는 분리는 일으키지만 단 수용량을 압축하지 못함. 즉 pi=117을 강제 분리해도 다른 paragraphs(pi=118 등)는 다음 단으로 밀려 누적 overflow → +1쪽.

### 다음 작업 후보

**Sub-issue #N (제안): column 가용 공간 정확도 조사**
- 21_언어 페이지 7에서 우리 엔진과 HWP 의 column 사용 공간 차이 측정
- 차이의 원인: trailing line_spacing 처리, spacing-after 적용 시점, 줄간격 누적 오차 등
- 차이 보정 후 vpos-reset 강제와 결합 시 21_언어 15쪽 가능 여부 재검증

본 가설 검증을 가능케 한 도구(`--respect-vpos-reset` 실험 플래그)는 향후 결합 검증용으로 코드베이스에 보존.

## 부록 B: 데이터 원본

수집 일자: 2026-04-25
브랜치: task306
커밋: Task #310 1·2단계 적용 후 (`621d0ba` 기준)

원본 dump 출력은 임시 파일(`/tmp/dump_*.txt`)에 저장. 재현은 `rhwp dump-pages`로 즉시 가능.
