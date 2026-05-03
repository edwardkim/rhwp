# Task #539 Stage 3 완료 보고서

**제목**: 광범위 회귀 검증 + 그룹 A 결정
**브랜치**: `local/task539`
**이슈**: https://github.com/edwardkim/rhwp/issues/539

---

## 1. 자동 테스트

```
cargo test --release --lib
test result: ok. 1119 passed; 0 failed; 1 ignored
```

회귀 0건.

## 2. 광범위 회귀 검증 (8개 핵심 샘플)

수정 전(Stage 1 commit, layout.rs 미수정) vs 수정 후(Stage 2 commit, 가드 완화) SVG 비교:

| 샘플 | 변경 페이지 | 시프트 패턴 |
|------|-----------|------------|
| `21_언어_기출_편집가능본.hwp` | 10/15 | (직접 대상) 정합성 개선 |
| `synam-001.hwp` | **0/35** | ✅ 변경 없음 |
| `복학원서.hwp` | **0/1** | ✅ 변경 없음 |
| `exam_math.hwp` | **0/20** | ✅ 변경 없음 (Task #537 의 음의 시프트 그대로 — 본 task 무관) |
| `exam_kor.hwp` | **0/20** | ✅ 변경 없음 |
| `exam_eng.hwp` | 1/8 | exam_eng_002.svg 의 151 paragraph 가 +7.68 px (한쪽 방향) 시프트 = 정합성 개선 |
| `exam_science.hwp` | **0/6** | ✅ 변경 없음 |
| `2010-01-06.hwp` | **0/6** | ✅ 변경 없음 |

**모든 샘플에서 음의 시프트 0건** (= 회귀 없음).
변경된 페이지는 본 task 직접 대상 또는 동일 본질의 추가 정합성 개선.

본 fix 는 매우 국소적: `treat_as_char=true + InFrontOfText/BehindText` Shape 가 prev_pi 의 controls 에 있는 경우 only — 대부분 문서에서 발생하지 않는 조건.

## 3. 본 task 직접 대상 정량 (재확인)

| 위치 | 이전 (px) | 수정 후 (px) | IR 기대 (px) |
|------|----------|-------------|-------------|
| 7p pi=145 → pi=146 (르포르) | 14.67 | **24.21** ✓ | 24.21 |
| 9p pi=181 → pi=182 (수피즘) | 14.67 | **24.21** ✓ | 24.21 |

두 케이스 모두 IR vpos delta 와 정합.

## 4. 그룹 A 처리 결정 (작업지시자 결정 사항)

### 4.1 그룹 A 현황 (사전 분석 시 측정)

페이지 2 [4~6] → 지문 gap = 33.01 px = IR 정확.
- pi=44 ParaShape: line=165%, ls=716
- pi=45 (빈 paragraph) ParaShape: **line=60%, ls=−440** ← 음수
- pi=46 ParaShape: line=165%, ls=716

rhwp 는 IR vpos 정확 따름. 한컴 PDF 가 더 넓다면 한컴은 60% 음수 ls 를 다르게 처리.

### 4.2 의심 본질 (한컴 환경 검증 필요)

1. 한컴 60% line spacing 의 명세 동작 vs rhwp 구현 차이
2. 빈 paragraph 의 visual height 처리 (음수 ls 의 floor 적용 가능성)

### 4.3 본 task 분리 결정

메모리 룰 적용:
- "PDF 비교 결과는 절대 기준이 아님" — 한컴 2010/2020 환경 비교 필요
- "룰과 휴리스틱 구분" — 60% ls 동작이 명세 룰인지 휴리스틱인지 미확인
- "본질 정정 회귀 위험" — 빈 paragraph / 음수 ls 처리는 광범위 영향

→ **그룹 A 는 본 task 에서 처리하지 않고 별도 issue 로 분리** 권고.

## 5. 잔존 사항

본 task 직접 대상(그룹 B) 은 모두 fix. 잔존:

1. **그룹 A** (지문 시작 [X~Y] → 지문 사이 줄간격, 9곳): 별도 task 분리 필요 (한컴 환경 검증 후)
2. **base=716 잔존 케이스** (Task #537 잔존): 본 task 와 별개 메커니즘. 별도 issue 검토 후보.

## 6. 산출물

| 파일 | 변경 |
|------|------|
| (Stage 1+2 의 변경) | `src/renderer/layout.rs`, `integration_tests.rs` |
| `mydocs/working/task_m100_539_stage3.md` | 본 보고서 |

## 7. 다음 단계

1. **최종 보고서 작성** (`mydocs/report/task_m100_539_report.md`)
2. 그룹 A 별도 issue 등록 (작업지시자 승인 후)
3. `local/devel` merge → `devel` push → upstream PR

## 8. 승인 요청

Stage 3 완료. 최종 보고서 + merge 진행 + 그룹 A 별도 issue 등록 승인 요청.
