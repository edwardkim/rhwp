# Task #1035 Stage 3 진단 보고서 — p21 case-specific 시도 + fundamental tension

**Issue**: [#1035 HWP3 vs HWP5 변환본 페이지별 paragraph alignment 차이](https://github.com/edwardkim/regression-rhwp/issues/1035)
**Branch**: `local/task1035`
**작업 내용**: Stage 2 의 60/64 alignment 잔존 4 페이지 (대표 p21) case-specific fix 시도 + fundamental tension 분석

---

## 1. p21 미정합 단언

작업지시자 시각 검증으로 p21 미정합 보고:
- HWP3 native p21 첫 paragraph: pi=440 "4. 서버통합 및 원격지 재해복구센터 시스템 구성요건"
- HWP5 변환본 (Task #1035 fix) p21 첫 paragraph: **pi=442** " 󰏅 주전산센터와..."

pi=440, 441 (heading + empty) 가 HWP5 p20 끝에 packing 되어 미정합.

---

## 2. Stage 3 시도 — aux_trigger narrow 가드

### 2.1 시도 항목

PR #1009 의 aux_trigger (empty bridge 휴리스틱) 를 narrow 하여 pi=440 case 정확히 발동 + over-split 회피 시도.

| 시도 | sample16-hwp5 | alignment |
|------|---------------|-----------|
| aux: empty_between≥2 + prev_end > body/2 (PR #1009 원래) | 65 (+1) | 23/64 |
| aux: empty_between≥3 + prev_end > body × 0.75 (Stage 3 narrow) | 65 (+1) | 23/64 |
| aux 완전 제거 (Stage 2 fix) | **64** ✓ | **60/64** ✓ |

→ aux_trigger narrow 시도 모두 over-split 회귀 발생. 어떤 narrow 가드도 효과 없음.

### 2.2 aux_trigger 발동 위치 단언 (RHWP_DEBUG_VARIANT_PAGEBREAK)

```
VPB[typeset]: pi=440 prev_end=57252 curr_first=852 main=false aux=true
```

aux_trigger 가 정확히 1회 발동 (pi=440). 그러나 이 1회 발동이 +1 over-split 직접 원인.

---

## 3. Fundamental Tension 분석

### 3.1 paginator 의 cumulative height vs encoder vpos signal

`./target/release/rhwp dump-pages` 출력 비교:

| | HWP3 native p20 | HWP5 변환본 (Task #1035) p20 |
|---|----------------|----------------------------|
| 마지막 paragraph | pi=439 | **pi=440** (heading) |
| **`used` (rhwp 측정)** | **909.7px** | **971.0px** (거의 body 971.3 끝) |
| **`hwp_used` (encoder vpos)** | 851.6px | **32.7px** (매우 작음) |
| paragraph 평균 height | 작음 (font 작음) | 큼 (HWP3 h=24.9 vs HWP5 h=63.0 등) |

### 3.2 핵심 발견

**HWP5 변환본 encoder 의 vpos signal**:
- pi=440 vpos = **852** (page-reset signal — encoder 가 pi=440 을 새 페이지 시작으로 인코딩)
- p20 의 hwp_used = 32.7px → encoder 의도: page 거의 비어있음

**rhwp paginator 의 cumulative 측정**:
- p20 의 used = 971.0px → rhwp 의 실측: page 거의 full
- pi=440 까지 packing 가능 (971 + pi=440 height 27 = 998 < body 1043)
- **encoder signal 무시 + cumulative height 신뢰**

→ **HWP5 변환본의 paragraph height 자체가 HWP3 보다 큼** (font/spacing). encoder 의 pi=440 page-reset 의도와 rhwp 의 paragraph height 측정이 충돌.

### 3.3 aux_trigger 의 +1 over-split 메커니즘

aux_trigger 가 pi=440 에서 force-break:
- p20 ends at pi=439 (force-break 직전)
- p21 starts at pi=440

그러나 p21 cumulative 측정:
- pi=440 (heading h=27) + pi=441 (lh=2528 큰 height) + pi=442 (box content)
- HWP5 변환본의 paragraph height 가 크므로 p21 cumulative 가 body 초과 가능성
- pi=442 가 p22 로 넘어가 + 후속 페이지 누적 → +1 페이지

이는 HWP5 변환본의 paragraph height 측정 자체가 encoder 의 vpos packing 과 다름. encoder 는 작은 vpos 로 paragraph 들을 page 에 fit 시키나 rhwp 의 height 측정은 다르게 evaluation.

---

## 4. Fundamental Tension — 단순 휴리스틱 해결 불가

### 4.1 본질

```
encoder vpos signal (pi=440 vpos=852 = page-reset)
       ↓
       force-break trigger 발동
       ↓
paginator force-break + page reset
       ↓
paginator 의 새 페이지 cumulative 측정 (paragraph height 기반)
       ↓
paragraph height 가 encoder vpos 가 함의한 "compact packing" 과 불일치
       ↓
+1 페이지 over-split
```

### 4.2 해결 방향 (다음 세션 후보)

**옵션 A — paginator의 variant 모드 도입**:
- HWP5 변환본 한정으로 paginator 가 **encoder vpos 를 cumulative 으로 사용**
- paragraph height 측정 무시 (또는 보조 정보로만 사용)
- 광범위 변경 — paginator 의 핵심 동작 변경

**옵션 B — paragraph height 측정 정합**:
- HWP5 변환본의 paragraph height 가 HWP3 보다 큰 이유 (font metric 또는 spacing) 분석
- height 측정을 HWP3 동등으로 조정
- 본 task 의 격차 D (Task #1008) 와 유사한 폰트 매핑 영역

**옵션 C — force-break 시 cumulative reset 정밀화**:
- aux_trigger 발동 시 paginator state 더 정확하게 reset
- 어떤 state 가 누적되어 +1 페이지 야기하는지 분석 필요

### 4.3 우선순위

옵션 B 가 가장 안전 (paragraph height 정합으로 모든 alignment 동시 향상 가능). 옵션 A 는 광범위 회귀 risk. 옵션 C 는 분석 어려움.

---

## 5. Stage 2 fix 의 평가

Stage 2 fix (main_trigger only + threshold 0.95) 는:
- ✓ sample16-hwp5 페이지 수 64 유지 (PR #1009 회귀 회피)
- ✓ alignment 37.5% → 93.75% (대폭 향상)
- ✗ 잔존 4 미정합 페이지 (p21 등) — fundamental tension 으로 단순 휴리스틱 해결 불가

→ **본 task 범위 내 추가 fix 시 회귀 risk 큼**. Stage 2 fix 그대로 PR 진행하고 잔존 4 페이지는 별도 issue 로 분리 권고.

---

## 6. 한컴 정답지 비교 단언 (Stage 3 추가)

작업지시자 한컴 한글 viewer 정답지 스크린샷 비교:

### 6.1 페이지 alignment 비교 (한컴 정답 p21 vs rhwp p23)

한컴 정답 p21 (footer "-21-") 내용 = rhwp HWP3 p23 (idx=22) 내용:
- 11 items (pi=450~460), 같은 "나. 주전산센터 시스템 구성요건" subheader (pi=456 = 1x1 Table)
- alignment 일치 ✓ (페이지 label 차이만 — document page 21 vs rhwp page 23 numbering offset)

→ **rhwp HWP3 page alignment 는 한컴 정답과 정합**.

### 6.2 HWP5 변환본 p23 overflow 메커니즘

| | rhwp HWP3 p23 | rhwp HWP5 변환본 p23 |
|---|--------------|---------------------|
| items | 11 (pi=450~460) | 11 (pi=450~460, pi=460 PartialParagraph) |
| paragraph height | h=24.9 ~ 111.5 | **h=61.1 ~ 173.9** (~2배) |
| pi=460 처리 | FullParagraph (fits) | **PartialParagraph (split)** |

HWP5 변환본의 paragraph height 가 HWP3 의 **약 2배**. 같은 11 paragraphs 가 HWP3 에서 fit, HWP5 변환본 에서 마지막 paragraph split + 시각적 overflow.

### 6.3 페이지 수 38 vs 64 — 근사값 정정

이전 추정 (한컴 정답 38 페이지) 는 한컴 한글 viewer 의 로딩 시 임시 근사값. **실제 페이지 수 정답은 64 또는 그 근처** (rhwp 와 큰 차이 없음). **별도 issue 등록 대상 아님**.

---

## 7. 잔존 미정합의 본질

본 task #1035 Stage 2 fix 후 60/64 alignment 잔존 4 미정합 페이지는 **HWP5 변환본 paragraph height 가 HWP3 보다 큼** 으로 인해 발생:

- p21 미정합: pi=440 force-break 시 cumulative height 누적 over → +1 over-split (사용 불가)
- p23 overflow: pi=460 PartialParagraph split + 마지막 lines 외곽선 overflow

→ **본질**: HWP5 변환본의 font metric 또는 paragraph height 측정이 HWP3 보다 spread out. Task #1008 격차 D 영역 (폰트 매핑) 연장.

별도 issue 등록 권고: "HWP5 변환본 paragraph height 과대 측정 — HWP3 대비 약 2배, 페이지 alignment 잔존 + content overflow 야기".

---

## 8. 권고 결정

**Stage 2 fix (60/64 alignment + 64 pages 유지)** 가 본 task 의 최선. PR 진행. 잔존 미정합 (p21 + p23 overflow) 은 별도 issue (paragraph height 본질 분석) 로 분리.

작업지시자 옵션 A 결정 정합 → Stage 4 (PR 생성) 진행.
