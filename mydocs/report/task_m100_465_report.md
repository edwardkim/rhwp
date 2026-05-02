# Task #465 최종 보고서 — 재현 불가 (Cannot Reproduce)

**이슈**: #465 — exam_kor 16p 우측 "홀수형" 뱃지 누락 (master 외 별도 요소)
**브랜치**: `local/task465`
**상태**: **close (재현 불가)**

## 1. 검증 절차

1. 현 시점 release build 로 `samples/exam_kor.hwp` 페이지 13~16 SVG 추출
2. 각 페이지 SVG 에서 "홀" 텍스트 위치 추출
3. 신고 시점 commit `c71fa49` 로 `src/` 체크아웃 후 동일 검증

## 2. 결과

| 페이지 | 현재 SVG 위치 | 신고 시점 위치 | PDF 위치 | 일치 |
|--------|---------------|----------------|----------|------|
| 13 (cover) | x=906, y=261 우 | (동일) | 우 | ✓ |
| 14 (master[0]) | x=933, y=169 우 | (동일) | 우 | ✓ |
| 15 (master[1]) | x=128, y=171 좌 | (동일) | 좌 | ✓ |
| 16 (master[2] ext) | **x=933, y=169 우** | **x=933, y=169 우** | 우 | ✓ |

→ 현 시점 + 신고 시점 모두 SVG 가 4페이지 모두 PDF 와 동일 위치에 "홀수형" 뱃지 렌더링.

## 3. 결함 원천 분석

"홀수형" 글상자는 **sec1 paragraph 0 (cover)** 의 `사각형` 컨트롤 (`InFrontOfText`) 안에 있음. master 컨텐츠 아님.

`InFrontOfText` wrap 으로 sec1 모든 페이지에 동일 좌표 자동 stamp. 좌/우 위치는 sec1 첫 페이지 컨트롤 좌표가 page parity 와 무관하게 적용된 결과 — section-level decoration 으로 정상 동작.

이슈 본문의 "master cell[0] 의 도형 추정" 가설은 부정확. 실제로는 sec1 cover paragraph 의 InFrontOfText 사각형 글상자가 source.

## 4. 처리

- 코드 변경: 없음
- 이슈 close (재현 불가)
- 신고자 (planet6897) 에게 검증 결과 코멘트 후 close

## 5. 후속

본 검증으로 sec1 cover (pi=0) 의 InFrontOfText 사각형이 모든 sec1 페이지에 자동 stamp 되는 것이 정상 동작임을 확인. master 처리 로직 변경 필요 없음.
