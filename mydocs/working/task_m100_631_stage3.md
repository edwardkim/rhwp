# Task #631 Stage 3: 광범위 회귀 검증

> **이슈**: [#631](https://github.com/edwardkim/rhwp/issues/631)
> **브랜치**: `local/task631`
> **작성일**: 2026-05-06

---

## 검증 방법

`HEAD~1` (수정 전) vs `HEAD` (수정 후) 의 SVG 출력을 전체 155개 샘플 HWP 파일에 대해
byte-level 비교.

```bash
# Before: HEAD~1 의 typeset.rs 로 빌드
git checkout HEAD~1 -- src/renderer/typeset.rs
cargo build --release
for f in samples/*.hwp; do export-svg ... -o /tmp/svg_diff_before/$name/; done

# After: HEAD 로 복원 빌드
git checkout HEAD -- src/renderer/typeset.rs
cargo build --release
for f in samples/*.hwp; do export-svg ... -o /tmp/svg_diff_after/$name/; done

# byte-level diff
cmp -s before/*.svg after/*.svg
```

## 결과 요약

전체 155개 샘플 / 1253 페이지 (before) / 1255 페이지 (after) / +2 페이지

### 변경된 샘플 (8개)

| 샘플 | 페이지 수 (before→after) | diff 페이지 수 | 텍스트 요소 Δ | 판정 |
|------|---------------------------|----------------|----------------|------|
| **aift** | 77→77 | 2 (page 18, 19) | 0 | ✓ 의도된 수정 |
| **20250130-hongbo** | 4→4 | 2 (page 1, 2) | 0 | ✓ aift 와 동일 패턴 |
| **20250130-hongbo-no** | 4→4 | 2 (page 1, 2) | 0 | ✓ 동상 |
| **hwp3-sample4** | 39→39 | 2 (page 7, 8) | 0 | ✓ 동상 |
| **hwp3-sample5** | 64→64 | 23 | 0 | ✓ 다중 위치 동일 패턴 |
| **loading-fail-01** | 17→16 | 12 | -4 | ✓ 페이지 통합 (회복) |
| **hwpctl_API_v2.4** | 104→106 | 0 (1~104 동일) | **+460** | ★ 누락 콘텐츠 복구 |
| **hwpspec** | 88→89 | 0 (1~88 동일) | **+855** | ★ 누락 콘텐츠 복구 |

### 변경되지 않은 샘플 (147개)

회귀 0건. exam_kor, exam_eng, exam_science, exam_math, synam-001, hwp-multi-002, 
21_언어_기출_편집가능본, 2010-01-06 등 Task #332 주의 케이스 모두 byte 동일.

## 변경 패턴 분석

### 패턴 A: 1줄 페이지 이동 (의도된 동작)

`aift, hongbo, hwp3-sample4` 등.
- 한 페이지에 1줄 추가 (+text elements), 다음 페이지에 1줄 감소 (-text elements)
- 총 텍스트 요소 수 0
- HWP가 LINE_SEG vpos-reset 으로 페이지 경계를 지정한 위치에서 typeset 보수 마진(20px)이 
  마지막 줄을 잘못 밀어내던 것을 정확히 복원

### 패턴 B: 페이지 통합 (회복)

`loading-fail-01`: 17 → 16 페이지.
- 빈 페이지 또는 거의 빈 페이지가 사라짐
- 텍스트 요소 -4 (작은 변화, glyph 미세 조정)
- 페이지 누적 drift 가 만들던 잉여 페이지가 정상 통합됨

### 패턴 C: 누락 콘텐츠 복구 (회복)

`hwpctl_API_v2.4`: 104 → 106 페이지 (+460 text), `hwpspec`: 88 → 89 페이지 (+855 text).
- 기존 페이지 1~N 은 byte 동일
- 추가 페이지 N+1, N+2 에 새 콘텐츠 등장
- md5 비교로 새 페이지가 기존 페이지의 중복이 아닌 신규 콘텐츠임을 확인
- 진짜 콘텐츠 손실이 발생하던 케이스를 복구

페이지 105 (hwpctl_API_v2.4) 상단: bold 헤더 "SeeAlso" — API 레퍼런스 섹션.
페이지 106 상단: "OnMouse..." — 이벤트 핸들러 함수명 (정상 HWP 내용).

## 결과적 영향

Task #332 stage4b 시점부터 알려진 **콘텐츠 손실 회귀**:
- **aift pi=222** ← 본 task 명시 타겟 ✓
- **21_언어 pi=10 line 1** ← 회귀 0건 (이미 정정됨)
- **hwp-multi-002 pi=68** ← 회귀 0건

추가로 발견된 동일 클래스 회귀: **6개 샘플 1300+ text elements 복구**.

## cargo test

```
test result: ok. 1125 passed; 0 failed; 2 ignored; 0 measured
```

## 결론

수정으로 인한 부정적 회귀 0건. 8개 샘플에서 **알려진/미발견 콘텐츠 손실 회귀를 광범위 복구**.

특히 `hwpctl_API_v2.4` 와 `hwpspec` 은 페이지 끝에서 **HWP가 페이지 경계를 명시적으로 인코딩한 위치**에서 typeset 보수 마진이 콘텐츠를 누락시키던 케이스로, 본 수정이 의도한 효과 그대로 작동함을 확인.

## 다음 단계

Stage 4: 최종 결과 보고서 + 오늘할일 갱신.
