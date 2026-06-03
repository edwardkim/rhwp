# Task 403 — 완료 보고서: 역공학용 HWP 샘플 자동 생성

> 2026-03-29

## 구현 내용

### 1. 샘플 생성 프레임워크

- `re_sample_gen.rs`: 테스트 케이스로 HWP 샘플 자동 생성
- `template/empty.hwp` + DocumentCore API(`insert_text_native`, `split_paragraph_native`)로 생성
- LINE_SEG를 비워서(`default`) 저장 → 한컴이 열 때 자체 재계산

### 2. 3종 파일명 체계

| 접미사 | 용도 |
|--------|------|
| `re-*.hwp` | rhwp LINE_SEG 채워진 버전 (rhwp 렌더링 확인용) |
| `re-*-empty.hwp` | LINE_SEG 비운 버전 (한컴에서 열어 저장하기 위한 원본) |
| `re-*-empty-hancom.hwp` | 한컴이 LINE_SEG를 채워 저장한 버전 (역공학 정답지) |

### 3. 생성된 샘플 (17개)

| 카테고리 | 파일 | 검증 대상 |
|----------|------|----------|
| 기본 폭 (6개) | re-01 ~ re-06 | 한글/공백/영문/숫자/혼합/구두점 |
| 폰트별 (7개) | re-font-* | 바탕/바탕체/굴림/굴림체/돋움/돋움체/맑은고딕 |
| 정렬별 (4개) | re-align-* | 양쪽/왼쪽/가운데/오른쪽 |

### 4. 역공학 프로세스 검증

```
rhwp 생성 (*-empty.hwp)
  → 작업지시자가 한컴에서 열기 + 저장 (*-empty-hancom.hwp)
  → rhwp가 한컴 LINE_SEG vs 자체 reflow 비교
  → 차이 패턴에서 한컴 계산 공식 도출
```

## 역공학 결과: 한컴 정답지 비교 (17개 empty-hancom)

| 샘플 | 줄 수 | 줄바꿈 | 전체 |
|------|------|--------|------|
| re-01-hangul-only | 100% | 100% | 100% |
| re-02-space-count | 100% | 100% | 100% |
| re-03-latin-only | 100% | 0% (ts=+1) | 0% |
| re-04-digit-only | 100% | 100% | 100% |
| re-05-mixed-koen | 100% | 0% (ts=-1/-8) | 0% |
| re-06-punctuation | 100% | 100% | 100% |
| re-font-batang | 100% | 100% | 100% |
| re-font-batangche | 100% | 100% | 100% |
| re-font-gulim | 100% | 100% | 100% |
| re-font-gulimche | 100% | 100% | 100% |
| re-font-dotum | 100% | 100% | 100% |
| re-font-dotumche | 100% | 100% | 100% |
| re-font-malgun | 100% | 100% | 100% |
| re-align-justify | 100% | 100% | 100% |
| re-align-left | 100% | 100% | 100% |
| re-align-center | 100% | 100% | 100% |
| re-align-right | 100% | 100% | 100% |
| **합계** | **100%** | **88.2%** | **88.2%** |

## Task 403에서 추가로 수정된 코드

| 수정 | 파일 | 효과 |
|------|------|------|
| BreakToken에 char_widths 필드 추가 | line_breaking.rs | 가변폭 영문 개별 글자 폭 저장 |
| char_level_break_hwp에 개별 폭 전달 | line_breaking.rs | re-04 숫자 100% 해결, re-03 영문 +7→+1 개선 |
| 영문 단어 토큰에 개별 폭 수집 | line_breaking.rs | 토큰화 시 글자별 폭 측정 |

## 잔여 불일치

| 샘플 | 차이 | 원인 추정 | 다음 조치 |
|------|------|----------|----------|
| re-03-latin-only | ts +1/+2 | 양자화 미세 차이 | tolerance 조정 또는 양자화 방식 정밀화 |
| re-05-mixed-koen | ts -1/-8 | 한영 전환 시 폭 누적 방식 차이 | 한영 혼합 전용 샘플로 추가 분석 |

## 핵심 발견

1. **한컴은 LINE_SEG가 비어있어도 자체 재계산** — LINE_SEG는 캐시이며 한컴은 항상 재계산 가능
2. **역공학 프로세스 유효** — empty → 한컴 저장 → 비교로 한컴의 정확한 계산 결과 확보
3. **한글/공백/구두점/폰트별/정렬별은 한컴과 100% 일치** — 기본 조판은 정확
4. **영문 가변폭 char_level_break 정밀도** — 균등 분배에서 개별 폭으로 전환하여 대폭 개선
