# Task #537 Stage 3 완료 보고서

**제목**: 광범위 회귀 검증
**브랜치**: `local/task537`
**이슈**: https://github.com/edwardkim/regex/issues/537

---

## 1. 자동 테스트

```
cargo test --release --lib
test result: ok. 1117 passed; 0 failed; 1 ignored
```

신규 단위테스트 1건 포함, 전체 통과. 회귀 0건.

## 2. 광범위 회귀 검증 (SVG 시프트 분석)

핵심 샘플 7종에 대해 수정 전 / 수정 후 SVG 출력을 비교:

| 샘플 | 변경 페이지 | 평균 dy 패턴 |
|------|-----------|------------|
| `synam-001.hwp` | 16/35 | 미세 (≤+12.80) |
| `복학원서.hwp` | 1/1 | 좌표 변화 없음 |
| `exam_math.hwp` | 18/20 | 혼합 (-15.33 ~ +19.95) |
| `exam_kor.hwp` | 18/20 | 일관 +9.17 (= 687 HU = 1 ls) |
| `exam_eng.hwp` | 6/8 | 일관 +7.68 ~ +14.24 |
| `exam_science.hwp` | 5/6 | 일관 +4.14 ~ +13.07 |
| `2010-01-06.hwp` | 6/6 | 일관 +19.09 (= 2 × 9.55) |

### 2.1 시프트 정합성 검증

`exam_kor.hwp` 페이지 1 의 ① (col 0):
- IR pi=7 vpos=73997, body_y=211.7
- **IR 기대 baseline** = 211.7 + 73997×96/7200 + bl ≈ **1205.19**
- **수정 전**: y=1196.01 (drift -9.17 px = -687 HU)
- **수정 후**: y=1205.19 ✓ (IR 정확)

→ `+9.17` 시프트는 누적 drift 보정 = **정합성 개선**.

### 2.2 음의 시프트 (exam_math)

`exam_math.hwp` 의 ①②③④⑤ 답안 paragraph 가 일부 위치에서 **−14.67 px (= 1 lh)** 시프트 발견:
- 이 paragraph 들은 **수식(Shape) 컨트롤 직후** 위치 (예: pi=18 의 수식 호스트 → pi=19 의 답안)
- Shape + lazy_base 의 상호작용으로 보정 적용 여부가 변동 → 결과 좌표 차이
- 자체 정합성: IR vpos 기준 거리 비교 시 **수정 후가 IR 와 더 가까운 케이스 다수** (별도 검증)

음의 시프트 원인은 lazy_base correction 이 수정 전엔 over-correct 되거나(Shape 가 base 산출에 기여) 혹은 본 task fix 가 새로운 trigger 조건을 활성화시킨 결과로 추정. **단정적 회귀 판단은 시각 비교(한컴 PDF) 후 결정** 필요.

## 3. 본 task 직접 대상 11곳 (재확인)

| 페이지 | 문제 | ①→② gap (수정 전 / 수정 후) | IR 기대 |
|--------|------|---------------------------|--------|
| P2 | 3번 | 63.09 / **72.64** ✓ | 72.64 |
| P3 | 6번 | 63.09 / **72.64** ✓ | 72.64 |
| P5 | 9번 | 14.67 / **24.21** ✓ | 24.21 |
| P6 | 12번 | 38.88 / **48.43** ✓ | 48.43 |
| P8 | 15번 | 38.88 / **48.43** ✓ | 48.43 |
| P9 | 17번 | 38.88 / **48.43** ✓ | 48.43 |
| P9 | 18번 | 38.88 / **48.43** ✓ | 48.43 |
| P12 | 23번 | 38.88 / **48.43** ✓ | 48.43 |
| P12 | 24번 | 38.88 / **48.43** ✓ | 48.43 |
| P13 | 27번 | 38.88 / **48.43** ✓ | 48.43 |
| P14 | 29번 | 38.88 / **48.43** ✓ | 48.43 |

작업지시자 명시한 11곳 모두 IR vpos delta 와 일치하는 정상 gap.

## 4. 잔존/주의 사항

### 4.1 base=716 잔존 (회귀 아님)
21_언어_기출.hwp 의 다른 페이지(pi=147 등) 에 `base=716` 잔존. 수정 전에도 존재 → 회귀 아님. 별도 메커니즘 (prev_pi 의 line_segs.last() 의 line_spacing 가 패치에 사용되지만, 실 paragraph 의 rendered last line 이 다른 경우 발생). 별도 issue 검토 후보.

### 4.2 광범위 시프트
exam_kor / exam_eng / exam_science / 2010-01-06 등에서 페이지당 수십~수백 paragraph 가 시프트. 한 케이스(exam_kor pi=7) 검증 결과 **IR vpos 정합 개선**. 다른 케이스도 동일 메커니즘일 가능성 높음.

→ **작업지시자에게 한컴 PDF 시각 비교를 통한 최종 판단 권고** (메모리 룰: PDF 비교 결과는 절대 기준이 아님 — 한컴 2010/2020 환경 함께 점검).

### 4.3 exam_math 의 음의 시프트
수식(Shape) 직후 paragraph 에서 `-14.67 px` (= 1 lh) 시프트. lazy_base 와 Shape correction 상호작용. 회귀 가능성 배제 못함 → **시각 비교 필요**.

### 4.4 Clippy 기존 결함 (별도 issue 후보)
```
src/document_core/commands/table_ops.rs:1007
src/document_core/commands/object_ops.rs:298
```
본 task 와 무관하게 사전 존재.

## 5. 산출물

| 파일 | 변경 |
|------|------|
| (Stage 1+2 의 변경) | `src/renderer/layout.rs`, `integration_tests.rs` |
| `mydocs/working/task_m100_537_stage3.md` | 본 보고서 |

## 6. 다음 단계 권고

1. **최종 보고서 작성** (`mydocs/report/task_m100_537_report.md`)
2. **작업지시자 시각 비교**:
   - 21_언어_기출_편집가능본 11곳 SVG vs 한컴 2010/2020 PDF
   - exam_kor / exam_math / 2010-01-06 광범위 시프트 표본 검토
3. 시각 비교에서 회귀 발견 시:
   - lazy_base 보정 가드 추가 (예: prev_pi 가 FullParagraph PageItem 일 때만 적용)
   - `local/task537_v2` 브랜치로 후속 수정
4. 회귀 없으면 `local/devel` merge 진행

## 7. 승인 요청

Stage 3 완료. 최종 보고서 작성 + merge 진행 승인 요청.
