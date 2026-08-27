---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6249 review - CIRCLED/GANADA 번호 포맷 OOB 방어

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6249
- 작성자: `kevin9327`
- 원 PR head: `e11ab9e89b07`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@5645e1f5b`
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건

## 검토 판단

**수용 권고.** 손상된 자동번호 값 `0` 또는 배열 범위 밖 값이 `CIRCLED[(n - 1)]`,
`GANADA[(n - 1)]`로 들어가 overflow/OOB panic을 만들 수 있는 경로를 `checked_sub()`와 `.get()`으로
방어했다. 정상 범위 출력은 유지하고 비정상 값은 숫자 문자열로 fallback한다.

## 증적과 검증

- 원 PR 보고서: `mydocs/report/bug-circled/README.md`
- 원 PR SVG 증적:
  `mydocs/report/bug-circled/{before,after,before_p2,after_p2}.svg`
- focused lib test: `cargo test --locked --lib test_format_number --target-dir target/pr-review` 5 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 후속

추가 보정 필요 없음. 방어적 fallback이라 정상 문서 렌더 출력 변화는 기대하지 않는다.
