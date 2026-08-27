---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6210 review - #6184 이월 직전 host 꼬리 줄 보존

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6210
- 작성자: `planet6897`
- 원 PR head: `e97c1db04070`
- 통합 검토 브랜치: `review/planet6897-6199-6217-20260827`
- 기준: `upstream/devel@584320e0ee02`
- 원 PR 상태: non-draft, source CI green, comments/reviews 0건
- 관련 이슈: #6184

## 검토 판단

**수용 권고**. rowbreak 직전 host 꼬리 줄이 저장 line segment상 현재 body 안에서 끝나고 다음 문단의
vpos가 reset되는 경우에만 pre-emit을 허용한다. 또한 host fit 판정에는 말미 line spacing을 제외한
`height_for_fit`을 사용하고, typeset/paint 양쪽에서 pre-emitted host paragraph를 확인해 다음 쪽 중복
그림을 막는다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/issue-6184-tail-line-before-deferred-table/{before_p1_bottom,after_p1_bottom,before_p2_top,after_p2_top}.png`
- 검토자가 직접 확인한 대표 after: 꼬리 줄은 이전 쪽 하단에 남고, 다음 쪽 상단 흐름표 위 중복 텍스트가
  사라짐
- 파일 버전 증적: `mydocs/pr/assets/pr_6199_6217_156489124_tail_line_before_deferred_table_hwp_info.json`
- focused test: `issue_6184_tail_line_before_deferred_table` 1 pass
- 공통 검증: fmt, suite manifest, unit tier, clippy, 전체 nextest, Native Skia 3종, WASM build 통과.
  상세 명령과 숫자는 통합 구현 문서에 기록했다.

## 후속

통합 PR에는 pre-emit 허용 조건과 중복 방지 가드가 모두 들어갔음을 명시한다.
