# Task #998 Stage 1 — 진단 정밀화

- 이슈: [#998](https://github.com/edwardkim/rhwp/issues/998)
- 부모: [#994](https://github.com/edwardkim/rhwp/issues/994) (PR #997)
- 브랜치: `local/task998`

## 1. HWP3 line_segs 측정 (HWP3 sample16 의 long-text paragraph)

```
pi=380 (cc=54):  ts=[0]                            1 line, ~54 chars
pi=407 (cc=87):  ts=[0, 46]                        2 lines, 46/41
pi=410 (cc=95):  ts=[0, 46, 89]                    3 lines, 46/43/6
pi=437 (cc=82):  ts=[0, 47]                        2 lines, 47/35
pi=442 (cc=214): ts=[0, 45, 87, 131, 173]          5 lines, 45/42/44/42/41
pi=443 (cc=162): ts=[0, 44, 87, 133]               4 lines, 44/43/46/29
pi=444 (cc=134): ts=[0, 46, 92]                    3 lines, 46/46/42
pi=445 (cc=156): ts=[0, 45, 92, 137]               4 lines, 45/47/45/19
pi=446 (cc=196): ts=[0, 53, 107, 159]              4 lines, 53/54/52/37
```

**평균 chars/line**: 43~46 (mid-line, 마지막 partial line 제외)

## 2. HWP5 vs HWP3 ParaShape 비교 (pi=443)

| 속성 | HWP3 | HWP5 |
|------|------|------|
| ParaShape ps_id | 496 | 32 |
| spacing_before | 1132 HU | **2264 HU (2x)** |
| margins.left | 6000 HU | **8000 HU** |
| margins.right | 1000 HU | **2000 HU** |
| indent | -2000 HU | -4000 HU |

**가용 폭 계산**:
- HWP3: sw=51024 HU = 681 px, margins=6500 → ~588 px content width
- HWP5: ParaShape margins=10000 + indent=-4000 → 균형 6000 + 2000 = effective_left=8000-4000=4000 HU? 복잡한 indent 계산
- 결과적 가용 폭은 HWP5 가 좁음 → fewer chars per line possible

## 3. G4 CHARS_PER_LINE 후보 평가

| CHARS_PER_LINE | HWP5 페이지 수 | 시각 |
|----------------|----------------|------|
| 35 (G4 초기) | 67 | 정상 |
| 44 | 65 | 정상 |
| 45 | 65 | 정상 |
| 46 | 65 | 정상 |

→ 44~46 모두 동일한 65 페이지. word boundary break 가 평탄화하기 때문.

## 4. 결론

### 후보 A (fix 가능)
- CHARS_PER_LINE=35 → 44~46 으로 조정
- HWP5 sample16: 67 → 65 페이지 (-2)
- HWP3 reference 64 와 +1 차이

### 후보 B (composer 범위 외)
- HWP5 ParaShape spacing_before 가 HWP3 의 2x
- 59 paragraph × 1132 HU = ~890 px = ~1 페이지
- Hancom 변환기의 데이터 자체 — composer 가 조정 불가

### Fix 선택
**후보 A 만 적용** (CHARS_PER_LINE=45 or 46). 잔존 +1 은 데이터 차이로 수용.
