---
kind: pr-review
status: accepted-pending-integration-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5942 검토 - 10k survey r39 한글 2024 기준선 (#5940, #5941)

## 접수와 범위

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5942](https://github.com/edwardkim/rhwp/pull/5942) / [@planet6897](https://github.com/planet6897) |
| base / source head | `devel` / `1ee84daf72b1f36243988d1b6ba6eb6d8d53535d` |
| 규모 | 2 files, +174 / -1, 1 commit |
| 접수 상태 | non-draft, reviewer `@jangster77` 지정, 작성 시점 `MERGEABLE/CLEAN` |

10k survey r39의 정답지/오라클 기준을 Hancom 2024로 옮기고 devel 회귀 기준을 기록하는 문서·측정 PR이다.
source commit 1/1이 통합 후보에 적용됐다.

## 검증과 판정

- source head의 범위 분류 결과는 5 success, 15 skipped, failure 0이다. skipped는 문서/측정 변경에 대한
  fast-pass 정책 결과이며 실패가 아니다.
- 보고서 [`survey_10k_r39_20260823.md`](../../report/archives/survey_10k_r39_20260823.md)의 기준 전환 근거와
  현재 `upstream/devel`의 HWP 2024 MCP 선택 규칙을 대조했다.
- 통합 branch에는 HWP 2024 MCP 다중 환경 사용 문서도 추가했고, `test-2024.hwp`의 비동기 PDF smoke에서
  queued→succeeded→download, server/client SHA-256 일치와 PDF signature/EOF를 확인했다. 인증값은 기록하지 않는다.

**수용 권고.** 기준선의 제품 선택과 실제 MCP 검증이 일치한다. 통합 PR 최신 fast-pass/CI와 작업지시자
승인이 merge 전 조건이다.
